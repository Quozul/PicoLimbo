use pico_nbt2::{NbtOptions, Value, from_reader_with_options, to_writer_with_options};
use serde::Serialize;
use std::io::Cursor;

#[test]
fn test_nameless_root() {
    #[derive(Serialize)]
    struct MyStruct {
        val: i32,
    }
    let s = MyStruct { val: 123 };
    let options = NbtOptions::new().nameless_root(true).dynamic_lists(false);

    let mut buf = Vec::new();
    to_writer_with_options(&mut buf, &s, Some("ignored"), options).unwrap();

    // Verify bytes: Tag(10) + [No Name] + Content
    // Normal: Tag(10) + NameLen(u16) + Name + Content
    // Nameless: Tag(10) + Content
    assert_eq!(buf[0], 10); // Tag Compound
    // Next should be content of compound.
    // Compound content: Tag(3) + NameLen("val") + "val" + Int(123) + Tag(0)
    // "val" len is 3.
    // So: 3 (Tag Int), 0, 3 (Name Len), 'v', 'a', 'l', 0, 0, 0, 123 (Int), 0 (End)

    // If it had name "ignored":
    // 10, 0, 7, 'i', 'g', 'n', 'o', 'r', 'e', 'd', ...

    // Let's check index 1.
    // If nameless, index 1 is start of content (Tag Int = 3).
    assert_eq!(buf[1], 3);

    // Roundtrip
    let mut cursor = Cursor::new(buf);
    let (name, value) = from_reader_with_options(&mut cursor, options).unwrap();
    assert_eq!(name, ""); // Name should be empty
    if let Value::Compound(map) = value {
        assert_eq!(map.get("val"), Some(&Value::Int(123)));
    } else {
        panic!("Expected Compound");
    }
}

#[test]
fn test_dynamic_lists_enabled() {
    #[derive(Serialize)]
    struct Hetero {
        list: Vec<Value>,
    }

    let s = Hetero {
        list: vec![Value::Int(1), Value::String("test".into())],
    };

    let options = NbtOptions::new().nameless_root(false).dynamic_lists(true);

    let mut buf = Vec::new();
    to_writer_with_options(&mut buf, &s, Some("root"), options).unwrap();

    // Verify structure
    // Root Compound -> "list" -> List Tag
    // List Tag ID should be 9 (List)
    // Element Type should be 10 (Compound) because it's heterogenous

    let (name, value) = from_reader_with_options(&mut Cursor::new(buf), options).unwrap();
    assert_eq!(name, "root");
    let Value::Compound(root) = value else {
        panic!("Expected Root Compound")
    };
    let Value::List(list) = root.get("list").unwrap() else {
        panic!("Expected List")
    };

    assert_eq!(list.len(), 2);
    // Elements should be Compounds wrapping the values
    match &list[0] {
        Value::Compound(c) => {
            assert_eq!(c.len(), 1);
            assert_eq!(c.get(""), Some(&Value::Int(1)));
        }
        _ => panic!("Expected Compound wrapper 1"),
    }
    match &list[1] {
        Value::Compound(c) => {
            assert_eq!(c.len(), 1);
            assert_eq!(c.get(""), Some(&Value::String("test".into())));
        }
        _ => panic!("Expected Compound wrapper 2"),
    }
}

#[test]
fn test_dynamic_lists_disabled_error() {
    #[derive(Serialize)]
    struct Hetero {
        list: Vec<Value>,
    }

    let s = Hetero {
        list: vec![Value::Int(1), Value::String("test".into())],
    };

    let options = NbtOptions::new().nameless_root(false).dynamic_lists(false);

    let mut buf = Vec::new();
    let res = to_writer_with_options(&mut buf, &s, Some("root"), options);
    assert!(res.is_err());
    assert!(
        res.unwrap_err()
            .to_string()
            .contains("Heterogeneous lists are not supported")
    );
}
