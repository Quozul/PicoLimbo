use crate::error::{Error, Result};
use crate::value::Value;
use byteorder::{BigEndian, ReadBytesExt};
use indexmap::IndexMap;
use serde::de::{self, IntoDeserializer, Visitor};
use std::io::Read;

use crate::NbtOptions;

pub struct NbtReader<R> {
    pub(crate) reader: R,
    /// The tag ID of the value we are about to read.
    /// Used primarily by Serde to know what the next type is.
    pub(crate) next_tag_id: Option<u8>,
    options: NbtOptions,
}

impl<R: Read> NbtReader<R> {
    pub const fn new(reader: R) -> Self {
        Self {
            reader,
            next_tag_id: None,
            options: NbtOptions::new(),
        }
    }

    pub const fn new_with_options(reader: R, options: NbtOptions) -> Self {
        Self {
            reader,
            next_tag_id: None,
            options,
        }
    }

    pub(crate) fn read_string(&mut self) -> Result<String> {
        let len = self.reader.read_u16::<BigEndian>()? as usize;
        let mut buf = vec![0; len];
        self.reader.read_exact(&mut buf)?;
        let s = std::str::from_utf8(&buf)
            .map_err(|e| Error::Message(format!("UTF-8 error: {e:?}")))?
            .to_string();
        Ok(s)
    }

    fn read_byte_array(&mut self) -> Result<Vec<u8>> {
        let len = self.reader.read_i32::<BigEndian>()?;
        let len =
            usize::try_from(len).map_err(|_| Error::Message("Invalid array length".into()))?;
        let mut buf = vec![0; len];
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_int_array(&mut self) -> Result<Vec<i32>> {
        let len = self.reader.read_i32::<BigEndian>()?;
        let len =
            usize::try_from(len).map_err(|_| Error::Message("Invalid int array length".into()))?;
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(self.reader.read_i32::<BigEndian>()?);
        }
        Ok(list)
    }

    fn read_long_array(&mut self) -> Result<Vec<i64>> {
        let len = self.reader.read_i32::<BigEndian>()?;
        let len =
            usize::try_from(len).map_err(|_| Error::Message("Invalid long array length".into()))?;
        let mut list = Vec::with_capacity(len);
        for _ in 0..len {
            list.push(self.reader.read_i64::<BigEndian>()?);
        }
        Ok(list)
    }

    fn read_value(&mut self, tag_id: u8) -> Result<Value> {
        match tag_id {
            1 => Ok(Value::Byte(self.reader.read_i8()?)),
            2 => Ok(Value::Short(self.reader.read_i16::<BigEndian>()?)),
            3 => Ok(Value::Int(self.reader.read_i32::<BigEndian>()?)),
            4 => Ok(Value::Long(self.reader.read_i64::<BigEndian>()?)),
            5 => Ok(Value::Float(self.reader.read_f32::<BigEndian>()?)),
            6 => Ok(Value::Double(self.reader.read_f64::<BigEndian>()?)),
            7 => Ok(Value::ByteArray(self.read_byte_array()?)),
            8 => Ok(Value::String(self.read_string()?)),
            9 => {
                let elem_type = self.reader.read_u8()?;
                let len = self.reader.read_i32::<BigEndian>()?;
                let len = usize::try_from(len)
                    .map_err(|_| Error::Message("Invalid list length".into()))?;
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    list.push(self.read_value(elem_type)?);
                }
                Ok(Value::List(list))
            }
            10 => {
                let mut map = IndexMap::new();
                loop {
                    let tag_type = self.reader.read_u8()?;
                    if tag_type == 0 {
                        break;
                    }
                    let name = self.read_string()?;
                    let value = self.read_value(tag_type)?;
                    map.insert(name, value);
                }
                Ok(Value::Compound(map))
            }
            11 => Ok(Value::IntArray(self.read_int_array()?)),
            12 => Ok(Value::LongArray(self.read_long_array()?)),
            id => Err(Error::InvalidTagId(id)),
        }
    }

    /// Reads the root NBT tag.
    pub(crate) fn read_root(&mut self) -> Result<(String, Value)> {
        let tag_id = self.reader.read_u8()?;
        // Note: The root tag is technically allowed to be something other than Compound in very old versions,
        // but practically it is almost always Compound (10).

        let name = if self.options.is_nameless_root() {
            String::new()
        } else {
            self.read_string()?
        };

        let value = self.read_value(tag_id)?;
        Ok((name, value))
    }
}

impl<'de, R: Read> de::Deserializer<'de> for &mut NbtReader<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let tag_id = self
            .next_tag_id
            .ok_or_else(|| Error::Message("Tag ID not set".into()))?;
        self.next_tag_id = None; // Consume it

        match tag_id {
            1 => visitor.visit_i8(self.reader.read_i8()?),
            2 => visitor.visit_i16(self.reader.read_i16::<BigEndian>()?),
            3 => visitor.visit_i32(self.reader.read_i32::<BigEndian>()?),
            4 => visitor.visit_i64(self.reader.read_i64::<BigEndian>()?),
            5 => visitor.visit_f32(self.reader.read_f32::<BigEndian>()?),
            6 => visitor.visit_f64(self.reader.read_f64::<BigEndian>()?),
            7 => visitor.visit_byte_buf(self.read_byte_array()?),
            8 => {
                let s = self.read_string()?;
                visitor.visit_string(s)
            }
            9 => {
                let elem_type = self.reader.read_u8()?;
                let len = self.reader.read_i32::<BigEndian>()?;
                let len = usize::try_from(len)
                    .map_err(|_| Error::Message("Invalid list length".into()))?;
                visitor.visit_seq(ListAccess::new(self, elem_type, len))
            }
            10 => visitor.visit_map(CompoundAccess::new(self)),
            11 => {
                let len = self.reader.read_i32::<BigEndian>()?;
                let len = usize::try_from(len)
                    .map_err(|_| Error::Message("Invalid int array length".into()))?;
                visitor.visit_seq(ListAccess::new(self, 3, len)) // 3 is Int
            }
            12 => {
                let len = self.reader.read_i32::<BigEndian>()?;
                let len = usize::try_from(len)
                    .map_err(|_| Error::Message("Invalid long array length".into()))?;
                visitor.visit_seq(ListAccess::new(self, 4, len)) // 4 is Long
            }
            id => Err(Error::InvalidTagId(id)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct CompoundAccess<'a, R> {
    reader: &'a mut NbtReader<R>,
}

impl<'a, R: Read> CompoundAccess<'a, R> {
    const fn new(reader: &'a mut NbtReader<R>) -> Self {
        Self { reader }
    }
}

impl<'de, R: Read> de::MapAccess<'de> for CompoundAccess<'_, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        let tag_type = self.reader.reader.read_u8()?;
        if tag_type == 0 {
            return Ok(None);
        }
        self.reader.next_tag_id = Some(tag_type);
        let name = self.reader.read_string()?;
        seed.deserialize(name.into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.reader)
    }
}

struct ListAccess<'a, R> {
    reader: &'a mut NbtReader<R>,
    elem_type: u8,
    len: usize,
    current: usize,
}

impl<'a, R: Read> ListAccess<'a, R> {
    const fn new(reader: &'a mut NbtReader<R>, elem_type: u8, len: usize) -> Self {
        Self {
            reader,
            elem_type,
            len,
            current: 0,
        }
    }
}

impl<'de, R: Read> de::SeqAccess<'de> for ListAccess<'_, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        if self.current >= self.len {
            return Ok(None);
        }
        self.current += 1;
        self.reader.next_tag_id = Some(self.elem_type);
        seed.deserialize(&mut *self.reader).map(Some)
    }
}

impl de::IntoDeserializer<'_, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> de::Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Byte(n) => visitor.visit_i8(n),
            Self::Short(n) => visitor.visit_i16(n),
            Self::Int(n) => visitor.visit_i32(n),
            Self::Long(n) => visitor.visit_i64(n),
            Self::Float(n) => visitor.visit_f32(n),
            Self::Double(n) => visitor.visit_f64(n),
            Self::ByteArray(n) => visitor.visit_byte_buf(n),
            Self::String(n) => visitor.visit_string(n),
            Self::List(n) => visitor.visit_seq(SeqAccess::new(n.into_iter())),
            Self::Compound(n) => visitor.visit_map(MapAccess::new(n.into_iter())),
            Self::IntArray(n) => visitor.visit_seq(SeqAccess::new(n.into_iter().map(Value::Int))),
            Self::LongArray(n) => visitor.visit_seq(SeqAccess::new(n.into_iter().map(Value::Long))),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            Self::Compound(value) => {
                let mut iter = value.into_iter();
                let Some((variant, value)) = iter.next() else {
                    return Err(serde::ser::Error::custom("expected enum variant name"));
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::ser::Error::custom(
                        "expected enum variant name: found multiple keys",
                    ));
                }
                (variant, Some(value))
            }
            Self::String(variant) => (variant, None),
            other => {
                return Err(serde::ser::Error::custom(format!(
                    "expected enum variant name: found {other:?}"
                )));
            }
        };

        visitor.visit_enum(EnumAccess { variant, value })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self {
            Self::Byte(n) => visitor.visit_bool(n != 0),
            _ => self.deserialize_any(visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf unit unit_struct seq tuple
        tuple_struct map struct identifier ignored_any
    }
}

struct SeqAccess<I> {
    iter: I,
}

impl<I> SeqAccess<I> {
    const fn new(iter: I) -> Self {
        Self { iter }
    }
}

impl<'de, I> de::SeqAccess<'de> for SeqAccess<I>
where
    I: Iterator<Item = Value>,
{
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.iter
            .next()
            .map_or_else(|| Ok(None), |value| seed.deserialize(value).map(Some))
    }
}

struct MapAccess<I> {
    iter: I,
    value: Option<Value>,
}

impl<I> MapAccess<I> {
    const fn new(iter: I) -> Self {
        Self { iter, value: None }
    }
}

impl<'de, I> de::MapAccess<'de> for MapAccess<I>
where
    I: Iterator<Item = (String, Value)>,
{
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(key.into_deserializer()).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.value.take().map_or_else(
            || Err(serde::ser::Error::custom("value is missing")),
            |value| seed.deserialize(value),
        )
    }
}

struct EnumAccess {
    variant: String,
    value: Option<Value>,
}

impl<'de> de::EnumAccess<'de> for EnumAccess {
    type Error = Error;
    type Variant = VariantAccess;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantAccess)>
    where
        V: de::DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantAccess { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantAccess {
    value: Option<Value>,
}

impl<'de> de::VariantAccess<'de> for VariantAccess {
    type Error = Error;

    fn unit_variant(self) -> Result<()> {
        self.value.map_or(Ok(()), de::Deserialize::deserialize)
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.value.map_or_else(
            || Err(serde::ser::Error::custom("struct variant missing value")),
            |value| seed.deserialize(value),
        )
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::List(v)) => de::Deserializer::deserialize_any(Value::List(v), visitor),
            Some(other) => Err(serde::ser::Error::custom(format!(
                "expected tuple variant, found {other:?}"
            ))),
            None => Err(serde::ser::Error::custom("tuple variant missing value")),
        }
    }

    fn struct_variant<V>(self, _fields: &'static [&'static str], visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Compound(v)) => {
                de::Deserializer::deserialize_any(Value::Compound(v), visitor)
            }
            Some(other) => Err(serde::ser::Error::custom(format!(
                "expected struct variant, found {other:?}"
            ))),
            None => Err(serde::ser::Error::custom("struct variant missing value")),
        }
    }
}

/// Interpret a `Value` as an instance of type `T`.
///
/// # Errors
///
/// This conversion can fail if the structure of the Value does not match the
/// structure expected by `T`, for example if `T` is a struct end the Value
/// contains something other than a Compound. It can also fail if the structure
/// is correct but `T`'s implementation of `Deserialize` decides that something
/// is wrong, for example because a required struct field is missing.
pub fn from_value<T>(value: Value) -> Result<T>
where
    T: de::DeserializeOwned,
{
    de::Deserialize::deserialize(value)
}
