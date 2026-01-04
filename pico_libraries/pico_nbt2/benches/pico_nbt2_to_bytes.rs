#![feature(test)]

#[cfg(test)]
mod tests {
    extern crate test;

    use indexmap::IndexMap;
    use pico_nbt2::{CompressionType, NbtOptions, Value};
    use std::io::BufWriter;
    use test::Bencher;

    #[bench]
    fn pico_nbt2_to_bytes(b: &mut Bencher) {
        let options = NbtOptions::new().dynamic_lists(true);
        let nbt_value = Value::Compound(IndexMap::from([
            ("First".to_string(), Value::Int(123)),
            (
                "Second".to_string(),
                Value::List(vec![Value::Int(123), Value::Short(42)]),
            ),
        ]));

        b.iter(|| {
            let buf = Vec::<u8>::with_capacity(1_024);
            let writer = BufWriter::new(buf);
            let mut encoder = pico_nbt2::encode(writer, CompressionType::None).unwrap();
            pico_nbt2::to_writer_value_with_options(
                &mut encoder,
                &nbt_value,
                Some("Hello, World"),
                options,
            )
            .unwrap();
        });
    }
}
