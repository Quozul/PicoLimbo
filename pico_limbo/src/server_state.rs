use crate::server::game_mode::GameMode;
use minecraft_protocol::prelude::{BinaryReaderError, Dimension};
use pico_structures::prelude::{Schematic, SchematicError, World, WorldLoadingError};
use pico_text_component::prelude::{Component, MiniMessageError, parse_mini_message};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use thiserror::Error;
use tracing::debug;

#[derive(PartialEq, Eq, Default)]
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

#[derive(Default)]
pub enum Boundaries {
    #[default]
    Disabled,
    Enabled {
        min_y: i32,
        teleport_message: Option<Component>,
    },
}

#[derive(Default)]
pub enum TabList {
    #[default]
    None,
    Header {
        header: Component,
    },
    Footer {
        footer: Component,
    },
    HeaderAndFooter {
        header: Component,
        footer: Component,
    },
}

#[derive(Default)]
pub struct ServerState {
    forwarding_mode: ForwardingMode,
    spawn_dimension: Dimension,
    motd: Component,
    time_world: i64,
    lock_time: bool,
    max_players: u32,
    welcome_message: Option<Component>,
    connected_clients: Arc<AtomicU32>,
    show_online_player_count: bool,
    game_mode: GameMode,
    hardcore: bool,
    spawn_position: (f64, f64, f64),
    view_distance: i32,
    world: Option<Arc<World>>,
    boundaries: Boundaries,
    tab_list: TabList,
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
        match &self.forwarding_mode {
            ForwardingMode::Modern { secret } => Ok(secret.clone()),
            _ => Err(MisconfiguredForwardingError),
        }
    }

    pub const fn is_bungee_guard_forwarding(&self) -> bool {
        matches!(self.forwarding_mode, ForwardingMode::BungeeGuard { .. })
    }

    pub fn tokens(&self) -> Result<Vec<String>, MisconfiguredForwardingError> {
        match &self.forwarding_mode {
            ForwardingMode::BungeeGuard { tokens } => Ok(tokens.clone()),
            _ => Err(MisconfiguredForwardingError),
        }
    }

    pub const fn motd(&self) -> &Component {
        &self.motd
    }

    pub const fn max_players(&self) -> u32 {
        self.max_players
    }

    pub const fn welcome_message(&self) -> Option<&Component> {
        self.welcome_message.as_ref()
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

    pub fn world(&self) -> Option<Arc<World>> {
        self.world.clone()
    }

    pub const fn time_world_ticks(&self) -> i64 {
        self.time_world
    }

    pub const fn is_time_locked(&self) -> bool {
        self.lock_time
    }

    pub const fn boundaries(&self) -> &Boundaries {
        &self.boundaries
    }
    pub const fn tab_list(&self) -> &TabList {
        &self.tab_list
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
    time_world: i64,
    lock_time: bool,
    description_text: String,
    max_players: u32,
    welcome_message: String,
    show_online_player_count: bool,
    game_mode: GameMode,
    hardcore: bool,
    spawn_position: (f64, f64, f64),
    view_distance: i32,
    schematic_file_path: String,
    boundaries: Boundaries,
    tab_list: TabList,
}

#[derive(Debug, Error)]
pub enum ServerStateBuilderError {
    #[error(transparent)]
    SchematicLoadingFailed(#[from] SchematicError),
    #[error(transparent)]
    BinaryReader(#[from] BinaryReaderError),
    #[error(transparent)]
    WorldLoading(#[from] WorldLoadingError),
    #[error(transparent)]
    MiniMessage(#[from] MiniMessageError),
    #[error("the configured spawn position Y is below the configured minimum Y position")]
    InvalidSpawnPosition(),
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

    /// Set the time of the world
    pub const fn time_world(&mut self, time_world: i64) -> &mut Self {
        self.time_world = time_world;
        self
    }

    pub const fn lock_time(&mut self, lock_time: bool) -> &mut Self {
        self.lock_time = lock_time;
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
        self.view_distance = view_distance.max(0);
        self
    }

    pub fn schematic(&mut self, schematic_file_path: String) -> &mut Self {
        self.schematic_file_path = schematic_file_path;
        self
    }

    pub fn tab_list<S>(
        &mut self,
        header: S,
        footer: S,
    ) -> Result<&mut Self, ServerStateBuilderError>
    where
        S: AsRef<str>,
    {
        self.tab_list = if header.as_ref().is_empty() && footer.as_ref().is_empty() {
            TabList::None
        } else if header.as_ref().is_empty() {
            TabList::Footer {
                footer: parse_mini_message(footer.as_ref())?,
            }
        } else if footer.as_ref().is_empty() {
            TabList::Header {
                header: parse_mini_message(header.as_ref())?,
            }
        } else {
            TabList::HeaderAndFooter {
                header: parse_mini_message(header.as_ref())?,
                footer: parse_mini_message(footer.as_ref())?,
            }
        };

        Ok(self)
    }

    pub fn boundaries<S>(
        &mut self,
        min_y: i32,
        teleport_message: S,
    ) -> Result<&mut Self, ServerStateBuilderError>
    where
        S: AsRef<str>,
    {
        let teleport_message = optional_mini_message(teleport_message.as_ref())?;
        self.boundaries = Boundaries::Enabled {
            min_y,
            teleport_message,
        };
        Ok(self)
    }

    /// Finish building, returning an error if any required fields are missing.
    pub fn build(self) -> Result<ServerState, ServerStateBuilderError> {
        let world = if self.schematic_file_path.is_empty() {
            None
        } else {
            let schematic = time_operation("Loading schematic", || {
                let internal_mapping = blocks_report::load_internal_mapping()?;
                let schematic_file_path = PathBuf::from(self.schematic_file_path);
                Schematic::load_schematic_file(&schematic_file_path, &internal_mapping)
            })?;
            let world = time_operation("Loading world", || World::from_schematic(&schematic))?;
            Some(Arc::new(world))
        };
        Ok(ServerState {
            forwarding_mode: self.forwarding_mode,
            spawn_dimension: self.dimension.unwrap_or_default(),
            motd: parse_mini_message(&self.description_text)?,
            time_world: self.time_world,
            lock_time: self.lock_time,
            max_players: self.max_players,
            welcome_message: optional_mini_message(&self.welcome_message)?,
            connected_clients: Arc::new(AtomicU32::new(0)),
            show_online_player_count: self.show_online_player_count,
            game_mode: self.game_mode,
            hardcore: self.hardcore,
            spawn_position: self.spawn_position,
            view_distance: self.view_distance,
            world,
            boundaries: self.boundaries,
            tab_list: self.tab_list,
        })
    }
}

fn optional_mini_message(content: &str) -> Result<Option<Component>, MiniMessageError> {
    let component = if content.is_empty() {
        None
    } else {
        Some(parse_mini_message(content)?)
    };
    Ok(component)
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs_f64();

    if total_secs >= 1.0 {
        format!("{total_secs:.1}s")
    } else {
        format!("{}ms", duration.as_millis())
    }
}

fn time_operation<T, F>(operation_name: &str, operation: F) -> T
where
    F: FnOnce() -> T,
{
    debug!("{operation_name}...");
    let start = std::time::Instant::now();
    let result = operation();
    let elapsed = start.elapsed();
    debug!("Time elapsed: {}", format_duration(elapsed));
    result
}
