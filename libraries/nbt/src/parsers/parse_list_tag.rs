use crate::binary_reader::BinaryReader;
use crate::nbt::Nbt;
use crate::parsers::parse_with_type::parse_with_type;

pub fn parse_list_tag(reader: &mut BinaryReader) -> (u8, Vec<Nbt>) {
    let mut values = Vec::new();

    let tag_type = reader.read_type();
    let list_length = reader.read_i32();
    if list_length <= 0 && tag_type == 0 {
        return (tag_type, values);
    }

    for _ in 0..list_length {
        let next_tag = parse_with_type(reader, tag_type, true);
        values.push(next_tag);
    }

    (tag_type, values)
}
