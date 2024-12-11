use crate::binary_reader::BinaryReader;
use crate::nbt::Nbt;
use crate::parse::parse_tag;

pub fn parse_compound_tag(reader: &mut BinaryReader) -> Vec<Nbt> {
    let mut values = Vec::new();

    loop {
        let next_tag = parse_tag(reader);
        if next_tag == Nbt::End {
            break;
        }
        values.push(next_tag);
    }

    values
}
