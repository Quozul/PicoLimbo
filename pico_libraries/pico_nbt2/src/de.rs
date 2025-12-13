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
