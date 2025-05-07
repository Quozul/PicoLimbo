//! A small crate to compress a directory into .tar.gz
//! and to extract it (skipping if already exists).

use std::fs::{self, File};
use std::io;
use std::path::Path;

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use tar::{Archive, Builder};

/// Compress the entire directory `src_dir` (recursively)
/// into a gzip‐compressed tar file at `dest_file`.
pub fn compress_dir<P: AsRef<Path>, Q: AsRef<Path>>(src_dir: P, dest_file: Q) -> io::Result<()> {
    let tar_gz = File::create(dest_file)?;
    let encoder = GzEncoder::new(tar_gz, Compression::default());
    let mut tar = Builder::new(encoder);
    tar.append_dir_all(".", src_dir)?;
    Ok(())
}

/// Given a gzip‐compressed tar archive in memory (`archive_data`),
/// unpack it into `dest_dir`.
/// If `dest_dir` already exists, extraction is skipped.
pub fn extract_archive<P: AsRef<Path>>(archive_data: &[u8], dest_dir: P) -> io::Result<()> {
    let dest = dest_dir.as_ref();
    if dest.exists() {
        // skip if already extracted
        return Ok(());
    }

    // create parent dirs (and the dest folder)
    fs::create_dir_all(dest)?;

    let decoder = GzDecoder::new(archive_data);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

/// Embed a file from OUT_DIR into your binary.
///
/// Usage:
///   // default: looks for "${OUT_DIR}/assets.tar.gz"
///   const ASSETS: &[u8] = embed_assets!();
///
///   // custom: e.g. "${OUT_DIR}/my_model.bin"
///   const MODEL: &[u8] = embed_assets!("my_model.bin");
#[macro_export]
macro_rules! embed_assets {
    ($file:literal) => {
        include_bytes!(concat!(env!("OUT_DIR"), "/", $file))
    };
    () => {
        include_bytes!(concat!(env!("OUT_DIR"), "/assets.tar.gz"))
    };
}
