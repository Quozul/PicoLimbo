use crate::manifest;
use protocol_version::protocol_version::ProtocolVersion;
use std::path::{Path, PathBuf};
use std::process::Command;
use tracing::{debug, info};

pub async fn download_server_jar(protocol_version: &ProtocolVersion) -> anyhow::Result<PathBuf> {
    let jar_file_name = format!("server_{}.jar", protocol_version.to_string().to_lowercase());
    let version_manifest = manifest::get_version_manifest().await?;
    let download_directory = PathBuf::new().join("cache").join("servers");
    if !download_directory.exists() {
        std::fs::create_dir_all(&download_directory)?;
    }
    let download_path = download_directory.join(jar_file_name);
    if !download_path.exists() {
        version_manifest
            .find_version(protocol_version)
            .unwrap()
            .download_server_jar(&download_path)
            .await?;
    } else {
        info!(
            "Server jar already downloaded, if you want to re-download it, delete the file at: {}",
            download_path.display()
        );
    }
    Ok(download_path)
}

pub async fn run_data_generator(
    protocol_version: &ProtocolVersion,
    server_jar: &Path,
) -> anyhow::Result<PathBuf> {
    debug!(
        "Running data generator for protocol version {}",
        protocol_version
    );
    let cache_dir = PathBuf::new().join("cache");
    let server_jar_from_cache = server_jar.strip_prefix(&cache_dir)?;
    let download_directory = PathBuf::new()
        .join("generated")
        .join(protocol_version.to_string());

    if !download_directory.exists() {
        std::fs::create_dir_all(&download_directory)?;
    } else if download_directory.read_dir()?.next().is_some() {
        info!(
            "Data generator already ran for this protocol version, if you want to re-run it, delete the directory at: {}",
            download_directory.display()
        );
        return Ok(download_directory);
    }

    let process = if protocol_version.is_after_inclusive(ProtocolVersion::V1_18) {
        Command::new("java")
            .arg("-DbundlerMainClass=net.minecraft.data.Main")
            .arg("-jar")
            .arg(server_jar_from_cache)
            .arg("--all")
            .arg("--output")
            .arg(&download_directory)
            .current_dir(cache_dir)
            .spawn()
    } else {
        Command::new("java")
            .arg("-cp")
            .arg(server_jar_from_cache)
            .arg("net.minecraft.data.Main")
            .arg("--all")
            .arg("--output")
            .arg(&download_directory)
            .current_dir(cache_dir)
            .spawn()
    };

    process?.wait()?;

    Ok(download_directory)
}
