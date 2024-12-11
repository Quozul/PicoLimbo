use crate::binary_reader::BinaryReader;
use crate::nbt::Nbt;
use crate::parsers::parse_with_type::parse_with_type;

pub fn parse_tag(reader: &mut BinaryReader) -> Nbt {
    let tag_type = reader.read_type();
    parse_with_type(reader, tag_type, false)
}
