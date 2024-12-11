use crate::nbt::Nbt;
use serde_json::{Number, Value};

impl Nbt {
    pub fn from_json(json: &Value, name: Option<String>) -> Nbt {
        match json {
            Value::Null => {
                panic!("Null value is not supported");
            }
            Value::Bool(b) => {
                let value = if *b { 1 } else { 0 };
                Nbt::Byte { name, value }
            }
            Value::Number(value) => Nbt::number_to_tag(value, name),
            Value::String(value) => Nbt::String {
                name,
                value: value.clone(),
            },
            Value::Array(array) => Nbt::array_to_tag(array, name),
            Value::Object(object) => {
                let mut nbt_compound = Vec::new();
                for (key, value) in object {
                    let nbt = Nbt::from_json(value, Some(key.clone()));
                    nbt_compound.push(nbt);
                }
                Nbt::Compound {
                    name,
                    value: nbt_compound,
                }
            }
        }
    }

    /// - If in the range of byte (e.g. 0, 1.0, 1.27e2), converts to a  byte.
    /// - Otherwise, if in the range of short (e.g. 128, 1234), converts to a  short.
    /// - Otherwise, if in the range of int (e.g. 12345678.0, -1.23e8), converts to an  int.
    /// - Otherwise, if in the range of long (e.g. 2147483649), converts to a  long.
    /// - Otherwise, if it can be stored precisely by float (e.g. 0.5, 31.75), converts to a  float.
    /// - Otherwise, converts to a  double.
    fn number_to_tag(number: &Number, key: Option<String>) -> Nbt {
        if let Some(value) = number.as_i64() {
            if value >= i8::MIN as i64 && value <= i8::MAX as i64 {
                Nbt::Byte {
                    name: key,
                    value: value as i8,
                }
            } else if value >= i16::MIN as i64 && value <= i16::MAX as i64 {
                Nbt::Short {
                    name: key,
                    value: value as i16,
                }
            } else if value >= i32::MIN as i64 && value <= i32::MAX as i64 {
                Nbt::Int {
                    name: key,
                    value: value as i32,
                }
            } else {
                Nbt::Long { name: key, value }
            }
        } else if let Some(value) = number.as_f64() {
            if value.fract() == 0.0 {
                if value >= i8::MIN as f64 && value <= i8::MAX as f64 {
                    Nbt::Byte {
                        name: key,
                        value: value as i8,
                    }
                } else if value >= i16::MIN as f64 && value <= i16::MAX as f64 {
                    Nbt::Short {
                        name: key,
                        value: value as i16,
                    }
                } else if value >= i32::MIN as f64 && value <= i32::MAX as f64 {
                    Nbt::Int {
                        name: key,
                        value: value as i32,
                    }
                } else {
                    Nbt::Long {
                        name: key,
                        value: value as i64,
                    }
                }
            } else if value >= f32::MIN as f64 && value <= f32::MAX as f64 {
                Nbt::Float {
                    name: key,
                    value: value as f32,
                }
            } else {
                Nbt::Double { name: key, value }
            }
        } else {
            panic!("Unsupported number type");
        }
    }

    /// The conversion from JsonArray to NBT is a little buggy.
    ///
    /// First converts all the elements in the array to NBT, if their data types are different, this array cannot be converted into NBT. That means arrays like [0,1,true] and [5e-1,0.25] can be converted to NBT successfully, while [0,1,128], [0.5, 0.6], and [0.0, 0.1] cannot be converted to NBT.
    ///
    /// And when it can be converted to NBT:
    ///
    /// - If the elements are converted to byte, the array is converted to a byte array.
    /// - If the elements are converted to int, the array is converted to an int array.
    /// - If the elements are converted to long, the array is converted to a long array.
    /// - Otherwise, the array is converted to a  list.
    ///
    /// For example, [true, 127] is converted to [B; 1B, 127B].
    fn array_to_tag(array: &Vec<Value>, key: Option<String>) -> Nbt {
        // First converts all the elements in the array to NBT
        let mut nbt_array = Vec::new();
        for element in array {
            let nbt = Nbt::from_json(element, None);
            nbt_array.push(nbt);
        }

        // Then get the type of the array
        let tag_type = Nbt::get_type_of_array(&nbt_array);

        match tag_type {
            1 => Nbt::ByteArray {
                name: key,
                value: nbt_array
                    .iter()
                    .map(|nbt| match nbt {
                        Nbt::Byte { value, .. } => *value,
                        _ => panic!("Array elements have different data types"),
                    })
                    .collect(),
            },
            3 => Nbt::IntArray {
                name: key,
                value: nbt_array
                    .iter()
                    .map(|nbt| match nbt {
                        Nbt::Int { value, .. } => *value,
                        _ => panic!("Array elements have different data types"),
                    })
                    .collect(),
            },
            4 => Nbt::LongArray {
                name: key,
                value: nbt_array
                    .iter()
                    .map(|nbt| match nbt {
                        Nbt::Long { value, .. } => *value,
                        _ => panic!("Array elements have different data types"),
                    })
                    .collect(),
            },
            _ => Nbt::List {
                name: key,
                value: nbt_array,
                tag_type,
            },
        }
    }

    fn get_type_of_array(array: &[Nbt]) -> u8 {
        array
            .iter()
            .map(|element| element.get_tag_type())
            .reduce(|a, b| if a == b { a } else { 5 })
            .unwrap_or(0)
    }
}
