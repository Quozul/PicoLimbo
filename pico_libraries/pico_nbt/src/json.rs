use crate::{Error, Result, Value};
use serde_json::Value as JsonValue;

/// Converts a JSON value to an NBT value.
///
/// # Errors
/// Returns an error if:
/// * The JSON contains a `null` value, which is not supported in NBT.
/// * A number is invalid or cannot be represented in NBT types.
/// * Array elements cannot be converted to NBT values.
fn convert_array(arr: Vec<JsonValue>) -> Result<Value> {
    if arr.is_empty() {
        return Ok(Value::List(Vec::new()));
    }

    let mut is_byte = true;
    let mut is_int = true;
    let mut is_long = true;

    for elem in &arr {
        if let JsonValue::Number(n) = elem {
            if let Some(i) = n.as_i64() {
                if is_byte && (i < i64::from(i8::MIN) || i > i64::from(i8::MAX)) {
                    is_byte = false;
                }
                if is_int && (i < i64::from(i32::MIN) || i > i64::from(i32::MAX)) {
                    is_int = false;
                }
            } else {
                // Float or something else
                is_byte = false;
                is_int = false;
                is_long = false;
            }
        } else {
            // Not a number
            is_byte = false;
            is_int = false;
            is_long = false;
        }

        if !is_long {
            break;
        }
    }

    if is_byte {
        let mut bytes = Vec::with_capacity(arr.len());
        for elem in arr {
            if let JsonValue::Number(n) = elem {
                let i = n
                    .as_i64()
                    .ok_or_else(|| Error::Message("Expected integer in byte array".into()))?;
                let byte_val = i8::try_from(i)
                    .map_err(|_| Error::Message("Value out of range for byte array".into()))?;
                bytes.push(u8::from_ne_bytes(byte_val.to_ne_bytes()));
            }
        }
        return Ok(Value::ByteArray(bytes));
    } else if is_int {
        let mut ints = Vec::with_capacity(arr.len());
        for elem in arr {
            if let JsonValue::Number(n) = elem {
                let i = n
                    .as_i64()
                    .ok_or_else(|| Error::Message("Expected integer in int array".into()))?;
                let int_val = i32::try_from(i)
                    .map_err(|_| Error::Message("Value out of range for int array".into()))?;
                ints.push(int_val);
            }
        }
        return Ok(Value::IntArray(ints));
    } else if is_long {
        let mut longs = Vec::with_capacity(arr.len());
        for elem in arr {
            if let JsonValue::Number(n) = elem {
                longs.push(
                    n.as_i64()
                        .ok_or_else(|| Error::Message("Expected integer in long array".into()))?,
                );
            }
        }
        return Ok(Value::LongArray(longs));
    }

    let mut list = Vec::with_capacity(arr.len());
    for elem in arr {
        list.push(json_to_nbt(elem)?);
    }
    Ok(Value::List(list))
}

/// Converts a JSON value to an NBT value.
///
/// # Errors
/// Returns an error if:
/// * The JSON contains a `null` value, which is not supported in NBT.
/// * A number is invalid or cannot be represented in NBT types.
/// * Array elements cannot be converted to NBT values.
pub fn json_to_nbt(json: JsonValue) -> Result<Value> {
    match json {
        JsonValue::Null => Err(Error::Message("JSON null is not supported in NBT".into())),
        JsonValue::Bool(b) => Ok(Value::Byte(i8::from(b))),
        JsonValue::Number(n) => n.as_i64().map_or_else(
            || {
                n.as_f64().map_or_else(
                    || Err(Error::Message("Invalid JSON number".into())),
                    |f| {
                        #[allow(clippy::cast_possible_truncation)]
                        // I cannot find a better solution than casting the double down to float
                        let f32_val = f as f32;
                        if (f64::from(f32_val) - f).abs() < f64::EPSILON {
                            Ok(Value::Float(f32_val))
                        } else {
                            Ok(Value::Double(f))
                        }
                    },
                )
            },
            |i| {
                i8::try_from(i).map_or_else(
                    |_| {
                        i16::try_from(i).map_or_else(
                            |_| {
                                i32::try_from(i)
                                    .map_or_else(|_| Ok(Value::Long(i)), |int| Ok(Value::Int(int)))
                            },
                            |s| Ok(Value::Short(s)),
                        )
                    },
                    |s| Ok(Value::Byte(s)),
                )
            },
        ),
        JsonValue::String(s) => Ok(Value::String(s)),
        JsonValue::Array(arr) => convert_array(arr),
        JsonValue::Object(obj) => {
            let mut map = indexmap::IndexMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_nbt(v)?);
            }
            Ok(Value::Compound(map))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use serde_json::json;

    #[test]
    fn bool_true() {
        assert_eq!(json_to_nbt(json!(true)).unwrap(), Value::Byte(1));
    }

    #[test]
    fn bool_false() {
        assert_eq!(json_to_nbt(json!(false)).unwrap(), Value::Byte(0));
    }

    #[test]
    fn zero_byte() {
        assert_eq!(json_to_nbt(json!(0)).unwrap(), Value::Byte(0));
    }

    #[test]
    fn short_value() {
        assert_eq!(json_to_nbt(json!(128)).unwrap(), Value::Short(128));
    }

    #[test]
    fn int_value() {
        assert_eq!(
            json_to_nbt(json!(12_345_678)).unwrap(),
            Value::Int(12_345_678)
        );
    }

    #[test]
    fn long_value() {
        assert_eq!(
            json_to_nbt(json!(2_147_483_649_u64)).unwrap(),
            Value::Long(2_147_483_649)
        );
    }

    #[test]
    fn float_value() {
        assert_eq!(
            json_to_nbt(json!(std::f32::consts::PI)).unwrap(),
            Value::Float(std::f32::consts::PI)
        );
    }

    #[test]
    fn double_value() {
        assert_eq!(
            json_to_nbt(json!(std::f64::consts::PI)).unwrap(),
            Value::Double(std::f64::consts::PI)
        );
    }

    #[test]
    fn string_value() {
        assert_eq!(
            json_to_nbt(json!("hello")).unwrap(),
            Value::String("hello".into())
        );
    }

    #[test]
    fn test_json_object() {
        let json_obj = json!({
            "foo": "bar",
            "baz": 100
        });
        let nbt_compound = json_to_nbt(json_obj).unwrap();
        if let Value::Compound(map) = nbt_compound {
            assert_eq!(map.get("foo"), Some(&Value::String("bar".into())));
            assert_eq!(map.get("baz"), Some(&Value::Byte(100)));
        } else {
            panic!("Expected Compound");
        }
    }

    #[test]
    fn test_json_nested() {
        let json_data = json!({
            "list": [
                { "id": 1 },
                { "id": 2 }
            ]
        });
        let nbt = json_to_nbt(json_data).unwrap();
        // Verify structure
        let Value::Compound(root) = nbt else {
            panic!("Expected Root Compound")
        };
        let Value::List(list) = root.get("list").unwrap() else {
            panic!("Expected List")
        };
        assert_eq!(list.len(), 2);
        let Value::Compound(c1) = &list[0] else {
            panic!("Expected Compound 1")
        };
        assert_eq!(c1.get("id"), Some(&Value::Byte(1)));
    }

    #[test]
    fn test_byte_array_conversion() {
        let byte_arr = json!([1, 2, 3]);
        let nbt = json_to_nbt(byte_arr).unwrap();
        if let Value::ByteArray(arr) = nbt {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], 1);
        } else {
            panic!("Expected ByteArray, got {nbt:?}");
        }
    }

    #[test]
    fn test_int_array_conversion() {
        let int_arr = json!([1000, 2000, 3000]);
        let nbt = json_to_nbt(int_arr).unwrap();
        if let Value::IntArray(arr) = nbt {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], 1000);
        } else {
            panic!("Expected IntArray, got {nbt:?}");
        }
    }

    #[test]
    fn test_long_array_conversion() {
        let long_arr = json!([10_000_000_000_i64, 20_000_000_000_i64]);
        let nbt = json_to_nbt(long_arr).unwrap();
        if let Value::LongArray(arr) = nbt {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], 10_000_000_000);
        } else {
            panic!("Expected LongArray, got {nbt:?}");
        }
    }

    #[test]
    fn test_mixed_content_is_list() {
        let mixed = json!([1, "test"]);
        let nbt = json_to_nbt(mixed).unwrap();
        if let Value::List(arr) = nbt {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::Byte(1));
            assert_eq!(arr[1], Value::String("test".into()));
        } else {
            panic!("Expected List for mixed content");
        }
    }

    #[test]
    fn test_string_list_is_list() {
        let str_list = json!(["foo", "bar"]);
        let nbt = json_to_nbt(str_list).unwrap();
        if let Value::List(arr) = nbt {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], Value::String("foo".into()));
            assert_eq!(arr[1], Value::String("bar".into()));
        } else {
            panic!("Expected List for string list");
        }
    }

    #[test]
    fn test_json_null_error() {
        let res = json_to_nbt(json!(null));
        assert!(res.is_err());
    }
}
