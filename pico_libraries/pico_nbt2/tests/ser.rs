use indexmap::IndexMap;
use pico_nbt2::{CompressionType, Value, encode};
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

    let mut encoded_bytes = Vec::new();
    let mut encoder = encode(&mut encoded_bytes, CompressionType::None).unwrap();
    pico_nbt2::to_writer(&mut encoder, &value, Some("hello world")).unwrap();
    drop(encoder);

    assert_eq!(encoded_bytes, expected_bytes);
}
