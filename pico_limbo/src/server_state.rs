use crate::server::game_mode::GameMode;
use minecraft_protocol::prelude::Dimension;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum ForwardingMode {
    #[default]
    Disabled,
    Legacy,
    BungeeGuard {
        tokens: Vec<String>,
    },
    Modern {
        secret: Vec<u8>,
    },
}

#[derive(Debug, Error)]
#[error("secret key not set")]
pub struct MisconfiguredForwardingError;

#[derive(Clone, Default)]
pub struct ServerState {
    forwarding_mode: ForwardingMode,
    spawn_dimension: Dimension,
    description_text: String,
    max_players: u32,
    welcome_message: String,
    connected_clients: Arc<AtomicU32>,
    show_online_player_count: bool,
    game_mode: GameMode,
    hardcore: bool,
    spawn_position: (f64, f64, f64),
    view_distance: i32,
    schematic_file_path: String,
}

impl ServerState {
    /// Start building a new `ServerState`.
    pub fn builder() -> ServerStateBuilder {
        ServerStateBuilder::default()
    }

    pub const fn is_legacy_forwarding(&self) -> bool {
        matches!(self.forwarding_mode, ForwardingMode::Legacy)
    }

    pub const fn is_modern_forwarding(&self) -> bool {
        matches!(self.forwarding_mode, ForwardingMode::Modern { .. })
    }

    pub fn secret_key(&self) -> Result<Vec<u8>, MisconfiguredForwardingError> {
        match self.forwarding_mode.clone() {
            ForwardingMode::Modern { secret } => Ok(secret),
            _ => Err(MisconfiguredForwardingError),
        }
    }

    pub const fn is_bungee_guard_forwarding(&self) -> bool {
        matches!(self.forwarding_mode, ForwardingMode::BungeeGuard { .. })
    }

    pub fn tokens(&self) -> Result<Vec<String>, MisconfiguredForwardingError> {
        match self.forwarding_mode.clone() {
            ForwardingMode::BungeeGuard { tokens } => Ok(tokens),
            _ => Err(MisconfiguredForwardingError),
        }
    }

    pub fn description_text(&self) -> &str {
        &self.description_text
    }

    pub const fn max_players(&self) -> u32 {
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

    pub const fn spawn_dimension(&self) -> Dimension {
        self.spawn_dimension
    }

    pub const fn game_mode(&self) -> GameMode {
        self.game_mode
    }

    pub const fn is_hardcore(&self) -> bool {
        self.hardcore
    }

    pub const fn spawn_position(&self) -> (f64, f64, f64) {
        self.spawn_position
    }

    pub const fn view_distance(&self) -> i32 {
        self.view_distance
    }

    pub fn schematic_file_path(&self) -> Option<PathBuf> {
        if self.schematic_file_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(&self.schematic_file_path))
        }
    }

    pub fn increment(&self) {
        self.connected_clients.fetch_add(1, Ordering::SeqCst);
    }

    pub fn decrement(&self) {
        self.connected_clients.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Default)]
pub struct ServerStateBuilder {
    forwarding_mode: ForwardingMode,
    dimension: Option<Dimension>,
    description_text: String,
    max_players: u32,
    welcome_message: String,
    show_online_player_count: bool,
    game_mode: GameMode,
    hardcore: bool,
    spawn_position: (f64, f64, f64),
    view_distance: i32,
    schematic_file_path: String,
}

impl ServerStateBuilder {
    pub fn enable_legacy_forwarding(&mut self) -> &mut Self {
        self.forwarding_mode = ForwardingMode::Legacy;
        self
    }

    pub fn enable_bungee_guard_forwarding(&mut self, tokens: Vec<String>) -> &mut Self {
        self.forwarding_mode = ForwardingMode::BungeeGuard { tokens };
        self
    }

    pub fn enable_modern_forwarding<K>(&mut self, key: K) -> &mut Self
    where
        K: Into<Vec<u8>>,
    {
        self.forwarding_mode = ForwardingMode::Modern { secret: key.into() };
        self
    }

    pub fn disable_forwarding(&mut self) -> &mut Self {
        self.forwarding_mode = ForwardingMode::Disabled;
        self
    }

    /// Set the spawn dimension
    pub const fn dimension(&mut self, dimension: Dimension) -> &mut Self {
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

    pub const fn max_players(&mut self, max_players: u32) -> &mut Self {
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

    pub const fn show_online_player_count(&mut self, show: bool) -> &mut Self {
        self.show_online_player_count = show;
        self
    }

    pub const fn game_mode(&mut self, game_mode: GameMode) -> &mut Self {
        self.game_mode = game_mode;
        self
    }

    pub const fn hardcore(&mut self, hardcore: bool) -> &mut Self {
        self.hardcore = hardcore;
        self
    }

    pub const fn spawn_position(&mut self, position: (f64, f64, f64)) -> &mut Self {
        self.spawn_position = position;
        self
    }

    pub fn view_distance(&mut self, view_distance: i32) -> &mut Self {
        self.view_distance = view_distance.clamp(0, 32);
        self
    }

    pub fn schematic(&mut self, schematic_file_path: String) -> &mut Self {
        self.schematic_file_path = schematic_file_path;
        self
    }

    /// Finish building, returning an error if any required fields are missing.
    pub fn build(self) -> ServerState {
        ServerState {
            forwarding_mode: self.forwarding_mode,
            spawn_dimension: self.dimension.unwrap_or_default(),
            description_text: self.description_text,
            max_players: self.max_players,
            welcome_message: self.welcome_message,
            connected_clients: Arc::new(AtomicU32::new(0)),
            show_online_player_count: self.show_online_player_count,
            game_mode: self.game_mode,
            hardcore: self.hardcore,
            spawn_position: self.spawn_position,
            view_distance: self.view_distance,
            schematic_file_path: self.schematic_file_path,
        }
    }
}
