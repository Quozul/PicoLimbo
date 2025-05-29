use minecraft_server::server::GetDataDirectory;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct ServerState {
    secret_key: Vec<u8>,
    modern_forwarding: bool,
    data_directory: PathBuf,
}

impl ServerState {
    /// Start building a new ServerState.
    pub fn builder() -> ServerStateBuilder {
        ServerStateBuilder::default()
    }

    pub fn secret_key(&self) -> &[u8] {
        &self.secret_key
    }

    pub fn is_modern_forwarding(&self) -> bool {
        self.modern_forwarding
    }
}

impl GetDataDirectory for ServerState {
    fn data_directory(&self) -> &PathBuf {
        &self.data_directory
    }
}

#[derive(Error, Debug)]
pub enum ServerStateBuildError {
    #[error("asset_directory was not set")]
    MissingAssetDirectory,
}

#[derive(Default, Debug)]
pub struct ServerStateBuilder {
    secret_key: Option<Vec<u8>>,
    modern_forwarding: bool,
    asset_directory: Option<PathBuf>,
}

impl ServerStateBuilder {
    /// Set the secret key. If you never call this, it'll default to `Vec::new()`.
    pub fn secret_key<K>(&mut self, key: K) -> &mut Self
    where
        K: Into<Vec<u8>>,
    {
        self.secret_key = Some(key.into());
        self
    }

    /// Enable or disable modern forwarding. Defaults to `false`.
    pub fn modern_forwarding(&mut self, enabled: bool) -> &mut Self {
        self.modern_forwarding = enabled;
        self
    }

    /// Set the asset directory (required).
    pub fn data_directory<P>(&mut self, path: P) -> &mut Self
    where
        P: Into<PathBuf>,
    {
        self.asset_directory = Some(path.into());
        self
    }

    /// Finish building, returning an error if any required fields are missing.
    pub fn build(self) -> Result<ServerState, ServerStateBuildError> {
        Ok(ServerState {
            secret_key: self.secret_key.unwrap_or_default(),
            modern_forwarding: self.modern_forwarding,
            data_directory: self
                .asset_directory
                .ok_or(ServerStateBuildError::MissingAssetDirectory)?,
        })
    }
}
