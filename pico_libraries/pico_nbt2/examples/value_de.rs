use pico_nbt2::Value;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct MyStruct {
    name: String,
    id: i32,
    active: bool,
}

fn main() -> anyhow::Result<()> {
    let mut map = indexmap::IndexMap::new();
    map.insert("name".into(), Value::String("Bananrama".into()));
    map.insert("id".into(), Value::Int(42));
    map.insert("active".into(), Value::Byte(1));
    let value = Value::Compound(map);

    let obj: MyStruct = pico_nbt2::from_value(value)?;

    println!("Deserialized: {obj:?}");

    Ok(())
}
