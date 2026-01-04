use pico_nbt2::to_bytes;
use serde::Serialize;

#[test]
fn test_struct_serialization_fail() {
    #[derive(Serialize)]
    struct MyStruct {
        foo: i32,
        bar: String,
    }

    let s = MyStruct {
        foo: 123,
        bar: "hello".to_string(),
    };

    // This currently fails to compile or run because to_bytes takes &Value, not T: Serialize
    // But if we try to use it via some wrapper or if we change to_bytes signature, it would fail.
    // For now, we can't even call it.
    // So this test is just a placeholder to show what we WANT to do.

    let bytes = to_bytes(&s, Some("root")).expect("Failed to serialize struct");

    // Verify content
    let (name, value) = pico_nbt2::from_slice(&bytes).expect("Failed to deserialize");
    assert_eq!(name, "root");
    if let pico_nbt2::Value::Compound(map) = value {
        assert_eq!(map.get("foo"), Some(&pico_nbt2::Value::Int(123)));
        assert_eq!(
            map.get("bar"),
            Some(&pico_nbt2::Value::String("hello".into()))
        );
    } else {
        panic!("Expected compound");
    }
}
