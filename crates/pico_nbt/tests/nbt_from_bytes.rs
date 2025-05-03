#[cfg(test)]
mod tests {
    use nbt::prelude::{BinaryReader, Nbt, parse_tag};
    const HELLO_WORLD_NBT: &[u8] = include_bytes!("../test_files/hello_world.nbt");
    const BIG_TEST_NBT: &[u8] = include_bytes!("../test_files/bigtest.nbt");

    #[test]
    fn test_parse_hello_world() {
        // Given
        let mut reader = BinaryReader::new(HELLO_WORLD_NBT);

        // When
        let result = parse_tag(&mut reader);

        // Then
        assert_eq!(
            result,
            Nbt::Compound {
                name: Some(String::from("hello world")),
                value: Vec::from([Nbt::String {
                    name: Some(String::from("name")),
                    value: String::from("Bananrama"),
                }])
            }
        );
    }

    #[test]
    fn test_encode_hello_world() {
        // Given
        let nbt = Nbt::Compound {
            name: Some(String::from("hello world")),
            value: Vec::from([Nbt::String {
                name: Some(String::from("name")),
                value: String::from("Bananrama"),
            }]),
        };

        // When
        let serialized = nbt.to_bytes(false);

        // Then
        assert_eq!(serialized, HELLO_WORLD_NBT);
    }

    #[test]
    fn test_parse_big_test() {
        // Given
        let expected = build_expected_big_test_nbt();
        let mut reader = BinaryReader::new(BIG_TEST_NBT);

        // When
        let result = parse_tag(&mut reader);

        // Then
        assert_eq!(result, expected);
    }

    #[test]
    fn test_encode_big_test() {
        // Given
        let nbt = build_expected_big_test_nbt();

        // When
        let encoded_bytes = nbt.to_bytes(false);

        // Then
        assert_eq!(encoded_bytes, BIG_TEST_NBT);
    }

    fn build_expected_big_test_nbt() -> Nbt {
        let mut value = Vec::new();
        for n in 0..1000 {
            let r = ((n * n * 255 + n * 7) % 100) as i8;
            value.push(r);
        }

        Nbt::Compound {
            name: Some(String::from("Level")),
            value: Vec::from([
                Nbt::Long {
                    name: Some(String::from("longTest")),
                    value: 9223372036854775807,
                },
                Nbt::Short {
                    name: Some(String::from("shortTest")),
                    value: 32767,
                },
                Nbt::String {
                    name: Some(String::from("stringTest")),
                    value: String::from("HELLO WORLD THIS IS A TEST STRING ÅÄÖ!"),
                },
                Nbt::Float {
                    name: Some(String::from("floatTest")),
                    value: 0.498_231_470_584_869_38_f32,
                },
                Nbt::Int {
                    name: Some(String::from("intTest")),
                    value: 2147483647,
                },
                Nbt::Compound {
                    name: Some(String::from("nested compound test")),
                    value: Vec::from([
                        Nbt::Compound {
                            name: Some(String::from("ham")),
                            value: Vec::from([
                                Nbt::String {
                                    name: Some(String::from("name")),
                                    value: String::from("Hampus"),
                                },
                                Nbt::Float {
                                    name: Some(String::from("value")),
                                    value: 0.75,
                                },
                            ]),
                        },
                        Nbt::Compound {
                            name: Some(String::from("egg")),
                            value: Vec::from([
                                Nbt::String {
                                    name: Some(String::from("name")),
                                    value: String::from("Eggbert"),
                                },
                                Nbt::Float {
                                    name: Some(String::from("value")),
                                    value: 0.5,
                                },
                            ]),
                        },
                    ]),
                },
                Nbt::List {
                    name: Some(String::from("listTest (long)")),
                    tag_type: 4,
                    value: Vec::from([
                        Nbt::Long {
                            name: None,
                            value: 11,
                        },
                        Nbt::Long {
                            name: None,
                            value: 12,
                        },
                        Nbt::Long {
                            name: None,
                            value: 13,
                        },
                        Nbt::Long {
                            name: None,
                            value: 14,
                        },
                        Nbt::Long {
                            name: None,
                            value: 15,
                        },
                    ]),
                },
                Nbt::List {
                    name: Some(String::from("listTest (compound)")),
                    tag_type: 10,
                    value: Vec::from([
                        Nbt::Compound {
                            name: None,
                            value: Vec::from([
                                Nbt::String {
                                    name: Some(String::from("name")),
                                    value: String::from("Compound tag #0"),
                                },
                                Nbt::Long {
                                    name: Some(String::from("created-on")),
                                    value: 1264099775885,
                                },
                            ]),
                        },
                        Nbt::Compound {
                            name: None,
                            value: Vec::from([
                                Nbt::String {
                                    name: Some(String::from("name")),
                                    value: String::from("Compound tag #1"),
                                },
                                Nbt::Long {
                                    name: Some(String::from("created-on")),
                                    value: 1264099775885,
                                },
                            ]),
                        },
                    ]),
                },
                Nbt::Byte {
                    name: Some(String::from("byteTest")),
                    value: 127,
                },
                Nbt::ByteArray {
                    name: Some(String::from(
                        "byteArrayTest (the first 1000 values of (n*n*255+n*7)%100, starting with n=0 (0, 62, 34, 16, 8, ...))",
                    )),
                    value,
                },
                Nbt::Double {
                    name: Some(String::from("doubleTest")),
                    value: 0.493_128_713_218_231_48_f64,
                },
            ]),
        }
    }
}
