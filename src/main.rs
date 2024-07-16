use std::io::{Read, Write};

use flakes::*;

struct Dummy;

impl Builder for Dummy {
    fn build(source: impl Read, input: &serde_json::Value, store: &mut Store<Single, Self>) -> std::io::Result<serde_json::Value> {
        std::thread::sleep(std::time::Duration::from_secs(5));
        store.create_file("output")?.write(b"Hello, world!")?;
        Ok(serde_json::Value::Null)
    }
}

fn main() -> Result<(), std::io::Error> {
    let store = store::<Dummy>();
    let input = serde_json::json!({"a":2});
    let (path, _) = store.create("dummy", "Hello, world!".as_bytes(), &input)?;
    Ok(())
}
