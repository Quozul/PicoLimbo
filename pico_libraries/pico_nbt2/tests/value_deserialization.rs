use pico_nbt2::Value;
use serde::Deserialize;

#[derive(Debug, PartialEq, Deserialize)]
struct TestStruct {
    name: String,
    id: i32,
    active: bool,
}

#[test]
fn test_struct_from_value() {
    let mut map = indexmap::IndexMap::new();
    map.insert("name".into(), Value::String("foo".into()));
    map.insert("id".into(), Value::Int(123));
    map.insert("active".into(), Value::Byte(1));
    let value = Value::Compound(map);

    // This function will be implemented
    let obj: TestStruct = pico_nbt2::from_value(value).unwrap();

    assert_eq!(
        obj,
        TestStruct {
            name: "foo".into(),
            id: 123,
            active: true,
        }
    );
}

#[test]
fn test_vec_from_list() {
    let list = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);

    let vec: Vec<i32> = pico_nbt2::from_value(list).unwrap();
    assert_eq!(vec, vec![1, 2, 3]);
}

#[test]
fn test_map_from_compound() {
    let mut map = indexmap::IndexMap::new();
    map.insert("key".into(), Value::String("value".into()));
    let value = Value::Compound(map);

    let map: std::collections::HashMap<String, String> = pico_nbt2::from_value(value).unwrap();
    assert_eq!(map.get("key"), Some(&"value".to_string()));
}
