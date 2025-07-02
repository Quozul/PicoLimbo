use crate::binary_reader::BinaryReader;
use crate::prelude::Nbt;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum NbtDecodeError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid UTF-8 string: {0}")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Unexpected end of data while trying to read {0} bytes")]
    UnexpectedEof(usize),
    #[error("Unsupported tag type: {0}")]
    UnsupportedTagType(u8),
}

type Result<T> = std::result::Result<T, NbtDecodeError>;

fn parse_tag(reader: &mut BinaryReader) -> Result<Nbt> {
    let tag_type = reader.read_u8();
    parse_with_type(reader, tag_type, false)
}

fn parse_compound_tag(reader: &mut BinaryReader) -> Result<Vec<Nbt>> {
    let mut values = Vec::new();

    loop {
        let next_tag = parse_tag(reader)?;
        if next_tag == Nbt::End {
            break;
        }
        values.push(next_tag);
    }

    Ok(values)
}

fn parse_list_tag(reader: &mut BinaryReader) -> Result<(u8, Vec<Nbt>)> {
    let tag_type = reader.read_u8();
    let list_length = reader.read_i32();

    if list_length <= 0 && tag_type == 0 {
        return Ok((tag_type, Vec::new()));
    }

    let mut values = Vec::with_capacity(list_length as usize);
    for _ in 0..list_length {
        let next_tag = parse_with_type(reader, tag_type, true)?;
        values.push(next_tag);
    }

    Ok((tag_type, values))
}

fn parse_with_type(reader: &mut BinaryReader, tag_type: u8, skip_name: bool) -> Result<Nbt> {
    let name = if skip_name || tag_type == 0 {
        None
    } else {
        reader.read_name()
    };

    match tag_type {
        0 => Ok(Nbt::End),
        1 => {
            let value = reader.read_i8();
            Ok(Nbt::Byte { name, value })
        }
        2 => {
            let value = reader.read_i16();
            Ok(Nbt::Short { name, value })
        }
        3 => {
            let value = reader.read_i32();
            Ok(Nbt::Int { name, value })
        }
        4 => {
            let value = reader.read_i64();
            Ok(Nbt::Long { name, value })
        }
        5 => {
            let value = reader.read_f32();
            Ok(Nbt::Float { name, value })
        }
        6 => {
            let value = reader.read_f64();
            Ok(Nbt::Double { name, value })
        }
        7 => {
            let value = reader.read_byte_array();
            Ok(Nbt::ByteArray { name, value })
        }
        8 => {
            let value = reader.read_string().unwrap_or_default();
            Ok(Nbt::String { name, value })
        }
        9 => {
            let (tag_type, value) = parse_list_tag(reader)?;
            Ok(Nbt::List {
                name,
                value,
                tag_type,
            })
        }
        10 => {
            let value = parse_compound_tag(reader)?;
            Ok(Nbt::Compound { name, value })
        }
        11 => {
            let value = reader.read_int_array();
            Ok(Nbt::IntArray { name, value })
        }
        12 => {
            let value = reader.read_long_array();
            Ok(Nbt::LongArray { name, value })
        }
        _ => Err(NbtDecodeError::UnsupportedTagType(tag_type)),
    }
}

impl Nbt {
    pub fn from_file(path: &Path) -> Result<Nbt> {
        let file = File::open(path)?;
        let mut buf_reader = BufReader::new(file);
        let mut buf = Vec::new();
        buf_reader.read_to_end(&mut buf)?;
        let mut binary_reader = BinaryReader::new(&buf);
        parse_tag(&mut binary_reader)
    }
}
