#[cfg(test)]
mod test {
    use pico_nbt::prelude::Nbt;
    use serde_json::json;

    macro_rules! nbt_test {
        ($($name:ident : $input:expr => $expected:expr ;)*) => {
            $(
                #[test]
                fn $name() {
                    let result = Nbt::from_json(&$input, None);
                    assert_eq!(result, $expected);
                }
            )*
        };
    }

    nbt_test! {
        string:        json!("Hello, World!")  => Nbt::String    { name: None, value: "Hello, World!".to_string() };
        bool_true:     json!(true)             => Nbt::Byte      { name: None, value: 1 };
        bool_false:    json!(false)            => Nbt::Byte      { name: None, value: 0 };
        zero:          json!(0)                => Nbt::Byte      { name: None, value: 0 };
        byte_overflow: json!(1.27e2)           => Nbt::Byte      { name: None, value: 127 };
        short:         json!(128)              => Nbt::Short     { name: None, value: 128 };
        int:           json!(12345678.0)       => Nbt::Int       { name: None, value: 12345678 };
        long:          json!(2147483649_u64)   => Nbt::Long      { name: None, value: 2147483649 };
        float_half:    json!(0.5)              => Nbt::Float     { name: None, value: 0.5 };
        float_precise: json!(31.75)            => Nbt::Float     { name: None, value: 31.75 };
        byte_array:    json!([0, 127])         => Nbt::ByteArray { name: None, value: vec![0, 127] };
        int_array:     json!([12345678.0])     => Nbt::IntArray  { name: None, value: vec![12345678] };
        long_array:    json!([2147483649_u64]) => Nbt::LongArray { name: None, value: vec![2147483649] };
        object:        json!({
            "code": 200,
            "success": true,
            "payload": {
                "features": [
                    "serde",
                    "json"
                ],
                "homepage": "/"
            }
        }) => get_object_expected();
    }

    fn get_object_expected() -> Nbt {
        Nbt::Compound {
            name: None,
            value: vec![
                Nbt::Short {
                    name: Some("code".to_string()),
                    value: 200,
                },
                Nbt::Compound {
                    name: Some("payload".to_string()),
                    value: vec![
                        Nbt::List {
                            name: Some("features".to_string()),
                            value: vec![
                                Nbt::String {
                                    name: None,
                                    value: "serde".to_string(),
                                },
                                Nbt::String {
                                    name: None,
                                    value: "json".to_string(),
                                },
                            ],
                            tag_type: 8, // Tag type of strings
                        },
                        Nbt::String {
                            name: Some("homepage".to_string()),
                            value: "/".to_string(),
                        },
                    ],
                },
                Nbt::Byte {
                    name: Some("success".to_string()),
                    value: 1,
                },
            ],
        }
    }
}
