use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Represents an NBT value.
///
/// This enum is designed to be zero-copy where possible, using `Cow` for strings and byte arrays.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    #[serde(with = "serde_bytes")]
    ByteArray(Vec<u8>),
    String(String),
    List(Vec<Self>),
    Compound(IndexMap<String, Self>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl Value {
    /// Returns the NBT tag ID for this value.
    #[must_use]
    pub const fn id(&self) -> u8 {
        match self {
            Self::Byte(_) => 1,
            Self::Short(_) => 2,
            Self::Int(_) => 3,
            Self::Long(_) => 4,
            Self::Float(_) => 5,
            Self::Double(_) => 6,
            Self::ByteArray(_) => 7,
            Self::String(_) => 8,
            Self::List(_) => 9,
            Self::Compound(_) => 10,
            Self::IntArray(_) => 11,
            Self::LongArray(_) => 12,
        }
    }

    #[must_use]
    pub const fn get_byte(&self) -> Option<i8> {
        if let Self::Byte(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_short(&self) -> Option<i16> {
        if let Self::Short(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_int(&self) -> Option<i32> {
        if let Self::Int(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_long(&self) -> Option<i64> {
        if let Self::Long(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_float(&self) -> Option<f32> {
        if let Self::Float(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_double(&self) -> Option<f64> {
        if let Self::Double(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_byte_array(&self) -> Option<&[u8]> {
        if let Self::ByteArray(v) = self {
            Some(v.as_slice())
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_str(&self) -> Option<&str> {
        if let Self::String(v) = self {
            Some(v.as_str())
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_list(&self) -> Option<&[Self]> {
        if let Self::List(vec) = self {
            Some(vec.as_slice())
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_compound(&self) -> Option<&IndexMap<String, Self>> {
        if let Self::Compound(map) = self {
            Some(map)
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_int_array(&self) -> Option<&[i32]> {
        if let Self::IntArray(vec) = self {
            Some(vec.as_slice())
        } else {
            None
        }
    }

    #[must_use]
    pub const fn get_long_array(&self) -> Option<&[i64]> {
        if let Self::LongArray(vec) = self {
            Some(vec.as_slice())
        } else {
            None
        }
    }

    /// Serializes the value to a byte vector.
    ///
    /// # Arguments
    /// * `compression` - NBT compression type
    /// * `options` - NBT options
    /// * `root_name` - Optional name for the root tag
    ///
    /// # Errors
    /// Returns an error if serialization fails.
    pub fn to_byte(
        &self,
        compression: crate::io::CompressionType,
        options: crate::NbtOptions,
        root_name: Option<&str>,
    ) -> crate::error::Result<Vec<u8>> {
        let mut writer = crate::io::encode(Vec::new(), compression)?;
        crate::ser::to_writer_value_with_options(&mut writer, self, root_name, options)?;
        writer.finish()
    }
}

impl From<i8> for Value {
    fn from(v: i8) -> Self {
        Self::Byte(v)
    }
}

impl From<i16> for Value {
    fn from(v: i16) -> Self {
        Self::Short(v)
    }
}

impl From<i32> for Value {
    fn from(v: i32) -> Self {
        Self::Int(v)
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Self::Long(v)
    }
}

impl From<f32> for Value {
    fn from(v: f32) -> Self {
        Self::Float(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Self::Double(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Self::String(v.to_string())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}
