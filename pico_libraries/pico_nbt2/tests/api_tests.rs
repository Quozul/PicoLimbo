use pico_nbt2::{Value, from_file, from_path};
use std::fs::File;
use std::path::PathBuf;

#[test]
fn test_from_file_hello_world() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("hello_world.nbt");

    let mut file = File::open(&path).expect("Failed to open hello_world.nbt");
    let (name, value) = from_file(&mut file).expect("Failed to parse hello_world.nbt from file");

    assert_eq!(name, "hello world");
    if let Value::Compound(map) = value {
        assert_eq!(map.get("name"), Some(&Value::String("Bananrama".into())));
    } else {
        panic!("Expected root to be a Compound");
    }
}

#[test]
fn test_from_path_hello_world() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("hello_world.nbt");

    let (name, value) = from_path(&path).expect("Failed to parse hello_world.nbt from path");

    assert_eq!(name, "hello world");
    if let Value::Compound(map) = value {
        assert_eq!(map.get("name"), Some(&Value::String("Bananrama".into())));
    } else {
        panic!("Expected root to be a Compound");
    }
}

#[test]
fn test_from_path_bigtest_compressed() {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("bigtest.nbt");

    let (name, value) = from_path(&path).expect("Failed to parse bigtest.nbt from path");

    assert_eq!(name, "Level");
    if let Value::Compound(map) = value {
        assert!(map.contains_key("intTest"));
    } else {
        panic!("Expected root to be a Compound");
    }
}
