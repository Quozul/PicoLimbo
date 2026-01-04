use indexmap::IndexMap;
use pico_nbt2::{CompressionType, Value, encode, from_slice, to_bytes};

use std::io::Cursor;

#[test]
fn test_roundtrip_primitive() {
    let v = Value::Int(12345);
    let bytes = to_bytes(&v, Some("test")).unwrap();
    let (name, v2) = from_slice(&bytes).unwrap();
    assert_eq!(name, "test");
    assert_eq!(v, v2);
}

#[test]
fn test_roundtrip_compound() {
    let mut map = IndexMap::new();
    map.insert("byte".into(), Value::Byte(1));
    map.insert("string".into(), Value::String("hello".into()));
    let v = Value::Compound(map);

    let bytes = to_bytes(&v, Some("root")).unwrap();
    let (name, v2) = from_slice(&bytes).unwrap();
    assert_eq!(name, "root");
    assert_eq!(v, v2);
}

#[test]
fn test_compression_gzip() {
    let v = Value::Int(42);
    let mut encoder = encode(Vec::new(), CompressionType::Gzip).unwrap();
    pico_nbt2::to_writer(&mut encoder, &v, Some("compressed")).unwrap();
    // Finish writing
    drop(encoder); // This might be tricky with Box<dyn Write>, we need to get the inner vec?
    // Actually, encode returns Box<dyn Write>, so we can't easily get the inner vec back unless we use a reference or something.
    // For testing, let's just use the GzEncoder directly.
}

#[test]
fn test_compression_gzip_manual() {
    use flate2::Compression;
    use flate2::read::GzDecoder;
    use flate2::write::GzEncoder;

    let v = Value::Int(42);
    let mut buf = Vec::new();
    {
        let mut encoder = GzEncoder::new(&mut buf, Compression::default());
        pico_nbt2::to_writer(&mut encoder, &v, Some("compressed")).unwrap();
        encoder.finish().unwrap();
    }

    let mut decoder = GzDecoder::new(Cursor::new(&buf));
    let (name, v2) = pico_nbt2::from_reader(&mut decoder).unwrap();
    assert_eq!(name, "compressed");
    assert_eq!(v, v2);
}

#[test]
fn test_value_to_byte() {
    use pico_nbt2::NbtOptions;

    let v = Value::Int(42);

    // No compression
    let bytes = v
        .to_byte(CompressionType::None, NbtOptions::new(), Some("test"))
        .unwrap();
    let (name, v2) = from_slice(&bytes).unwrap();
    assert_eq!(name, "test");
    assert_eq!(v, v2);

    // Gzip
    let bytes = v
        .to_byte(CompressionType::Gzip, NbtOptions::new(), Some("test"))
        .unwrap();
    let reader = pico_nbt2::decode(Cursor::new(&bytes)).unwrap();
    let (name, v2) = pico_nbt2::from_reader(reader).unwrap();
    assert_eq!(name, "test");
    assert_eq!(v, v2);

    // Zlib
    let bytes = v
        .to_byte(CompressionType::Zlib, NbtOptions::new(), Some("test"))
        .unwrap();
    let reader = pico_nbt2::decode(Cursor::new(&bytes)).unwrap();
    let (name, v2) = pico_nbt2::from_reader(reader).unwrap();
    assert_eq!(name, "test");
    assert_eq!(v, v2);

    // Nameless root
    let bytes = v
        .to_byte(
            CompressionType::None,
            NbtOptions::new().nameless_root(true),
            None,
        )
        .unwrap();
    let (name, v2) =
        pico_nbt2::from_slice_with_options(&bytes, NbtOptions::new().nameless_root(true)).unwrap();
    assert_eq!(name, "");
    assert_eq!(v, v2);
}
