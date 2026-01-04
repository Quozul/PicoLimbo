use indexmap::IndexMap;
use pico_nbt2::{
    NbtOptions, Value, from_path, from_path_struct, from_path_with_options, from_slice_struct,
};
use serde::Deserialize;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_hello_world_decode() {
    // Given
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("hello_world.nbt");

    // When
    let (name, value) = from_path(&path).expect("Failed to parse hello_world.nbt");

    // Then
    assert_eq!(name, "hello world");
    assert_eq!(
        value,
        Value::Compound(IndexMap::from([(
            "name".into(),
            Value::String("Bananrama".into())
        ),]))
    );
}

#[test]
fn test_bigtest() {
    // Given
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("bigtest.nbt");

    let mut value = Vec::new();
    for n in 0..1000 {
        let r = ((n * n * 255 + n * 7) % 100) as u8;
        value.push(r);
    }
    let expected_byte_array = Value::ByteArray(value);

    // When
    let (name, value) = from_path(&path).expect("Failed to parse bigtest.nbt");

    // Then
    assert_eq!(name, "Level");
    assert_eq!(
        value,
    Value::Compound(IndexMap::from([
        ("intTest".into(), Value::Int(2_147_483_647)),
        ("byteTest".into(), Value::Byte(127)),
        (
            "stringTest".into(),
            Value::String("HELLO WORLD THIS IS A TEST STRING ÅÄÖ!".into())
        ),
        ("doubleTest".into(), Value::Double(0.493_128_713_218_231_5)),
        ("floatTest".into(), Value::Float(0.498_231_47_f32)),
        ("longTest".into(), Value::Long(9_223_372_036_854_775_807)),
        ("shortTest".into(), Value::Short(32767)),
        (
            "listTest (long)".into(),
            Value::List(vec![
                Value::Long(11),
                Value::Long(12),
                Value::Long(13),
                Value::Long(14),
                Value::Long(15),
            ])
        ),
        (
            "nested compound test".into(),
            Value::Compound(IndexMap::from([
                (
                    "egg".into(),
                    Value::Compound(IndexMap::from([
                        ("name".into(), Value::String("Eggbert".into())),
                        ("value".into(), Value::Float(0.5)),
                    ])),
                ),
                (
                    "ham".into(),
                    Value::Compound(IndexMap::from([
                        ("name".into(), Value::String("Hampus".into())),
                        ("value".into(), Value::Float(0.75)),
                    ])),
                )
            ]))
        ),
        (
            "listTest (compound)".into(),
            Value::List(vec![Value::Compound(IndexMap::from([
                ("created-on".into(), Value::Long(1_264_099_775_885)),
                ("name".into(), Value::String("Compound tag #0".into()))
            ])), Value::Compound(IndexMap::from([
                ("created-on".into(), Value::Long(1_264_099_775_885)),
                ("name".into(), Value::String("Compound tag #1".into()))
            ]))])
        ),
        (
            "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))".into(), expected_byte_array
        )
    ]))
    );
}

#[test]
fn test_bigtest_struct() {
    #[derive(Debug, Deserialize)]
    struct ListTestCompound {
        #[serde(rename = "created-on")]
        created_on: i64,
        name: String,
    }
    #[derive(Debug, Deserialize)]
    struct NestedCompoundTest {
        name: String,
        value: f32,
    }
    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct BigTest {
        int_test: i32,
        byte_test: i8,
        string_test: String,
        double_test: f64,
        float_test: f32,
        long_test: i64,
        short_test: i16,
        #[serde(rename = "listTest (long)")]
        list_test_long: Vec<i64>,
        #[serde(rename = "nested compound test")]
        nested_compound_test: HashMap<String, NestedCompoundTest>,
        #[serde(rename = "listTest (compound)")]
        list_test_compound: Vec<ListTestCompound>,
        #[serde(
            rename = "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))"
        )]
        #[serde(with = "serde_bytes")]
        byte_array_test: Vec<u8>,
    }
    // Given
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("bigtest.nbt");

    let mut expected_byte_array = Vec::new();
    for n in 0..1000 {
        let r = ((n * n * 255 + n * 7) % 100) as u8;
        expected_byte_array.push(r);
    }

    // When
    let (name, value) = from_path_struct::<BigTest>(&path).expect("Failed to parse bigtest.nbt");

    // Then
    assert_eq!(name, "Level");
    // 1. Primitive Integers
    assert_eq!(value.int_test, 2_147_483_647, "int_test mismatch");
    assert_eq!(value.byte_test, 127, "byte_test mismatch");
    assert_eq!(value.short_test, 32_767, "short_test mismatch");
    assert_eq!(
        value.long_test, 9_223_372_036_854_775_807,
        "long_test mismatch"
    );

    // 2. Strings
    assert_eq!(
        value.string_test, "HELLO WORLD THIS IS A TEST STRING ÅÄÖ!",
        "string_test mismatch"
    );

    // 3. Floating Point
    // Note: Exact equality checks on floats are usually discouraged,
    // but for deserialization tests, we expect the exact bit pattern.
    assert_eq!(
        value.double_test, 0.493_128_713_218_231_5,
        "double_test mismatch"
    );
    assert_eq!(value.float_test, 0.498_231_47_f32, "float_test mismatch");

    // 4. Simple List (Long Array)
    assert_eq!(
        value.list_test_long,
        vec![11, 12, 13, 14, 15],
        "list_test_long mismatch"
    );

    // 5. Nested Map (HashMap<String, NestedCompoundTest>)
    // Check "egg"
    let egg = value
        .nested_compound_test
        .get("egg")
        .expect("Missing 'egg' in nested_compound_test");
    assert_eq!(egg.name, "Eggbert", "egg.name mismatch");
    assert_eq!(egg.value, 0.5, "egg.value mismatch");

    // Check "ham"
    let ham = value
        .nested_compound_test
        .get("ham")
        .expect("Missing 'ham' in nested_compound_test");
    assert_eq!(ham.name, "Hampus", "ham.name mismatch");
    assert_eq!(ham.value, 0.75, "ham.value mismatch");

    // 6. List of Compounds (Vec<ListTestCompound>)
    assert_eq!(
        value.list_test_compound.len(),
        2,
        "list_test_compound length mismatch"
    );

    // Item #0
    let item0 = &value.list_test_compound[0];
    assert_eq!(
        item0.name, "Compound tag #0",
        "list_test_compound[0].name mismatch"
    );
    assert_eq!(
        item0.created_on, 1_264_099_775_885,
        "list_test_compound[0].created_on mismatch"
    );

    // Item #1
    let item1 = &value.list_test_compound[1];
    assert_eq!(
        item1.name, "Compound tag #1",
        "list_test_compound[1].name mismatch"
    );
    assert_eq!(
        item1.created_on, 1_264_099_775_885,
        "list_test_compound[1].created_on mismatch"
    );

    // 7. Byte Array Generation & Validation
    assert_eq!(
        value.byte_array_test, expected_byte_array,
        "byte_array_test content mismatch"
    );
}

#[test]
fn test_player_nan() {
    // Given
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("Player-nan-value.dat");

    // When
    let (name, value) = from_path(&path).expect("Failed to parse Player-nan-value.dat");

    // Then
    assert!(name.is_empty());

    let y = value
        .get_compound()
        .expect("expected compound value")
        .get("Pos")
        .expect("expected Pos element")
        .get_list()
        .expect("expected list")
        .get(1)
        .expect("no y position")
        .get_double()
        .expect("expected double");

    assert!(y.is_nan());
}

#[test]
fn test_hello_world_struct() {
    // Given
    #[derive(Deserialize, Debug, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct HelloWorld {
        name: String,
    }

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("hello_world.nbt");

    // When
    let bytes = fs::read(&path).expect("Failed to read hello_world.nbt");
    let (name, hello) = from_slice_struct::<HelloWorld>(&bytes)
        .expect("Failed to parse hello_world.nbt into struct");

    // Then
    assert_eq!(name, "hello world");
    assert_eq!(hello.name, "Bananrama");
}

#[test]
fn test_nameless_root_hello_world_decode() {
    // Given
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("test_files");
    path.push("nameless_root_hello_world.nbt");
    let options = NbtOptions::new().nameless_root(true);

    // When
    let (name, value) = from_path_with_options(&path, options).expect("Failed to parse");

    // Then
    assert!(name.is_empty());
    assert_eq!(
        value,
        Value::Compound(IndexMap::from([(
            "name".into(),
            Value::String("Bananrama".into()),
        )]))
    );
}
