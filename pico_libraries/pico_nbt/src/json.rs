use crate::{Error, Result, Value};
use serde_json::Value as JsonValue;

fn convert_array(arr: Vec<JsonValue>, options: crate::NbtOptions) -> Result<Value> {
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

    // When numeric_widening is enabled, never emit ByteArray for integer arrays —
    // Mojang/NumberOps emits IntArray (or LongArray on overflow). ByteArray remains
    // possible only via the default (non-widening) code path.
    if is_byte && !options.is_numeric_widening() {
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
        list.push(json_to_nbt_with_options(elem, options)?);
    }
    Ok(Value::List(list))
}

/// Converts a JSON value to an NBT value using default options.
///
/// # Errors
/// Returns an error if:
/// * The JSON contains a `null` value, which is not supported in NBT.
/// * A number is invalid or cannot be represented in NBT types.
/// * Array elements cannot be converted to NBT values.
pub fn json_to_nbt(json: JsonValue) -> Result<Value> {
    json_to_nbt_with_options(json, crate::NbtOptions::new())
}

/// Converts a JSON value to an NBT value with caller-supplied options.
///
/// When `options.is_numeric_widening()` is true, integer JSON numbers are encoded
/// as `Value::Int` if they fit in `i32`, `Value::Long` otherwise — matching
/// Mojang's canonical NBT representation. Floating-point numbers are encoded as
/// `Value::Float` (32-bit) always, even when precision is lost. Booleans
/// (`JsonValue::Bool`) remain `Value::Byte(0/1)`. With the flag off, behavior is
/// the legacy smallest-fitting downcast (`Byte` / `Short` / `Int` / `Long`,
/// `Float` only when exact, `Double` otherwise).
///
/// # Errors
/// Returns an error if:
/// * The JSON contains a `null` value, which is not supported in NBT.
/// * A number is invalid or cannot be represented in NBT types.
/// * Array elements cannot be converted to NBT values.
pub fn json_to_nbt_with_options(json: JsonValue, options: crate::NbtOptions) -> Result<Value> {
    match json {
        JsonValue::Null => Err(Error::Message("JSON null is not supported in NBT".into())),
        JsonValue::Bool(b) => Ok(Value::Byte(i8::from(b))),
        JsonValue::Number(n) => convert_number(&n, options),
        JsonValue::String(s) => Ok(Value::String(s)),
        JsonValue::Array(arr) => convert_array(arr, options),
        JsonValue::Object(obj) => {
            let mut map = indexmap::IndexMap::new();
            for (k, v) in obj {
                map.insert(k, json_to_nbt_with_options(v, options)?);
            }
            Ok(Value::Compound(map))
        }
    }
}

fn convert_number(n: &serde_json::Number, options: crate::NbtOptions) -> Result<Value> {
    if let Some(i) = n.as_i64() {
        if options.is_numeric_widening() {
            return Ok(i32::try_from(i).map_or(Value::Long(i), Value::Int));
        }
        // Legacy: smallest-fitting type.
        return Ok(i8::try_from(i).map_or_else(
            |_| {
                i16::try_from(i).map_or_else(
                    |_| i32::try_from(i).map_or(Value::Long(i), Value::Int),
                    Value::Short,
                )
            },
            Value::Byte,
        ));
    }
    if let Some(f) = n.as_f64() {
        #[allow(clippy::cast_possible_truncation)]
        let f32_val = f as f32;
        if options.is_numeric_widening() {
            // Mojang's registry codecs expect Float strictly, even at the cost of
            // precision (e.g. cubic_bezier ease curves in minecraft:timeline).
            return Ok(Value::Float(f32_val));
        }
        if (f64::from(f32_val) - f).abs() < f64::EPSILON {
            return Ok(Value::Float(f32_val));
        }
        return Ok(Value::Double(f));
    }
    Err(Error::Message("Invalid JSON number".into()))
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

    #[test]
    fn widening_zero_becomes_int() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(0), opts).unwrap(),
            Value::Int(0),
        );
    }

    #[test]
    fn widening_short_range_becomes_int() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(1_000), opts).unwrap(),
            Value::Int(1_000),
        );
    }

    #[test]
    fn widening_int_max_becomes_int() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(i32::MAX), opts).unwrap(),
            Value::Int(i32::MAX),
        );
    }

    #[test]
    fn widening_overflow_becomes_long() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        let big: i64 = i64::from(i32::MAX) + 1;
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(big), opts).unwrap(),
            Value::Long(big),
        );
    }

    #[test]
    fn widening_negative_short_range_becomes_int() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(-1_000), opts).unwrap(),
            Value::Int(-1_000),
        );
    }

    #[test]
    fn widening_bool_stays_byte() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(true), opts).unwrap(),
            Value::Byte(1),
        );
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(false), opts).unwrap(),
            Value::Byte(0),
        );
    }

    #[test]
    fn widening_float_unchanged_when_exact() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(1.0_f64), opts).unwrap(),
            Value::Float(1.0),
        );
    }

    #[test]
    fn widening_imprecise_double_collapses_to_float() {
        // 0.362 cannot round-trip exactly between f64 and f32. Without widening,
        // the legacy code keeps it as Double to preserve precision. With widening
        // enabled, it must collapse to Float — Mojang's registry codecs expect
        // Float strictly (e.g. cubic_bezier values in minecraft:timeline).
        let opts = crate::NbtOptions::new().numeric_widening(true);
        let result =
            crate::json::json_to_nbt_with_options(json!(0.362_f64), opts).unwrap();
        let Value::Float(f) = result else {
            panic!("expected Float under widening, got {result:?}");
        };
        assert!(
            (f - 0.362_f32).abs() < f32::EPSILON,
            "expected ~0.362 as Float, got {f}",
        );
    }

    #[test]
    fn default_options_keep_downcasting() {
        let opts = crate::NbtOptions::new();
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(0), opts).unwrap(),
            Value::Byte(0),
        );
        assert_eq!(
            crate::json::json_to_nbt_with_options(json!(128), opts).unwrap(),
            Value::Short(128),
        );
    }

    #[test]
    fn default_options_keep_double_when_imprecise() {
        let opts = crate::NbtOptions::new();
        let result =
            crate::json::json_to_nbt_with_options(json!(0.362_f64), opts).unwrap();
        assert!(
            matches!(result, Value::Double(d) if (d - 0.362_f64).abs() < f64::EPSILON),
            "expected Double under default options, got {result:?}",
        );
    }

    #[test]
    fn nested_compound_widens_recursively() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        let json_obj = json!({"ticks": 18000, "show_in_commands": true});
        let nbt = crate::json::json_to_nbt_with_options(json_obj, opts).unwrap();
        let Value::Compound(map) = nbt else {
            panic!("Expected Compound");
        };
        assert_eq!(map.get("ticks"), Some(&Value::Int(18000)));
        assert_eq!(map.get("show_in_commands"), Some(&Value::Byte(1)));
    }

    #[test]
    fn widening_byte_range_array_becomes_int_array() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        let nbt = crate::json::json_to_nbt_with_options(json!([1, 2, 3]), opts).unwrap();
        assert_eq!(nbt, Value::IntArray(vec![1, 2, 3]));
    }

    #[test]
    fn widening_short_range_array_becomes_int_array() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        let nbt =
            crate::json::json_to_nbt_with_options(json!([1000, 2000, 3000]), opts).unwrap();
        assert_eq!(nbt, Value::IntArray(vec![1000, 2000, 3000]));
    }

    #[test]
    fn widening_overflow_array_becomes_long_array() {
        let opts = crate::NbtOptions::new().numeric_widening(true);
        let big: i64 = i64::from(i32::MAX) + 1;
        let nbt = crate::json::json_to_nbt_with_options(json!([big, big]), opts).unwrap();
        assert_eq!(nbt, Value::LongArray(vec![big, big]));
    }

    #[test]
    fn default_byte_range_array_stays_byte_array() {
        let opts = crate::NbtOptions::new();
        let nbt = crate::json::json_to_nbt_with_options(json!([1, 2, 3]), opts).unwrap();
        let Value::ByteArray(bytes) = nbt else {
            panic!("expected ByteArray with default options, got {nbt:?}");
        };
        assert_eq!(bytes, vec![1, 2, 3]);
    }
}
