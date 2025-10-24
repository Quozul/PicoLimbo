use crate::nbt_context::NbtContext;
use crate::nbt_version::NbtFeatures;
use pico_binutils::prelude::{BinaryWriter, BinaryWriterError, IntPrefixed, UShortPrefixed};

#[derive(PartialEq, Debug, Clone)]
pub enum Nbt {
    End,
    Byte {
        name: Option<String>,
        value: i8,
    },
    Short {
        name: Option<String>,
        value: i16,
    },
    Int {
        name: Option<String>,
        value: i32,
    },
    Long {
        name: Option<String>,
        value: i64,
    },
    Float {
        name: Option<String>,
        value: f32,
    },
    Double {
        name: Option<String>,
        value: f64,
    },
    ByteArray {
        name: Option<String>,
        value: Vec<i8>,
    },
    String {
        name: Option<String>,
        value: String,
    },
    List {
        name: Option<String>,
        value: Vec<Nbt>,
        tag_type: u8,
    },
    Compound {
        name: Option<String>,
        value: Vec<Nbt>,
    },
    IntArray {
        name: Option<String>,
        value: Vec<i32>,
    },
    LongArray {
        name: Option<String>,
        value: Vec<i64>,
    },
}

impl Nbt {
    const STRING_TAG_TYPE: u8 = 8;
    const COMPOUND_TAG_TYPE: u8 = 10;

    pub fn nameless_compound(value: Vec<Nbt>) -> Nbt {
        Self::Compound { name: None, value }
    }

    pub fn compound(name: impl ToString, value: Vec<Nbt>) -> Nbt {
        Self::Compound {
            name: Some(name.to_string()),
            value,
        }
    }

    pub fn compound_list(name: impl ToString, value: Vec<Nbt>) -> Nbt {
        Self::List {
            name: Some(name.to_string()),
            value,
            tag_type: Self::COMPOUND_TAG_TYPE,
        }
    }

    pub fn string_list(name: impl ToString, value: Vec<String>) -> Nbt {
        Self::List {
            name: Some(name.to_string()),
            value: value
                .iter()
                .map(|s| Self::String {
                    name: None,
                    value: s.clone(),
                })
                .collect(),
            tag_type: Self::STRING_TAG_TYPE,
        }
    }

    pub fn string(name: impl ToString, value: impl ToString) -> Nbt {
        Self::String {
            name: Some(name.to_string()),
            value: value.to_string(),
        }
    }

    pub fn bool(name: impl ToString, value: bool) -> Nbt {
        Self::Byte {
            name: Some(name.to_string()),
            value: if value { 1 } else { 0 },
        }
    }

    pub fn int(name: impl ToString, value: i32) -> Nbt {
        Self::Int {
            name: Some(name.to_string()),
            value,
        }
    }

    pub fn byte(name: impl ToString, value: i8) -> Nbt {
        Self::Byte {
            name: Some(name.to_string()),
            value,
        }
    }

    pub fn to_bytes(&self, nbt_features: NbtFeatures) -> Result<Vec<u8>, BinaryWriterError> {
        let mut writer = BinaryWriter::default();
        let context = NbtContext::root();
        self.to_bytes_tag(&mut writer, context, nbt_features)?;
        Ok(writer.into_inner())
    }

    pub fn find_tag(&self, name: impl ToString) -> Option<&Nbt> {
        let name = name.to_string();
        match self {
            Self::Compound { value, .. } => value
                .iter()
                .find(|v| v.get_name().is_some_and(|v| v == name)),
            _ => None,
        }
    }

    pub fn get_nbt_vec(&self) -> Option<Vec<Nbt>> {
        match self {
            Self::Compound { value, .. } => Some(value.clone()),
            Self::List { value, .. } => Some(value.clone()),
            _ => None,
        }
    }

    pub fn get_byte_array(&self) -> Option<Vec<i8>> {
        match self {
            Self::ByteArray { value, .. } => Some(value.clone()),
            _ => None,
        }
    }

    pub fn get_short(&self) -> Option<i16> {
        match self {
            Self::Short { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub fn get_int_array(&self) -> Option<Vec<i32>> {
        match self {
            Self::IntArray { value, .. } => Some(value.clone()),
            _ => None,
        }
    }

    pub fn get_int(&self) -> Option<i32> {
        match self {
            Nbt::Int { value, .. } => Some(*value),
            _ => None,
        }
    }

    pub fn get_string(&self) -> Option<String> {
        match self {
            Nbt::String { value, .. } => Some(value.clone()),
            _ => None,
        }
    }

    pub fn get_bool(&self) -> Option<bool> {
        match self {
            Nbt::Byte { value, .. } => Some(*value == 1),
            _ => None,
        }
    }

    pub fn set_name(&self, name: String) -> Nbt {
        match self {
            Nbt::Compound { value, .. } => Nbt::Compound {
                name: Some(name),
                value: value.clone(),
            },
            _ => panic!("Cannot set name on non-compound"),
        }
    }

    pub(crate) fn get_tag_type(&self) -> u8 {
        match self {
            Nbt::End => 0,
            Nbt::Byte { .. } => 1,
            Nbt::Short { .. } => 2,
            Nbt::Int { .. } => 3,
            Nbt::Long { .. } => 4,
            Nbt::Float { .. } => 5,
            Nbt::Double { .. } => 6,
            Nbt::ByteArray { .. } => 7,
            Nbt::String { .. } => Self::STRING_TAG_TYPE,
            Nbt::List { .. } => 9,
            Nbt::Compound { .. } => Self::COMPOUND_TAG_TYPE,
            Nbt::IntArray { .. } => 11,
            Nbt::LongArray { .. } => 12,
        }
    }

    pub fn get_name(&self) -> Option<String> {
        match self {
            Nbt::End => None,
            Nbt::Byte { name, .. } => name.clone(),
            Nbt::Short { name, .. } => name.clone(),
            Nbt::Int { name, .. } => name.clone(),
            Nbt::Long { name, .. } => name.clone(),
            Nbt::Float { name, .. } => name.clone(),
            Nbt::Double { name, .. } => name.clone(),
            Nbt::ByteArray { name, .. } => name.clone(),
            Nbt::String { name, .. } => name.clone(),
            Nbt::List { name, .. } => name.clone(),
            Nbt::Compound { name, .. } => name.clone(),
            Nbt::IntArray { name, .. } => name.clone(),
            Nbt::LongArray { name, .. } => name.clone(),
        }
    }

    fn has_name(&self) -> bool {
        !matches!(self, Nbt::End)
    }

    fn to_bytes_tag(
        &self,
        writer: &mut BinaryWriter,
        context: NbtContext,
        nbt_features: NbtFeatures,
    ) -> Result<(), BinaryWriterError> {
        if context.should_include_tag_type() {
            writer.write(&self.get_tag_type())?;
        };

        if context.should_include_tag_name(nbt_features) && self.has_name() {
            match self.get_name() {
                None => {
                    writer.write(&0_u8)?;
                    writer.write(&0_u8)?;
                }
                Some(name) => {
                    writer.write(&UShortPrefixed::string(name))?;
                }
            }
        }

        match self {
            Nbt::End => {}
            Nbt::Byte { value, .. } => {
                writer.write(value)?;
            }
            Nbt::Short { value, .. } => {
                writer.write(value)?;
            }
            Nbt::Int { value, .. } => {
                writer.write(value)?;
            }
            Nbt::Long { value, .. } => {
                writer.write(value)?;
            }
            Nbt::Float { value, .. } => {
                writer.write(value)?;
            }
            Nbt::Double { value, .. } => {
                writer.write(value)?;
            }
            Nbt::ByteArray { value, .. } => {
                writer.write(&IntPrefixed::new(value))?;
            }
            Nbt::String { value, .. } => {
                writer.write(&UShortPrefixed::string(value))?;
            }
            Nbt::List {
                value, tag_type, ..
            } => {
                let is_same_type = is_vec_of_same_type(value);
                let is_dynamic_lists_enabled =
                    !is_same_type && nbt_features.is_dynamic_lists_available();

                // Write the type of the list
                if is_dynamic_lists_enabled {
                    writer.write(&Self::COMPOUND_TAG_TYPE)?;
                } else {
                    writer.write(tag_type)?;
                };

                // Write the length of the list
                writer.write(&(value.len() as i32))?;

                // Write each tag in the list
                for next_tag in value {
                    if is_dynamic_lists_enabled {
                        let is_compound = next_tag.get_tag_type() == Self::COMPOUND_TAG_TYPE;
                        if is_compound {
                            next_tag.to_bytes_tag(writer, NbtContext::list(), nbt_features)?;
                        } else {
                            let compound_tag = Nbt::Compound {
                                name: None,
                                value: vec![next_tag.clone()],
                            };
                            compound_tag.to_bytes_tag(writer, NbtContext::list(), nbt_features)?;
                        }
                    } else {
                        next_tag.to_bytes_tag(writer, NbtContext::list(), nbt_features)?;
                    }
                }
            }
            Nbt::Compound { value, .. } => {
                for next_tag in value {
                    next_tag.to_bytes_tag(writer, NbtContext::default(), nbt_features)?;
                }
                Nbt::End.to_bytes_tag(writer, NbtContext::default(), nbt_features)?;
            }
            Nbt::IntArray { value, .. } => {
                writer.write(&IntPrefixed::new(value))?;
            }
            Nbt::LongArray { value, .. } => {
                writer.write(&IntPrefixed::new(value))?;
            }
        };

        Ok(())
    }

    pub fn type_string(&self) -> &'static str {
        match self {
            Nbt::End => "End",
            Nbt::Byte { .. } => "Byte",
            Nbt::Short { .. } => "Short",
            Nbt::Int { .. } => "Int",
            Nbt::Long { .. } => "Long",
            Nbt::Float { .. } => "Float",
            Nbt::Double { .. } => "Double",
            Nbt::ByteArray { .. } => "ByteArray",
            Nbt::String { .. } => "String",
            Nbt::List { .. } => "List",
            Nbt::Compound { .. } => "Compound",
            Nbt::IntArray { .. } => "IntArray",
            Nbt::LongArray { .. } => "LongArray",
        }
    }
}

fn is_vec_of_same_type(value: &Vec<Nbt>) -> bool {
    let mut prev: Option<u8> = None;
    for tag in value {
        let curr = tag.get_tag_type();
        if let Some(p) = prev
            && p != curr
        {
            return false;
        }
        prev = Some(curr);
    }
    true
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nbt_root_compound_to_bytes() {
        let nbt = Nbt::Compound {
            value: vec![],
            name: None,
        };
        assert_eq!(
            nbt.to_bytes(NbtFeatures::builder().nameless().build())
                .unwrap(),
            vec![
                0x0a, // Tag type of compound
                0x00, // End tag
            ]
        );
    }

    #[test]
    fn test_nbt_nameless_compound_to_bytes() {
        let nbt = Nbt::Compound {
            name: None,
            value: vec![],
        };
        assert_eq!(
            nbt.to_bytes(NbtFeatures::default()).unwrap(),
            vec![
                0x0a, // Tag type of compound
                0x00, 0x00, // Tag name length of 0
                0x00, // End tag
            ]
        );
    }

    #[test]
    fn test_nbt_named_compound_to_bytes() {
        let nbt = Nbt::Compound {
            name: Some("hi".to_string()),
            value: vec![],
        };
        assert_eq!(
            nbt.to_bytes(NbtFeatures::default()).unwrap(),
            vec![
                0x0a, // Tag type of compound
                0x00, 0x02, // Tag name length of 2
                0x68, 0x69, // Tag name
                0x00, // End tag
            ]
        );
    }

    #[test]
    fn test_nbt_list_single_type() {
        // Given
        let nbt = Nbt::List {
            name: None,
            value: vec![
                Nbt::Int {
                    name: Some("test".to_string()), // The name is not encoded
                    value: 123,
                },
                Nbt::Int {
                    name: Some("test".to_string()),
                    value: 42,
                },
            ],
            tag_type: 3, // 3 is the tag type of Int
        };
        let expected = vec![
            9, // List own tag type
            0, 0, // List name length
            3, // Content tag type
            0, 0, 0, 2, // List length
            0, 0, 0, 123, // First element
            0, 0, 0, 42, // Second element
        ];

        // When
        let serialized = nbt.to_bytes(NbtFeatures::default()).unwrap();

        // Then
        assert_eq!(serialized, expected);
    }

    mod dynamic_lists {
        use crate::nbt::Nbt;
        use crate::nbt_version::NbtFeatures;

        #[test]
        fn test_nbt_list_heterogenous_type() {
            // Given
            let features = NbtFeatures::builder().dynamic_lists().build();
            let nbt = Nbt::List {
                name: None,
                value: vec![
                    Nbt::Int {
                        name: Some("test".to_string()), // Name is not encoded
                        value: 123,
                    },
                    Nbt::Short {
                        name: Some("test".to_string()), // Name is not encoded
                        value: 42,
                    },
                ],
                tag_type: 3, // This is ignored in this case
            };
            let expected = vec![
                9, // List own tag type
                0, 0,  // List name length
                10, // Content tag type
                0, 0, 0, 2, // List length
                // First element
                3, // Compound tag type
                0, 4, // Length of the name
                116, 101, 115, 116, // Name of the compound tag
                0, 0, 0, 123, // Compound value
                0,   // End tag type
                // Second element
                2, // Compound tag type
                0, 4, // Length of the name
                116, 101, 115, 116, // Name of the compound tag
                0, 42, // Compound value
                0,  // End tag type
            ];

            // When
            let serialized = nbt.to_bytes(features).unwrap();

            // Then
            assert_eq!(serialized, expected);
        }

        #[test]
        fn test_nbt_list_same_type_but_feature_enabled() {
            // Given
            let features = NbtFeatures::builder().dynamic_lists().build();
            let nbt = Nbt::List {
                name: None,
                value: vec![
                    Nbt::Int {
                        name: Some("foo".to_string()), // Name is not encoded
                        value: 123,
                    },
                    Nbt::Int {
                        name: Some("bar".to_string()), // Name is not encoded
                        value: 42,
                    },
                ],
                tag_type: 3, // This is ignored in this case
            };
            let expected = vec![
                9, // List own tag type
                0, 0, // List name length
                3, // Content tag type
                0, 0, 0, 2, // List length
                // First element
                0, 0, 0, 123, // Tag value
                // Second element
                0, 0, 0, 42, // Tag value
            ];

            // When
            let serialized = nbt.to_bytes(features).unwrap();

            // Then
            assert_eq!(serialized, expected);
        }
    }

    mod is_vec_of_same_type {
        use crate::nbt::{Nbt, is_vec_of_same_type};

        #[test]
        fn test_is_vec_of_same_type() {
            // Given
            let vec = vec![
                Nbt::Int {
                    name: Some("foo".to_string()),
                    value: 123,
                },
                Nbt::Int {
                    name: Some("bar".to_string()),
                    value: 42,
                },
            ];

            // When
            let result = is_vec_of_same_type(&vec);

            // Then
            assert!(result);
        }

        #[test]
        fn test_is_vec_not_of_same_type() {
            // Given
            let vec = vec![
                Nbt::Int {
                    name: Some("foo".to_string()),
                    value: 123,
                },
                Nbt::Short {
                    name: Some("bar".to_string()),
                    value: 42,
                },
            ];

            // When
            let result = is_vec_of_same_type(&vec);

            // Then
            assert!(!result);
        }
    }
}
