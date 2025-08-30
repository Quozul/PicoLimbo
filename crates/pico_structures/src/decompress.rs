use flate2::read::GzDecoder;
use std::fs::File;
use std::io;
use std::io::Read;
use std::path::Path;

pub fn decompress_gz_file(path: &Path) -> io::Result<Vec<u8>> {
    let file = File::open(path)?;

    let mut decoder = GzDecoder::new(file);

    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;

    Ok(buffer)
}
