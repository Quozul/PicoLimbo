#![feature(test)]

#[cfg(test)]
mod tests {
    extern crate test;
    use nbt::prelude::Nbt;
    use test::Bencher;

    #[bench]
    fn bench_nbt_to_bytes(b: &mut Bencher) {
        // Given
        let nbt = Nbt::Compound {
            name: Some("Hello, World".to_string()),
            value: vec![
                Nbt::Int {
                    name: Some("First".to_string()),
                    value: 123,
                },
                Nbt::List {
                    name: None,
                    value: vec![
                        Nbt::Int {
                            name: Some("test".to_string()),
                            value: 123,
                        },
                        Nbt::Short {
                            name: Some("test".to_string()),
                            value: 42,
                        },
                    ],
                    tag_type: 3,
                },
            ],
        };

        b.iter(|| {
            nbt.to_bytes(true);
        });
    }
}
