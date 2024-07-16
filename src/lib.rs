use piccolo::{Closure, Executor, Function, Lua, Value};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::{env, fs::File};

pub mod store;

pub trait Source {
    fn hash(&self) -> u64;
    fn aquire(self) -> Result<impl Read, std::io::Error>;
}

impl<'a> Source for &'a str {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        <&'a str as Hash>::hash(self, &mut hasher);
        hasher.finish()
    }

    fn aquire(self) -> Result<impl Read, std::io::Error> {
        Ok(std::io::Cursor::new(self))
    }
}

impl Source for File {
    fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now().hash(&mut hasher);
        hasher.finish()
    }

    fn aquire(self) -> Result<impl Read, std::io::Error> {
        Ok(self)
    }
}

pub fn to_json<'gc>(value: Value<'gc>) -> serde_json::Value {
    match value {
        Value::Boolean(bool) => serde_json::Value::Bool(bool),
        Value::Integer(int) => serde_json::Value::Number(int.into()),
        Value::String(string) => serde_json::Value::String(string.to_string()),
        Value::Table(table) => {
            let mut map = serde_json::Map::new();
            for (key, value) in table {
                map.insert(
                    match key {
                        Value::String(string) => string.to_string(),
                        _ => panic!("Unsupported key type, {:?}", key.type_name()),
                    },
                    to_json(value),
                );
            }
            serde_json::Value::Object(map)
        }
        Value::Nil => serde_json::Value::Null,
        _ => panic!("Unsupported value type, {:?}", value.type_name()),
    }
}

pub fn flake<S: Source>(
    code: S,
    input: &serde_json::Value,
) -> Result<serde_json::Value, std::io::Error> {
    let code = code.aquire()?;
    let mut lua = Lua::core();
    let exec = lua
        .try_enter(|ctx| {
            let closure = Closure::load(ctx, Some("<flake>"), code)?;
            Ok(ctx.stash(Executor::start(ctx, closure.into(), ())))
        })
        .map_err(|e| std::io::Error::other(e))?;
    lua.finish(&exec);
    lua.try_enter(|ctx| {
        let function = ctx.fetch(&exec).take_result::<Function>(ctx)??;
        let input = piccolo_util::serde::to_value(ctx, input)?;
        ctx.fetch(&exec).restart(ctx, function, input);
        Ok(())
    })
    .map_err(|e| std::io::Error::other(e))?;
    lua.finish(&exec);
    lua.try_enter(|ctx| {
        let result = ctx.fetch(&exec).take_result::<Value>(ctx)??;
        Ok(to_json(result))
    })
    .map_err(|e| std::io::Error::other(e))
}
