use protocol_version::protocol_version::ProtocolVersion;
use serde::Deserialize;
use std::io::Write;
use std::path::Path;
use tracing::debug;

const VERSION_MANIFEST_URL: &str = "https://launchermeta.mojang.com/mc/game/version_manifest.json";

#[derive(Deserialize)]
pub struct Version {
    id: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionDownload {
    url: String,
}

#[derive(Deserialize)]
struct VersionDownloads {
    server: VersionDownload,
}

#[derive(Deserialize)]
struct VersionMetadata {
    downloads: VersionDownloads,
}

impl Version {
    pub async fn download_server_jar(&self, download_location: &Path) -> anyhow::Result<()> {
        debug!("Downloading version {} metadata", self.id);
        let response = reqwest::get(&self.url).await?;
        let version_metadata = response.json::<VersionMetadata>().await?;
        let server_jar_url = version_metadata.downloads.server.url;
        debug!("Downloading server jar to {:?}", download_location);
        let server_jar_response = reqwest::get(server_jar_url).await?;
        let mut server_jar_file = std::fs::File::create(download_location)?;
        server_jar_file.write_all(&server_jar_response.bytes().await?)?;
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct VersionManifest {
    versions: Vec<Version>,
}

impl VersionManifest {
    pub fn find_version(&self, protocol_version: &ProtocolVersion) -> Option<&Version> {
        debug!("Looking for version {}", protocol_version.humanize());
        self.versions
            .iter()
            .find(|v| v.id == protocol_version.humanize())
    }
}

pub async fn get_version_manifest() -> anyhow::Result<VersionManifest> {
    debug!("Fetching version manifest");
    let response = reqwest::get(VERSION_MANIFEST_URL).await?;
    let version_manifest = response.json::<VersionManifest>().await?;
    Ok(version_manifest)
}
