use crate::nbt_version::NbtFeatures;
use crate::writers::{
    size_to_i32_bytes, write_array_i8, write_array_i32, write_array_i64, write_string,
};

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
    /// Only for network Nbt for versions 1.20.2 and above
    NamelessCompound {
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
    pub fn to_bytes(&self, nbt_features: NbtFeatures) -> Vec<u8> {
        self.to_bytes_tag(false, false, nbt_features)
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

    pub fn get_vec(&self) -> Option<Vec<Nbt>> {
        match self {
            Self::Compound { value, .. } => Some(value.clone()),
            Self::NamelessCompound { value, .. } => Some(value.clone()),
            Self::List { value, .. } => Some(value.clone()),
            _ => None,
        }
    }

    pub fn get_tag_type(&self) -> u8 {
        match self {
            Nbt::End => 0,
            Nbt::Byte { .. } => 1,
            Nbt::Short { .. } => 2,
            Nbt::Int { .. } => 3,
            Nbt::Long { .. } => 4,
            Nbt::Float { .. } => 5,
            Nbt::Double { .. } => 6,
            Nbt::ByteArray { .. } => 7,
            Nbt::String { .. } => 8,
            Nbt::List { .. } => 9,
            Nbt::Compound { .. } => 10,
            Nbt::NamelessCompound { .. } => 10,
            Nbt::IntArray { .. } => 11,
            Nbt::LongArray { .. } => 12,
        }
    }

    fn has_name(&self) -> bool {
        !matches!(self, Nbt::NamelessCompound { .. } | Nbt::End { .. })
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
            Nbt::NamelessCompound { .. } => None,
            Nbt::IntArray { name, .. } => name.clone(),
            Nbt::LongArray { name, .. } => name.clone(),
        }
    }

    fn serialize_name(&self) -> Vec<u8> {
        match self.get_name() {
            None => Vec::from([0, 0]),
            Some(name) => write_string(name),
        }
    }

    fn to_bytes_tag(
        &self,
        skip_name: bool,
        skip_tag_type: bool,
        nbt_features: NbtFeatures,
    ) -> Vec<u8> {
        let tag_type = self.get_tag_type();
        let mut base = if skip_tag_type {
            Vec::new()
        } else {
            Vec::from([tag_type])
        };

        if !skip_name && self.has_name() {
            base.extend(self.serialize_name());
        }

        match self {
            Nbt::End => {}
            Nbt::Byte { value, .. } => {
                base.extend(value.to_be_bytes());
            }
            Nbt::Short { value, .. } => {
                base.extend(value.to_be_bytes());
            }
            Nbt::Int { value, .. } => {
                base.extend(value.to_be_bytes());
            }
            Nbt::Long { value, .. } => {
                base.extend(value.to_be_bytes());
            }
            Nbt::Float { value, .. } => {
                base.extend(value.to_be_bytes());
            }
            Nbt::Double { value, .. } => {
                base.extend(value.to_be_bytes());
            }
            Nbt::ByteArray { value, .. } => {
                base.extend(write_array_i8(value));
            }
            Nbt::String { value, .. } => {
                base.extend(write_string(value.clone()));
            }
            Nbt::List {
                value, tag_type, ..
            } => {
                let mut serialized_value: Vec<u8> = if nbt_features.is_dynamic_lists_available() {
                    Vec::from([10])
                } else {
                    Vec::from([*tag_type])
                };
                let size_bytes = size_to_i32_bytes(value.len());
                serialized_value.extend_from_slice(&size_bytes);
                for next_tag in value {
                    if nbt_features.is_dynamic_lists_available() {
                        let next_tag_type = next_tag.get_tag_type();
                        let is_compound = next_tag_type == 10;
                        if is_compound {
                            serialized_value.extend(next_tag.to_bytes_tag(
                                true,
                                true,
                                nbt_features,
                            ));
                        } else {
                            let compound_tag = Nbt::Compound {
                                name: None,
                                value: vec![next_tag.clone()],
                            };
                            serialized_value.extend(compound_tag.to_bytes_tag(
                                true,
                                true,
                                nbt_features,
                            ));
                        }
                    } else {
                        serialized_value.extend(next_tag.to_bytes_tag(true, true, nbt_features));
                    }
                }
                base.extend(serialized_value);
            }
            Nbt::Compound { value, .. } => {
                let mut serialized_value: Vec<u8> = Vec::new();
                for next_tag in value {
                    serialized_value.extend(next_tag.to_bytes_tag(false, false, nbt_features));
                }
                serialized_value.extend(Nbt::End.to_bytes_tag(true, false, nbt_features));
                base.extend(serialized_value);
            }
            Nbt::NamelessCompound { value } => {
                let mut serialized_value: Vec<u8> = Vec::new();
                for next_tag in value {
                    serialized_value.extend(next_tag.to_bytes_tag(false, false, nbt_features));
                }
                serialized_value.extend(Nbt::End.to_bytes_tag(true, false, nbt_features));
                base.extend(serialized_value);
            }
            Nbt::IntArray { value, .. } => {
                base.extend(write_array_i32(value));
            }
            Nbt::LongArray { value, .. } => {
                base.extend(write_array_i64(value));
            }
        };

        base
    }

    pub fn to_nameless_compound(&self) -> Nbt {
        match self {
            Nbt::Compound { value, .. } => Nbt::NamelessCompound {
                value: value.clone(),
            },
            _ => panic!("Cannot convert non-compound Nbt to nameless compound"),
        }
    }

    pub fn to_named_compound(&self, name: String) -> Nbt {
        match self {
            Nbt::NamelessCompound { value, .. } => Nbt::Compound {
                name: Some(name),
                value: value.clone(),
            },
            Nbt::Compound { value, .. } => Nbt::Compound {
                name: Some(name),
                value: value.clone(),
            },
            _ => panic!("Cannot convert non-compound Nbt to nameless compound"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nbt_root_compound_to_bytes() {
        let nbt = Nbt::NamelessCompound { value: vec![] };
        assert_eq!(
            nbt.to_bytes(NbtFeatures::default()),
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
            nbt.to_bytes(NbtFeatures::default()),
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
            nbt.to_bytes(NbtFeatures::default()),
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
        let serialized = nbt.to_bytes(NbtFeatures::default());

        // Then
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_nbt_list_heterogenous_type() {
        // Given
        let features = NbtFeatures::builder().dynamic_lists().build();
        let nbt = Nbt::List {
            name: None,
            value: vec![
                Nbt::Int {
                    name: Some("test".to_string()),
                    value: 123,
                },
                Nbt::Short {
                    name: Some("test".to_string()),
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
        let serialized = nbt.to_bytes(features);

        // Then
        assert_eq!(serialized, expected);
    }
}
