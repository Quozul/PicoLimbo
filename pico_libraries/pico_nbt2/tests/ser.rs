use indexmap::IndexMap;
use pico_nbt2::{CompressionType, Value};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_hello_world_encode() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("hello_world.nbt");

    let expected_bytes = fs::read(&path).expect("Failed to read hello_world.nbt");

    let mut map = IndexMap::new();
    map.insert("name".into(), "Bananrama".into());
    let value = Value::Compound(map);

    let encoded_bytes = value
        .to_byte(
            CompressionType::None,
            pico_nbt2::NbtOptions::new(),
            Some("hello world"),
        )
        .unwrap();

    assert_eq!(encoded_bytes, expected_bytes);
}

#[test]
fn test_nameless_root_hello_world_encode() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("nameless_root_hello_world.nbt");

    let expected_bytes = fs::read(&path).expect("Failed to read nameless_root_hello_world.nbt");

    let mut map = IndexMap::new();
    map.insert("name".into(), "Bananrama".into());
    let value = Value::Compound(map);

    let encoded_bytes = value
        .to_byte(
            CompressionType::None,
            pico_nbt2::NbtOptions::new().nameless_root(true),
            None,
        )
        .unwrap();

    assert_eq!(encoded_bytes, expected_bytes);
}
