use minecraft_packets::play::Dimension;
use minecraft_server::prelude::ConnectedClients;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct ServerState {
    secret_key: Vec<u8>,
    modern_forwarding: bool,
    data_directory: PathBuf,
    spawn_dimension: Dimension,
    description_text: String,
    max_players: u32,
    welcome_message: String,
    connected_clients: Arc<AtomicU32>,
    show_online_player_count: bool,
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

    pub fn description_text(&self) -> &str {
        &self.description_text
    }

    pub fn max_players(&self) -> u32 {
        self.max_players
    }

    pub fn welcome_message(&self) -> Option<String> {
        if self.welcome_message.is_empty() {
            None
        } else {
            Some(self.welcome_message.clone())
        }
    }

    /// Returns the current number of connected clients.
    pub fn online_players(&self) -> u32 {
        if self.show_online_player_count {
            self.connected_clients.load(Ordering::SeqCst)
        } else {
            0
        }
    }

    pub fn spawn_dimension(&self) -> &Dimension {
        &self.spawn_dimension
    }

    pub fn data_directory(&self) -> &PathBuf {
        &self.data_directory
    }
}

impl ConnectedClients for ServerState {
    fn increment(&self) {
        self.connected_clients.fetch_add(1, Ordering::SeqCst);
    }

    fn decrement(&self) {
        self.connected_clients.fetch_sub(1, Ordering::SeqCst);
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
    dimension: Option<Dimension>,
    description_text: String,
    max_players: u32,
    welcome_message: String,
    show_online_player_count: bool,
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

    /// Set the spawn dimension
    pub fn dimension(&mut self, dimension: Dimension) -> &mut Self {
        self.dimension = Some(dimension);
        self
    }

    pub fn description_text<S>(&mut self, text: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.description_text = text.into();
        self
    }

    pub fn max_players(&mut self, max_players: u32) -> &mut Self {
        self.max_players = max_players;
        self
    }

    pub fn welcome_message<S>(&mut self, message: S) -> &mut Self
    where
        S: Into<String>,
    {
        self.welcome_message = message.into();
        self
    }

    pub fn show_online_player_count(&mut self, show: bool) -> &mut Self {
        self.show_online_player_count = show;
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
            spawn_dimension: self.dimension.unwrap_or_default(),
            description_text: self.description_text,
            max_players: self.max_players,
            welcome_message: self.welcome_message,
            connected_clients: Arc::new(AtomicU32::new(0)),
            show_online_player_count: self.show_online_player_count,
        })
    }
}
