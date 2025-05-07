use std::{env, path::PathBuf};

fn main() {
    let assets_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("../data/generated");

    let tar_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("assets.tar.gz");

    asset_pipeline::compress_dir(&assets_dir, &tar_path).expect("failed to compress assets");

    println!("cargo:rerun-if-changed={}", assets_dir.display());
}
