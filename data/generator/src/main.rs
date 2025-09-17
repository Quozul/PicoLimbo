mod download_server_jars;

use crate::download_server_jars::download_server_jars;
use anyhow::{Context, Result, anyhow, bail};
use protocol_version::protocol_version::ProtocolVersion;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tokio::{fs, process::Command};

/// Executes a shell command in a given directory and returns its output.
async fn execute(command_str: &str, cwd: &Path) -> Result<String> {
    let mut parts = command_str.split_whitespace();
    let program = parts.next().ok_or_else(|| anyhow!("Empty command"))?;
    let args: Vec<&str> = parts.collect();

    let output = Command::new(program)
        .args(&args)
        .current_dir(cwd)
        .output()
        .await
        .context("Failed to execute command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Command failed with exit code {}:\n{}",
            output.status,
            stderr
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Recursively copies a directory.
async fn copy_dir_recursive(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();
    fs::create_dir_all(&dst).await?;
    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let ty = entry.file_type().await?;
        if ty.is_dir() {
            Box::pin(copy_dir_recursive(
                entry.path(),
                dst.join(entry.file_name()),
            ))
            .await?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name())).await?;
        }
    }
    Ok(())
}

/// Copies a subdirectory from a source to a destination directory.
async fn move_subdir(from: &Path, to: &Path, subdir: &str) -> Result<PathBuf> {
    let source = from.join(subdir);
    let destination = to.join(subdir);
    copy_dir_recursive(&source, &destination).await?;
    Ok(destination)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Dynamically get all modern Minecraft versions.
    let supported_versions: Vec<ProtocolVersion> = ProtocolVersion::ALL_VERSION
        .iter()
        .filter(|v| v.is_modern())
        .cloned()
        .collect();

    // Create a map for quick lookup from version string to ProtocolVersion struct.
    let version_map: HashMap<String, ProtocolVersion> = supported_versions
        .into_iter()
        .map(|v| (v.humanize().to_string(), v))
        .collect();

    let version_strings: Vec<&str> = version_map.keys().map(|s| s.as_str()).collect();

    let manifest_directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let data_directory = manifest_directory.parent().unwrap();
    let server_jar_directory = data_directory.join("servers");
    let jar_files = download_server_jars(&version_strings, &server_jar_directory).await?;

    for version_jar in jar_files {
        let output_directory = data_directory
            .join("generated")
            .join(format!("V{}", version_jar.version.replace('.', "_")));

        if output_directory.exists() {
            println!(
                "Skipping version {}: output directory already exists.",
                version_jar.version
            );
            continue;
        }

        let temp_dir = tempfile::Builder::new()
            .prefix(&format!("generated_{}", version_jar.version))
            .tempdir()?;
        let generated_directory = temp_dir.path();

        // Determine the correct Java command based on the version.
        let proto_version = version_map
            .get(&version_jar.version)
            .with_context(|| format!("ProtocolVersion not found for {}", version_jar.version))?;

        // The command format changed in 1.18.
        let command = if proto_version.is_after_inclusive(ProtocolVersion::V1_18) {
            // Command for 1.18 and later
            format!(
                "java -DbundlerMainClass=net.minecraft.data.Main -jar {} --reports --server --output {}",
                version_jar.file_name,
                generated_directory.display()
            )
        } else {
            // Command for versions before 1.18
            format!(
                "java -cp {} net.minecraft.data.Main --reports --server --output {}",
                version_jar.file_name,
                generated_directory.display()
            )
        };

        println!(
            "Running data generator for version {}...",
            version_jar.version
        );
        match execute(&command, &server_jar_directory).await {
            Ok(_) => {
                println!(
                    "Generated {}: {}",
                    version_jar.version,
                    version_jar.path.display()
                );
            }
            Err(e) => {
                eprintln!(
                    "An error occurred while processing version {}: {:?}",
                    version_jar.file_name, e
                );
                continue;
            }
        }

        move_subdir(generated_directory, &output_directory, "data").await?;
        move_subdir(generated_directory, &output_directory, "reports").await?;
    }

    println!("All versions processed successfully.");
    Ok(())
}
