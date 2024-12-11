use crate::binary_reader::BinaryReader;
use crate::nbt::Nbt;
use crate::parsers::parse_compound_tag::parse_compound_tag;
use crate::parsers::parse_list_tag::parse_list_tag;

pub fn parse_with_type(reader: &mut BinaryReader, tag_type: u8, skip_name: bool) -> Nbt {
    let name = if skip_name || tag_type == 0 {
        None
    } else {
        reader.read_name()
    };

    match tag_type {
        0 => Nbt::End,
        1 => {
            let value = reader.read_i8();
            Nbt::Byte { name, value }
        }
        2 => {
            let value = reader.read_i16();
            Nbt::Short { name, value }
        }
        3 => {
            let value = reader.read_i32();
            Nbt::Int { name, value }
        }
        4 => {
            let value = reader.read_i64();
            Nbt::Long { name, value }
        }
        5 => {
            let value = reader.read_f32();
            Nbt::Float { name, value }
        }
        6 => {
            let value = reader.read_f64();
            Nbt::Double { name, value }
        }
        7 => {
            let value = reader.read_byte_array();
            Nbt::ByteArray { name, value }
        }
        8 => {
            let value = reader.read_string().unwrap_or_default();
            Nbt::String { name, value }
        }
        9 => {
            let (tag_type, value) = parse_list_tag(reader);
            Nbt::List {
                name,
                value,
                tag_type,
            }
        }
        10 => {
            let value = parse_compound_tag(reader);
            Nbt::Compound { name, value }
        }
        11 => {
            let value = reader.read_int_array();
            Nbt::IntArray { name, value }
        }
        12 => {
            let value = reader.read_long_array();
            Nbt::LongArray { name, value }
        }
        _ => panic!("Unsupported tag type {tag_type}"),
    }
}
