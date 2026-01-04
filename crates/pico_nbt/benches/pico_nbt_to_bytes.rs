#![feature(test)]

#[cfg(test)]
mod tests {
    extern crate test;
    use pico_nbt::prelude::{Nbt, NbtFeatures};
    use test::Bencher;

    #[bench]
    fn pico_nbt_to_bytes(b: &mut Bencher) {
        // Given
        let features = NbtFeatures::builder().dynamic_lists().build();
        let nbt = Nbt::Compound {
            name: Some("Hello, World".to_string()),
            value: vec![
                Nbt::Int {
                    name: Some("First".to_string()),
                    value: 123,
                },
                Nbt::List {
                    name: Some("Second".to_string()),
                    value: vec![
                        Nbt::Int {
                            name: None,
                            value: 123,
                        },
                        Nbt::Short {
                            name: None,
                            value: 42,
                        },
                    ],
                    tag_type: 3,
                },
            ],
        };

        b.iter(|| {
            nbt.to_bytes(features).unwrap();
        });
    }
}
