use anyhow::{Context, bail};
use futures::future::join_all;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

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
pub struct ServerJar {
    pub version: String,
    pub path: PathBuf,
    pub file_name: String,
}

const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

/// Generic helper to fetch a URL and deserialize the JSON response.
async fn api<T: for<'de> Deserialize<'de>>(url: &str) -> anyhow::Result<T> {
    reqwest::get(url)
        .await
        .context("Failed to send request")?
        .json::<T>()
        .await
        .context("Failed to deserialize JSON response")
}

/// Downloads a server jar from its version metadata URL and saves it.
async fn download_jar(version_url: &str, save_path: &Path) -> anyhow::Result<()> {
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
pub async fn download_server_jars(
    versions_to_download: &[&str],
    save_path: &Path,
) -> anyhow::Result<Vec<ServerJar>> {
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
