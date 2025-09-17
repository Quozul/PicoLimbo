use anyhow::{Context, Result, anyhow, bail};
use futures::future::join_all;
use protocol_version::protocol_version::ProtocolVersion;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use tokio::{fs, process::Command};

/// Type definitions for deserializing Mojang's version manifest JSON.
#[derive(Debug, Deserialize)]
struct VersionManifest {
    versions: Vec<VersionInfo>,
}

#[derive(Debug, Deserialize)]
struct VersionInfo {
    id: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct Version {
    downloads: Downloads,
}

#[derive(Debug, Deserialize)]
struct Downloads {
    server: ServerDownload,
}

#[derive(Debug, Deserialize)]
struct ServerDownload {
    url: String,
}

/// A struct to hold information about a downloaded or existing server jar.
#[derive(Debug, Clone)]
struct ServerJar {
    version: String,
    path: PathBuf,
    file_name: String,
}

const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

/// Generic helper to fetch a URL and deserialize the JSON response.
async fn api<T: for<'de> Deserialize<'de>>(url: &str) -> Result<T> {
    reqwest::get(url)
        .await
        .context("Failed to send request")?
        .json::<T>()
        .await
        .context("Failed to deserialize JSON response")
}

/// Downloads a server jar from its version metadata URL and saves it.
async fn download_jar(version_url: &str, save_path: &Path) -> Result<()> {
    println!("Downloading jar for version url: {}", version_url);
    let version_meta: Version = api(version_url).await?;
    let server_download_url = version_meta.downloads.server.url;

    let response = reqwest::get(&server_download_url)
        .await?
        .error_for_status()?;
    let content = response.bytes().await?;

    fs::write(save_path, &content).await?;
    println!("Successfully downloaded to {}", save_path.display());
    Ok(())
}

/// Downloads all specified Minecraft server jars if they don't already exist.
async fn download_server_jars(
    versions_to_download: &[&str],
    save_path: &Path,
) -> Result<Vec<ServerJar>> {
    if !save_path.exists() {
        fs::create_dir_all(save_path).await?;
    }

    let check_futs = versions_to_download.iter().map(|&version| {
        let path = save_path.join(format!("{}.jar", version));
        async move { (version, path.exists()) }
    });
    let checks = join_all(check_futs).await;

    if checks.iter().all(|(_, exists)| *exists) {
        println!("All server jars already exist.");
        return Ok(versions_to_download
            .iter()
            .map(|&v| {
                let file_name = format!("{}.jar", v);
                ServerJar {
                    version: v.to_string(),
                    path: save_path.join(&file_name),
                    file_name,
                }
            })
            .collect());
    }

    println!("Fetching version manifest...");
    let manifest: VersionManifest = api(VERSION_MANIFEST_URL).await?;

    let versions_map: HashMap<String, String> = manifest
        .versions
        .into_iter()
        .map(|v| (v.id, v.url))
        .collect();

    let download_futs = versions_to_download.iter().map(|&version| {
        let file_name = format!("{}.jar", version);
        let server_jar_path = save_path.join(&file_name);
        let version_url = versions_map.get(version).cloned();

        async move {
            if !server_jar_path.exists() {
                match version_url {
                    Some(url) => {
                        download_jar(&url, &server_jar_path)
                            .await
                            .with_context(|| {
                                format!("Failed to download jar for version {}", version)
                            })?;
                    }
                    None => {
                        bail!("Version {} not found in manifest", version);
                    }
                }
            }
            Ok(ServerJar {
                version: version.to_string(),
                path: server_jar_path,
                file_name,
            })
        }
    });

    let results = join_all(download_futs).await;
    results.into_iter().collect()
}

//==================================================================================//
// Part 2: Logic from generate.ts (Updated)
//==================================================================================//

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

/// Replaces text in all files within the wolf_variant directory.
async fn clean_wolf_variants(path: &Path) -> Result<()> {
    let mut dir = fs::read_dir(path).await?;
    while let Some(entry) = dir.next_entry().await? {
        if entry.file_type().await?.is_file() {
            let file_path = entry.path();
            let contents = fs::read_to_string(&file_path).await?;
            let new_contents = contents.replace("#minecraft:is_", "minecraft:");
            fs::write(&file_path, new_contents).await?;
        }
    }
    Ok(())
}

/// Removes all files in a directory except for `packets.json` and `blocks.json`.
async fn clean_reports_directory(path: &Path) -> Result<()> {
    let mut dir = fs::read_dir(path).await?;
    while let Some(entry) = dir.next_entry().await? {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();
        if file_name_str != "packets.json" && file_name_str != "blocks.json" {
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path).await?;
            } else {
                fs::remove_file(&path).await?;
            }
        }
    }
    Ok(())
}

/// Cleans the generated data directory by keeping only whitelisted files and folders.
async fn clean_data_directory(data_path: &Path) -> Result<()> {
    // Whitelist of files and directories to keep, based on the provided image.
    let files_to_keep = [
        "minecraft/cat_variant/red.json",
        "minecraft/chicken_variant/temperate.json",
        "minecraft/cow_variant/temperate.json",
        "minecraft/dimension_type/overworld.json",
        "minecraft/dimension_type/the_end.json",
        "minecraft/dimension_type/the_nether.json",
        "minecraft/frog_variant/temperate.json",
        "minecraft/painting_variant/fire.json",
        "minecraft/pig_variant/temperate.json",
        "minecraft/wolf_sound_variant/classic.json",
        "minecraft/wolf_variant/pale.json",
        "minecraft/worldgen/biome/plains.json",
    ];

    let dirs_to_keep = [
        "minecraft/damage_type", // Keep this entire directory
    ];

    // 1. Create a temporary directory for the cleaned data.
    let parent = data_path.parent().context("Data path has no parent")?;
    let temp_clean_path = parent.join("data_cleaned");
    if temp_clean_path.exists() {
        fs::remove_dir_all(&temp_clean_path).await?;
    }
    fs::create_dir(&temp_clean_path).await?;

    // 2. Copy whitelisted files into the temp directory.
    for file_path_str in files_to_keep {
        let src_path = data_path.join(file_path_str);
        if src_path.exists() {
            let dest_path = temp_clean_path.join(file_path_str);
            if let Some(p) = dest_path.parent() {
                fs::create_dir_all(p).await?;
            }
            fs::copy(&src_path, &dest_path).await.with_context(|| {
                format!("Failed to copy whitelisted file: {}", src_path.display())
            })?;
        }
    }

    // 3. Copy whitelisted directories into the temp directory.
    for dir_path_str in dirs_to_keep {
        let src_path = data_path.join(dir_path_str);
        if src_path.exists() && src_path.is_dir() {
            let dest_path = temp_clean_path.join(dir_path_str);
            copy_dir_recursive(&src_path, &dest_path)
                .await
                .with_context(|| {
                    format!(
                        "Failed to copy whitelisted directory: {}",
                        src_path.display()
                    )
                })?;
        }
    }

    // 4. Replace the old directory with the new one.
    fs::remove_dir_all(data_path).await?;
    fs::rename(&temp_clean_path, data_path).await?;

    Ok(())
}

//==================================================================================//
// Main application logic (Updated)
//==================================================================================//

#[tokio::main]
async fn main() -> Result<()> {
    // UPDATE: Dynamically get all modern Minecraft versions.
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

        // UPDATE: Determine the correct Java command based on the version.
        let proto_version = version_map
            .get(&version_jar.version)
            .with_context(|| format!("ProtocolVersion not found for {}", version_jar.version))?;

        // The command format changed in 1.18.
        let version_1_18 = &ProtocolVersion::V1_18;
        let command = if proto_version.is_after_inclusive(*version_1_18) {
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

        let data_directory = move_subdir(generated_directory, &output_directory, "data").await?;
        let reports_directory =
            move_subdir(generated_directory, &output_directory, "reports").await?;

        println!("Cleaning generated data for {}...", version_jar.version);
        clean_data_directory(&data_directory).await?;

        let wolf_variant_path = data_directory.join("minecraft/wolf_variant");
        if wolf_variant_path.exists() {
            clean_wolf_variants(&wolf_variant_path).await?;
        }

        clean_reports_directory(&reports_directory).await?;
    }

    println!("All versions processed successfully.");
    Ok(())
}
