use pico_nbt2::{Value, to_bytes};
use serde::Serialize;
use std::collections::HashMap;

#[test]
fn test_struct_serialization() {
    #[derive(Serialize)]
    struct MyStruct {
        int: i32,
        string: String,
        float: f32,
        bool: bool,
    }

    let s = MyStruct {
        int: 42,
        string: "test".to_string(),
        float: std::f32::consts::PI,
        bool: true,
    };

    let bytes = to_bytes(&s, Some("my_struct")).unwrap();
    let (name, value) = pico_nbt2::from_slice(&bytes).unwrap();

    assert_eq!(name, "my_struct");
    let Value::Compound(map) = value else {
        panic!("Expected Compound")
    };
    assert_eq!(map.get("int"), Some(&Value::Int(42)));
    assert_eq!(map.get("string"), Some(&Value::String("test".into())));
    assert_eq!(map.get("float"), Some(&Value::Float(std::f32::consts::PI)));
    assert_eq!(map.get("bool"), Some(&Value::Byte(1)));
}

#[test]
fn test_nested_struct_serialization() {
    #[derive(Serialize)]
    struct Inner {
        val: i16,
    }
    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
        list: Vec<i32>,
    }

    let s = Outer {
        inner: Inner { val: 100 },
        list: vec![1, 2, 3],
    };

    let bytes = to_bytes(&s, Some("outer")).unwrap();
    let (_, value) = pico_nbt2::from_slice(&bytes).unwrap();
    let Value::Compound(map) = value else {
        panic!("Expected Compound")
    };

    let Value::Compound(inner) = map.get("inner").unwrap() else {
        panic!("Expected Inner Compound")
    };
    assert_eq!(inner.get("val"), Some(&Value::Short(100)));

    let Value::List(list) = map.get("list").unwrap() else {
        panic!("Expected List")
    };
    assert_eq!(list.len(), 3);
    assert_eq!(list[0], Value::Int(1));
}

#[test]
fn test_map_serialization() {
    let mut map = HashMap::new();
    map.insert("key1".to_string(), 1);
    map.insert("key2".to_string(), 2);

    let bytes = to_bytes(&map, Some("map")).unwrap();
    let (_, value) = pico_nbt2::from_slice(&bytes).unwrap();
    let Value::Compound(nbt_map) = value else {
        panic!("Expected Compound")
    };

    assert_eq!(nbt_map.len(), 2);
    assert!(nbt_map.contains_key("key1"));
    assert!(nbt_map.contains_key("key2"));
}

#[test]
fn test_enum_serialization() {
    #[derive(Serialize)]
    enum MyEnum {
        VariantA(i32),
        VariantB { x: f64 },
        VariantC,
    }

    let a = MyEnum::VariantA(10);
    let bytes_a = to_bytes(&a, Some("a")).unwrap();
    let (_, val_a) = pico_nbt2::from_slice(&bytes_a).unwrap();
    // VariantA(10) -> { "VariantA": 10 } (newtype variant)
    // Wait, my implementation:
    // serialize_newtype_variant -> { variant: value }
    if let Value::Compound(map) = val_a {
        assert_eq!(map.get("VariantA"), Some(&Value::Int(10)));
    } else {
        panic!("Expected Compound for VariantA");
    }

    let b = MyEnum::VariantB { x: 1.5 };
    let bytes_b = to_bytes(&b, Some("b")).unwrap();
    let (_, val_b) = pico_nbt2::from_slice(&bytes_b).unwrap();
    // VariantB { x } -> { "VariantB": { "x": 1.5 } } (struct variant)
    if let Value::Compound(map) = val_b {
        if let Some(Value::Compound(inner)) = map.get("VariantB") {
            assert_eq!(inner.get("x"), Some(&Value::Double(1.5)));
        } else {
            panic!("Expected inner compound for VariantB");
        }
    } else {
        panic!("Expected Compound for VariantB");
    }

    let c = MyEnum::VariantC;
    let bytes_c = to_bytes(&c, Some("c")).unwrap();
    let (_, val_c) = pico_nbt2::from_slice(&bytes_c).unwrap();
    // VariantC -> "VariantC" (unit variant)
    // Wait, unit variant -> serialize_str(variant) -> Value::String("VariantC")
    // But to_bytes expects the root to be a Compound (usually).
    // If to_bytes is called with a String value, it writes Tag_String.
    // But NBT root MUST be Compound (usually).
    // My `to_writer` writes whatever `Value` is passed.
    // If `Value` is String, it writes Tag_String as root.
    // This is valid NBT (technically root can be anything, but usually Compound).
    // Let's verify.
    assert_eq!(val_c, Value::String("VariantC".into()));
}
