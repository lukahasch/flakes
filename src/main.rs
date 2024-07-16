use std::io::{Read, Write};

use flakes::*;

struct Dummy;

impl Builder for Dummy {
    fn build(source: impl Read, input: &serde_json::Value, store: &mut Store<Single, Self>) -> std::io::Result<serde_json::Value> {
        let mut buf = String::new();
        store.open_file("./src/main.rs")?.read_to_string(&mut buf)?;
        store.create_file("output")?.write(b"Hello, world!")?;
        Ok(serde_json::Value::String(buf))
    }
}

fn main() -> Result<(), std::io::Error> {
    let store = store::<Dummy>();
    let input = serde_json::json!({"a":2});
    let (path, output) = store.create("dummy", "Hello, world!".as_bytes(), &input)?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}
