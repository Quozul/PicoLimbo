//! `PicoLimbo`'s embedded plugin runtime.
//!
//! Plugins run through capability-gated script runtime adapters.
//! No filesystem, network, process, or dynamic module APIs are ambient by default.

#![allow(
    dead_code,
    clippy::too_many_lines,
    clippy::struct_excessive_bools,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_lossless,
    clippy::type_complexity,
    clippy::map_unwrap_or,
    clippy::collapsible_if,
    clippy::redundant_closure,
    clippy::match_wildcard_for_single_variants,
    clippy::unnecessary_wraps
)]
use rhai::{AST, Array, Dynamic, Engine, Map, Scope};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path};
#[cfg(feature = "plugin-lua")]
use std::sync::Arc;
#[cfg(feature = "plugin-lua")]
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{error, info, warn};

const PLUGIN_API_VERSION: u32 = 1;
const MAX_MANIFEST_BYTES: u64 = 16 * 1024;
const MAX_SCRIPT_BYTES: u64 = 256 * 1024;
const MAX_OPERATIONS: u64 = 100_000;
const MAX_STRING_BYTES: usize = 16 * 1024;
const MAX_COLLECTION_ITEMS: usize = 256;
/// Hard caps to keep plugin host memory predictable (target: server stays << 1 GiB).
const MAX_SCHEDULED_TASKS: usize = 256;
const MAX_SERVICE_ENTRIES: usize = 512;
const MAX_SERVICE_VALUE_BYTES: usize = 4 * 1024;
const MAX_CLIENT_ACTIONS_PER_PLAYER: usize = 16;
const MAX_KICK_QUEUE: usize = 128;
const MAX_WORLD_OVERLAY_BLOCKS: usize = 2048;
const MAX_DIRTY_CHUNKS: usize = 64;
/// High-frequency move packets are filtered unless this is true for a plugin path.
const PACKET_MOVE_PREFIXES: &[&str] = &[
    "packet:minecraft:move_player_pos",
    "packet:minecraft:move_player_pos_rot",
];

/// Process-local service map so script plugins can share named string values.
fn shared_services() -> &'static std::sync::Mutex<std::collections::HashMap<String, String>> {
    static SERVICES: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, String>>,
    > = std::sync::OnceLock::new();
    SERVICES.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn shared_online_players() -> &'static std::sync::atomic::AtomicU32 {
    static COUNT: std::sync::OnceLock<std::sync::atomic::AtomicU32> = std::sync::OnceLock::new();
    COUNT.get_or_init(|| std::sync::atomic::AtomicU32::new(0))
}

fn shared_spawn_position() -> &'static std::sync::Mutex<(f64, f64, f64)> {
    static SPAWN: std::sync::OnceLock<std::sync::Mutex<(f64, f64, f64)>> =
        std::sync::OnceLock::new();
    SPAWN.get_or_init(|| std::sync::Mutex::new((0.5, 64.0, 0.5)))
}

fn shared_has_world() -> &'static std::sync::atomic::AtomicBool {
    static FLAG: std::sync::OnceLock<std::sync::atomic::AtomicBool> = std::sync::OnceLock::new();
    FLAG.get_or_init(|| std::sync::atomic::AtomicBool::new(false))
}

/// Sparse runtime block overlay (internal ids). Capped for memory.
fn shared_world_overlay()
-> &'static std::sync::Mutex<std::collections::HashMap<(i32, i32, i32), u16>> {
    static OVERLAY: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<(i32, i32, i32), u16>>,
    > = std::sync::OnceLock::new();
    OVERLAY.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

fn shared_dirty_chunks() -> &'static std::sync::Mutex<std::collections::HashSet<(i32, i32)>> {
    static DIRTY: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<(i32, i32)>>> =
        std::sync::OnceLock::new();
    DIRTY.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

fn shared_online_names() -> &'static std::sync::Mutex<std::collections::HashSet<String>> {
    static NAMES: std::sync::OnceLock<std::sync::Mutex<std::collections::HashSet<String>>> =
        std::sync::OnceLock::new();
    NAMES.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()))
}

fn resolve_internal_id(block_state: &str) -> Option<u16> {
    let mapping = blocks_report::load_internal_mapping().ok()?;
    let lookup = blocks_report::BlockStateLookup::new(&mapping);
    lookup
        .parse_state_string(block_state)
        .ok()
        .map(|state| state.internal_id())
}

#[derive(Clone)]
struct ScheduledTask {
    plugin_id: String,
    function_name: String,
    run_at: Instant,
    interval: Option<Duration>,
}

fn shared_schedule() -> &'static std::sync::Mutex<Vec<ScheduledTask>> {
    static TASKS: std::sync::OnceLock<std::sync::Mutex<Vec<ScheduledTask>>> =
        std::sync::OnceLock::new();
    TASKS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

fn shared_kick_requests() -> &'static std::sync::Mutex<std::collections::HashMap<String, String>> {
    static KICKS: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, String>>> =
        std::sync::OnceLock::new();
    KICKS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

#[derive(Clone, Debug)]
pub enum PluginClientAction {
    Title(String),
    ActionBar(String),
    TeleportSpawn,
    /// Protocol block state id already mapped for a specific protocol is resolved at send time.
    BlockUpdate {
        x: i32,
        y: i32,
        z: i32,
        internal_id: u16,
    },
    /// Request void-style chunk column refresh (clients still need block updates for overlay).
    ResendChunk {
        chunk_x: i32,
        chunk_z: i32,
    },
}

fn shared_client_actions()
-> &'static std::sync::Mutex<std::collections::HashMap<String, Vec<PluginClientAction>>> {
    static ACTIONS: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<String, Vec<PluginClientAction>>>,
    > = std::sync::OnceLock::new();
    ACTIONS.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PluginManifest {
    id: String,
    name: String,
    version: String,
    api_version: u32,
    #[serde(default = "default_runtime")]
    runtime: String,
    #[serde(default)]
    entrypoint: Option<String>,
    #[serde(default)]
    commands: Vec<String>,
    #[serde(default)]
    permissions: Vec<String>,
    /// Hard dependencies: plugin IDs that must load successfully before this plugin.
    #[serde(default)]
    depends_on: Vec<String>,
    /// Soft dependencies: preferred load-before IDs; missing ones are ignored.
    #[serde(default)]
    soft_depends: Vec<String>,
    /// Optional human-readable command descriptions keyed by command name.
    #[serde(default)]
    command_descriptions: std::collections::HashMap<String, String>,
}

fn default_runtime() -> String {
    "rhai".to_owned()
}

fn default_entrypoint(runtime: &str) -> &'static str {
    match runtime {
        "lua" => "main.lua",
        _ => "main.rhai",
    }
}

#[derive(Debug, Error)]
enum PluginLoadError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid manifest: {0}")]
    Manifest(String),
    #[error("plugin API version {found} is incompatible (server supports {supported})")]
    ApiVersion { found: u32, supported: u32 },
    #[error("plugin manifest must remain inside its plugin directory")]
    InvalidManifestPath,
    #[error("plugin runtime '{0}' is not available in this build; enable its Cargo feature")]
    RuntimeUnavailable(String),
    #[error("plugin runtime initialization failed: {0}")]
    Runtime(String),
    #[error("plugin entrypoint must be a relative path inside its plugin directory")]
    InvalidEntrypoint,
    #[error("plugin script exceeds the 262144-byte limit")]
    ScriptTooLarge,
    #[error("plugin manifest exceeds the 16384-byte limit")]
    ManifestTooLarge,
    #[error("Rhai compilation failed: {0}")]
    Compile(String),
    #[error("plugin script must define `{name}` with {parameters} parameter(s)")]
    MissingFunction {
        name: &'static str,
        parameters: usize,
    },
    #[error("plugin on_enable() failed: {0}")]
    Enable(String),
}

struct LoadedPlugin {
    id: String,
    name: String,
    version: String,
    commands: HashSet<String>,
    command_descriptions: std::collections::HashMap<String, String>,
    permissions: PluginPermissions,
    hooks: PluginHooks,
    runtime: PluginRuntime,
    event_priority: EventPriority,
    depends_on: Vec<String>,
    soft_depends: Vec<String>,
    enabled: bool,
}

#[derive(Default)]
struct PluginPermissions {
    player_read: bool,
    player_message: bool,
    player_kick: bool,
    server_info: bool,
    services: bool,
    scheduler: bool,
    packets_inbound: bool,
    player_effects: bool,
    world_mutate: bool,
}

struct PluginHooks {
    on_event: bool,
    on_disable: bool,
}

/// Dispatch priority for plugin event handlers (higher runs first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventPriority {
    Monitor = 5,
    Highest = 4,
    High = 3,
    Normal = 2,
    Low = 1,
    Lowest = 0,
}

#[derive(Default)]
struct ServiceRegistry {
    services: std::collections::HashMap<String, String>,
}

impl ServiceRegistry {
    fn register(&mut self, name: String, value: String) {
        self.services.insert(name, value);
    }

    fn get(&self, name: &str) -> Option<String> {
        self.services.get(name).cloned()
    }
}

enum PluginRuntime {
    Rhai {
        engine: Box<Engine>,
        ast: AST,
        scope: Scope<'static>,
    },
    #[cfg(feature = "plugin-lua")]
    Lua {
        lua: mlua::Lua,
        operation_budget: Arc<AtomicU64>,
    },
}

impl PluginRuntime {
    fn on_enable(&mut self) -> Result<(), String> {
        match self {
            Self::Rhai { engine, ast, scope } => engine
                .call_fn::<()>(scope, ast, "on_enable", ())
                .map_err(|error| error.to_string()),
            #[cfg(feature = "plugin-lua")]
            Self::Lua {
                lua,
                operation_budget,
            } => {
                operation_budget.store(0, Ordering::Relaxed);
                let function = lua
                    .globals()
                    .get::<mlua::Function>("on_enable")
                    .map_err(|error| error.to_string())?;
                function.call::<()>(()).map_err(|error| error.to_string())
            }
        }
    }

    fn on_command(
        &mut self,
        command: &str,
        player: &str,
        args: &[String],
    ) -> Result<String, String> {
        match self {
            Self::Rhai { engine, ast, scope } => engine
                .call_fn::<String>(
                    scope,
                    ast,
                    "on_command",
                    (
                        command.to_owned(),
                        player.to_owned(),
                        args.iter().cloned().map(Dynamic::from).collect::<Array>(),
                    ),
                )
                .map_err(|error| error.to_string()),
            #[cfg(feature = "plugin-lua")]
            Self::Lua {
                lua,
                operation_budget,
            } => {
                operation_budget.store(0, Ordering::Relaxed);
                let function = lua
                    .globals()
                    .get::<mlua::Function>("on_command")
                    .map_err(|error| error.to_string())?;
                let lua_args = lua
                    .create_sequence_from(args.iter().cloned())
                    .map_err(|error| error.to_string())?;
                function
                    .call::<String>((command, player, lua_args))
                    .map_err(|error| error.to_string())
            }
        }
    }

    /// Returns Ok(true) when the handler cancelled the event.
    fn on_event(&mut self, event: &str, player: Option<(&str, &str)>) -> Result<bool, String> {
        match self {
            Self::Rhai { engine, ast, scope } => {
                let mut payload = Map::new();
                payload.insert("event".into(), Dynamic::from(event.to_owned()));
                if let Some((name, uuid)) = player {
                    payload.insert("player".into(), Dynamic::from(name.to_owned()));
                    payload.insert("uuid".into(), Dynamic::from(uuid.to_owned()));
                }
                match engine.call_fn::<Dynamic>(scope, ast, "on_event", (payload,)) {
                    Ok(value) => Ok(value.as_bool().unwrap_or(false)),
                    Err(error) => {
                        // Backward compatible: void handlers that return nothing.
                        let message = error.to_string();
                        if message.contains("Function not found") {
                            return Err(message);
                        }
                        // Rhai unit return often fails bool cast; treat non-bool success as not cancelled.
                        // Re-run is expensive; interpret common unit-return error as Ok(false).
                        if message.contains("Output type") || message.contains("cast") {
                            // Prefer best-effort non-cancel when return type mismatches.
                            Ok(false)
                        } else {
                            // Try calling as () for legacy scripts.
                            engine
                                .call_fn::<()>(
                                    scope,
                                    ast,
                                    "on_event",
                                    ({
                                        let mut payload = Map::new();
                                        payload.insert(
                                            "event".into(),
                                            Dynamic::from(event.to_owned()),
                                        );
                                        if let Some((name, uuid)) = player {
                                            payload.insert(
                                                "player".into(),
                                                Dynamic::from(name.to_owned()),
                                            );
                                            payload.insert(
                                                "uuid".into(),
                                                Dynamic::from(uuid.to_owned()),
                                            );
                                        }
                                        payload
                                    },),
                                )
                                .map(|_| false)
                                .map_err(|error| error.to_string())
                        }
                    }
                }
            }
            #[cfg(feature = "plugin-lua")]
            Self::Lua {
                lua,
                operation_budget,
            } => {
                operation_budget.store(0, Ordering::Relaxed);
                let function = lua
                    .globals()
                    .get::<mlua::Function>("on_event")
                    .map_err(|error| error.to_string())?;
                let payload = lua.create_table().map_err(|error| error.to_string())?;
                payload
                    .set("event", event)
                    .map_err(|error| error.to_string())?;
                if let Some((name, uuid)) = player {
                    payload
                        .set("player", name)
                        .map_err(|error| error.to_string())?;
                    payload
                        .set("uuid", uuid)
                        .map_err(|error| error.to_string())?;
                }
                match function.call::<Option<bool>>(payload) {
                    Ok(Some(cancelled)) => Ok(cancelled),
                    Ok(None) => Ok(false),
                    Err(error) => {
                        // Legacy void handlers.
                        let message = error.to_string();
                        if message.contains("nil") || message.contains("convert") {
                            Ok(false)
                        } else {
                            Err(message)
                        }
                    }
                }
            }
        }
    }

    fn on_disable(&mut self) -> Result<(), String> {
        match self {
            Self::Rhai { engine, ast, .. } => engine
                .call_fn::<()>(&mut Scope::new(), ast, "on_disable", ())
                .map_err(|error| error.to_string()),
            #[cfg(feature = "plugin-lua")]
            Self::Lua {
                lua,
                operation_budget,
            } => {
                operation_budget.store(0, Ordering::Relaxed);
                let function = lua
                    .globals()
                    .get::<mlua::Function>("on_disable")
                    .map_err(|error| error.to_string())?;
                function.call::<()>(()).map_err(|error| error.to_string())
            }
        }
    }

    fn call_named(&mut self, name: &str) -> Result<(), String> {
        match self {
            Self::Rhai { engine, ast, scope } => engine
                .call_fn::<()>(scope, ast, name, ())
                .map_err(|error| error.to_string()),
            #[cfg(feature = "plugin-lua")]
            Self::Lua {
                lua,
                operation_budget,
            } => {
                operation_budget.store(0, Ordering::Relaxed);
                let function = lua
                    .globals()
                    .get::<mlua::Function>(name)
                    .map_err(|error| error.to_string())?;
                function.call::<()>(()).map_err(|error| error.to_string())
            }
        }
    }
}

/// Owns the scripts loaded by one `PicoLimbo` server instance.
#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
    services: ServiceRegistry,
}

impl PluginManager {
    /// Create the plugin directory and load valid plugin subdirectories.
    /// A broken plugin is isolated: it is logged and skipped without stopping the server.
    pub fn load(directory: &Path) -> Result<Self, std::io::Error> {
        fs::create_dir_all(directory)?;
        let mut entries = fs::read_dir(directory)?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
            .map(|entry| entry.path())
            .collect::<Vec<_>>();
        entries.sort();

        let mut manager = Self::default();
        let mut registered_commands = HashSet::new();
        for path in entries {
            match Self::load_plugin(&path) {
                Ok(plugin) => {
                    if let Some(command) = plugin
                        .commands
                        .iter()
                        .find(|command| registered_commands.contains(*command))
                    {
                        warn!(
                            plugin = %plugin.id,
                            command = %command,
                            "Skipping plugin because its command is already registered"
                        );
                        continue;
                    }
                    registered_commands.extend(plugin.commands.iter().cloned());
                    info!(
                        plugin = %plugin.id,
                        name = %plugin.name,
                        version = %plugin.version,
                        "Loaded PicoLimbo plugin"
                    );
                    manager.plugins.push(plugin);
                }
                Err(err) => warn!(
                    plugin_path = %path.display(),
                    error = %err,
                    "Skipping PicoLimbo plugin"
                ),
            }
        }

        // Drop plugins whose hard dependencies are missing, then stable-sort so
        // dependency-free plugins come first (soft + hard deps influence order).
        let available: HashSet<String> = manager.plugins.iter().map(|p| p.id.clone()).collect();
        manager.plugins.retain(|plugin| {
            let missing: Vec<_> = plugin
                .depends_on
                .iter()
                .filter(|dep| !available.contains(dep.as_str()))
                .cloned()
                .collect();
            if missing.is_empty() {
                true
            } else {
                warn!(
                    plugin = %plugin.id,
                    missing = ?missing,
                    "Skipping plugin because hard dependencies are missing"
                );
                false
            }
        });
        manager.plugins.sort_by(|a, b| {
            let a_deps = a.depends_on.len() + a.soft_depends.len();
            let b_deps = b.depends_on.len() + b.soft_depends.len();
            a_deps.cmp(&b_deps).then_with(|| a.id.cmp(&b.id))
        });

        Ok(manager)
    }

    fn load_plugin(directory: &Path) -> Result<LoadedPlugin, PluginLoadError> {
        let canonical_directory = directory.canonicalize()?;
        let manifest_path = canonical_directory.join("plugin.toml").canonicalize()?;
        if !manifest_path.starts_with(&canonical_directory) {
            return Err(PluginLoadError::InvalidManifestPath);
        }
        let manifest_metadata = fs::metadata(&manifest_path)?;
        if manifest_metadata.len() > MAX_MANIFEST_BYTES {
            return Err(PluginLoadError::ManifestTooLarge);
        }

        let manifest_text = fs::read_to_string(&manifest_path)?;
        let manifest: PluginManifest = toml::from_str(&manifest_text)
            .map_err(|err| PluginLoadError::Manifest(err.to_string()))?;
        validate_manifest(&manifest, directory)?;

        let entrypoint_name = manifest
            .entrypoint
            .as_deref()
            .unwrap_or_else(|| default_entrypoint(&manifest.runtime));
        let entrypoint = Path::new(entrypoint_name);
        if entrypoint.is_absolute()
            || entrypoint.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(PluginLoadError::InvalidEntrypoint);
        }

        let script_path = canonical_directory.join(entrypoint).canonicalize()?;
        if !script_path.starts_with(&canonical_directory) {
            return Err(PluginLoadError::InvalidEntrypoint);
        }
        let metadata = fs::metadata(&script_path)?;
        if metadata.len() > MAX_SCRIPT_BYTES {
            return Err(PluginLoadError::ScriptTooLarge);
        }

        let script = fs::read_to_string(&script_path)?;
        let id = manifest.id.clone();
        let (mut runtime, has_on_event, has_on_disable) = build_plugin_runtime(
            &manifest.runtime,
            &id,
            &manifest.permissions,
            &manifest.commands,
            &script,
        )?;

        runtime.on_enable().map_err(PluginLoadError::Enable)?;

        Ok(LoadedPlugin {
            id,
            name: manifest.name,
            version: manifest.version,
            commands: manifest.commands.into_iter().collect(),
            permissions: PluginPermissions {
                player_read: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "player.read"),
                player_message: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "player.message"),
                server_info: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "server.info"),
                services: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "services"),
                scheduler: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "scheduler"),
                player_kick: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "player.kick"),
                packets_inbound: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "packets.inbound"),
                player_effects: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "player.effects"),
                world_mutate: manifest
                    .permissions
                    .iter()
                    .any(|permission| permission == "world.mutate"),
            },
            hooks: PluginHooks {
                on_event: has_on_event,
                on_disable: has_on_disable,
            },
            runtime,
            event_priority: EventPriority::Normal,
            depends_on: manifest.depends_on,
            soft_depends: manifest.soft_depends,
            command_descriptions: manifest.command_descriptions,
            enabled: true,
        })
    }

    /// Dispatch a slash command to the plugin that declared its name.
    /// Returns a response only when a plugin handled the command and has player.message permission.
    pub fn execute_command(&mut self, command_line: &str, player_name: &str) -> Option<String> {
        let mut parts = command_line.split_whitespace();
        let command = parts.next()?.trim_start_matches('/').to_ascii_lowercase();
        let args = parts.map(str::to_owned).collect::<Vec<_>>();

        for plugin in &mut self.plugins {
            if !plugin.enabled || !plugin.commands.contains(&command) {
                continue;
            }

            let player = if plugin.permissions.player_read {
                player_name.to_owned()
            } else {
                String::new()
            };

            match plugin.runtime.on_command(&command, &player, &args) {
                Ok(response) => {
                    if plugin.permissions.player_message && !response.is_empty() {
                        return Some(response);
                    }
                    return Some(String::new());
                }
                Err(err) => {
                    error!(
                        plugin = %plugin.id,
                        error = %err,
                        "Disabling PicoLimbo plugin after command handler failure"
                    );
                    plugin.enabled = false;
                }
            }
        }

        None
    }

    /// Dispatch a server event to every enabled plugin with an `on_event(event)` function.
    /// Dispatch a player-scoped event. Returns `true` if any handler cancelled it.
    pub fn emit_player_event(&mut self, event: &str, player_name: &str, player_uuid: &str) -> bool {
        // Deterministic order: higher EventPriority first, then plugin id.
        let mut order: Vec<usize> = (0..self.plugins.len()).collect();
        order.sort_by(|&a, &b| {
            self.plugins[b]
                .event_priority
                .cmp(&self.plugins[a].event_priority)
                .then_with(|| self.plugins[a].id.cmp(&self.plugins[b].id))
        });

        let is_packet_event = event.starts_with("packet:");
        let is_move_packet = PACKET_MOVE_PREFIXES
            .iter()
            .any(|prefix| event.starts_with(prefix));

        let mut cancelled = false;
        for index in order {
            let plugin = &mut self.plugins[index];
            if !plugin.enabled || !plugin.hooks.on_event {
                continue;
            }
            // Packet hooks are opt-in per plugin — avoids waking every script on each packet.
            if is_packet_event && !plugin.permissions.packets_inbound {
                continue;
            }
            // Movement packets are high-frequency; only deliver if the plugin opted into packets.
            // (Already gated above; kept explicit for clarity.)
            let _ = is_move_packet;

            // Monitor priority observes only; cannot cancel.
            let can_cancel = plugin.event_priority != EventPriority::Monitor;
            let player = plugin
                .permissions
                .player_read
                .then_some((player_name, player_uuid));
            match plugin.runtime.on_event(event, player) {
                Ok(true) if can_cancel => {
                    cancelled = true;
                }
                Ok(_) => {}
                Err(err) => {
                    warn!(
                        plugin = %plugin.id,
                        error = %err,
                        "PicoLimbo plugin event handler failed; disabling plugin"
                    );
                    plugin.enabled = false;
                }
            }
        }
        cancelled
    }

    /// Take a pending plugin kick reason for `player_name`, if any.
    pub fn take_kick_request(player_name: &str) -> Option<String> {
        shared_kick_requests()
            .lock()
            .ok()
            .and_then(|mut map| map.remove(player_name))
    }

    pub fn request_kick(player_name: impl Into<String>, reason: impl Into<String>) {
        let player_name = player_name.into();
        if let Ok(mut map) = shared_kick_requests().lock() {
            if map.len() >= MAX_KICK_QUEUE && !map.contains_key(&player_name) {
                return;
            }
            let reason = reason.into();
            let reason = if reason.len() > 512 {
                reason.chars().take(512).collect()
            } else {
                reason
            };
            map.insert(player_name, reason);
        }
    }

    pub fn wants_inbound_packets(&self) -> bool {
        self.plugins.iter().any(|plugin| {
            plugin.enabled && plugin.hooks.on_event && plugin.permissions.packets_inbound
        })
    }

    pub fn queue_client_action(player_name: impl Into<String>, action: PluginClientAction) {
        if let Ok(mut map) = shared_client_actions().lock() {
            let entry = map.entry(player_name.into()).or_default();
            if entry.len() < MAX_CLIENT_ACTIONS_PER_PLAYER {
                entry.push(action);
            }
        }
    }

    pub fn take_client_actions(player_name: &str) -> Vec<PluginClientAction> {
        shared_client_actions()
            .lock()
            .ok()
            .and_then(|mut map| map.remove(player_name))
            .unwrap_or_default()
    }

    pub fn note_player_online(name: &str) {
        if name.is_empty() {
            return;
        }
        if let Ok(mut set) = shared_online_names().lock() {
            set.insert(name.to_owned());
        }
    }

    pub fn note_player_offline(name: &str) {
        if let Ok(mut set) = shared_online_names().lock() {
            set.remove(name);
        }
    }

    /// Sparse mutator: stores internal block id and fans out block-update actions.
    pub fn world_set_block(x: i32, y: i32, z: i32, block_state: &str) -> bool {
        let Some(internal_id) = resolve_internal_id(block_state) else {
            return false;
        };
        if let Ok(mut overlay) = shared_world_overlay().lock() {
            if overlay.len() >= MAX_WORLD_OVERLAY_BLOCKS && !overlay.contains_key(&(x, y, z)) {
                return false;
            }
            overlay.insert((x, y, z), internal_id);
        } else {
            return false;
        }
        let chunk = (x >> 4, z >> 4);
        if let Ok(mut dirty) = shared_dirty_chunks().lock() {
            if dirty.len() < MAX_DIRTY_CHUNKS {
                dirty.insert(chunk);
            }
        }
        let action = PluginClientAction::BlockUpdate {
            x,
            y,
            z,
            internal_id,
        };
        if let Ok(names) = shared_online_names().lock() {
            for name in names.iter() {
                Self::queue_client_action(name.clone(), action.clone());
            }
        }
        true
    }

    pub fn world_get_block(x: i32, y: i32, z: i32) -> i64 {
        shared_world_overlay()
            .lock()
            .ok()
            .and_then(|o| o.get(&(x, y, z)).copied())
            .map(i64::from)
            .unwrap_or(-1)
    }

    pub fn world_clear_overlay() {
        if let Ok(mut o) = shared_world_overlay().lock() {
            o.clear();
        }
        if let Ok(mut d) = shared_dirty_chunks().lock() {
            d.clear();
        }
    }

    pub fn world_resend_chunk(chunk_x: i32, chunk_z: i32) {
        let action = PluginClientAction::ResendChunk { chunk_x, chunk_z };
        if let Ok(names) = shared_online_names().lock() {
            for name in names.iter() {
                Self::queue_client_action(name.clone(), action.clone());
            }
        }
    }

    pub fn overlay_snapshot() -> Vec<((i32, i32, i32), u16)> {
        shared_world_overlay()
            .lock()
            .ok()
            .map(|o| o.iter().map(|(k, v)| (*k, *v)).collect())
            .unwrap_or_default()
    }

    pub fn register_service(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.services.register(name.into(), value.into());
    }

    pub fn get_service(&self, name: &str) -> Option<String> {
        self.services.get(name)
    }

    pub fn plugin_summaries(&self) -> Vec<(String, String, String)> {
        self.plugins
            .iter()
            .filter(|plugin| plugin.enabled)
            .map(|plugin| {
                (
                    plugin.id.clone(),
                    plugin.name.clone(),
                    plugin.version.clone(),
                )
            })
            .collect()
    }

    /// Update the process-local online player counter used by `server.info` helpers.
    pub fn set_online_players(count: u32) {
        shared_online_players().store(count, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn set_world_meta(has_world: bool, spawn: (f64, f64, f64)) {
        shared_has_world().store(has_world, std::sync::atomic::Ordering::SeqCst);
        if let Ok(mut guard) = shared_spawn_position().lock() {
            *guard = spawn;
        }
    }

    /// Run due scheduled tasks. Safe to call frequently (e.g. keep-alive path).
    pub fn tick(&mut self) {
        let now = Instant::now();
        let Ok(mut tasks) = shared_schedule().lock() else {
            return;
        };
        let mut due = Vec::new();
        let mut remaining = Vec::new();
        for task in tasks.drain(..) {
            if task.run_at <= now {
                due.push(task);
            } else {
                remaining.push(task);
            }
        }
        *tasks = remaining;
        drop(tasks);

        for task in due {
            let Some(plugin) = self
                .plugins
                .iter_mut()
                .find(|plugin| plugin.enabled && plugin.id == task.plugin_id)
            else {
                continue;
            };
            if let Err(err) = plugin.runtime.call_named(&task.function_name) {
                warn!(
                    plugin = %plugin.id,
                    function = %task.function_name,
                    error = %err,
                    "Scheduled plugin callback failed; disabling plugin"
                );
                plugin.enabled = false;
                continue;
            }
            if let Some(interval) = task.interval {
                if let Ok(mut tasks) = shared_schedule().lock() {
                    if tasks.len() < MAX_SCHEDULED_TASKS {
                        tasks.push(ScheduledTask {
                            plugin_id: task.plugin_id,
                            function_name: task.function_name,
                            run_at: Instant::now() + interval,
                            interval: Some(interval),
                        });
                    }
                }
            }
        }
    }

    /// Command names registered by enabled plugins.
    pub fn command_names(&self) -> Vec<String> {
        let mut commands = self
            .plugins
            .iter()
            .filter(|plugin| plugin.enabled)
            .flat_map(|plugin| plugin.commands.iter().cloned())
            .collect::<Vec<_>>();
        commands.sort();
        commands
    }

    /// Number of successfully loaded plugins.
    pub const fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Whether no plugins were loaded.
    pub const fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Drop for PluginManager {
    fn drop(&mut self) {
        for plugin in &mut self.plugins {
            if plugin.enabled
                && plugin.hooks.on_disable
                && let Err(err) = plugin.runtime.on_disable()
            {
                warn!(plugin = %plugin.id, error = %err, "Plugin shutdown hook failed");
            }
        }
    }
}

fn build_plugin_runtime(
    runtime: &str,
    id: &str,
    permissions: &[String],
    commands: &[String],
    script: &str,
) -> Result<(PluginRuntime, bool, bool), PluginLoadError> {
    match runtime {
        "rhai" => build_rhai_runtime(id, permissions, commands, script),
        "lua" => build_lua_runtime(id, permissions, commands, script),
        other => Err(PluginLoadError::RuntimeUnavailable(other.to_owned())),
    }
}

fn build_rhai_runtime(
    id: &str,
    permissions: &[String],
    commands: &[String],
    script: &str,
) -> Result<(PluginRuntime, bool, bool), PluginLoadError> {
    let engine = create_engine(id, permissions);
    let ast = engine
        .compile(script)
        .map_err(|error| PluginLoadError::Compile(error.to_string()))?;
    let has_function = |name: &str, parameters: usize| {
        ast.iter_functions()
            .any(|function| function.name == name && function.params.len() == parameters)
    };
    let declares_function = |name: &str| ast.iter_functions().any(|function| function.name == name);

    for (name, parameters, required) in [
        ("on_enable", 0, true),
        ("on_command", 3, !commands.is_empty()),
        ("on_event", 1, false),
        ("on_disable", 0, false),
    ] {
        if declares_function(name) && !has_function(name, parameters)
            || required && !has_function(name, parameters)
        {
            return Err(PluginLoadError::MissingFunction { name, parameters });
        }
    }

    let has_on_event = has_function("on_event", 1);
    let has_on_disable = has_function("on_disable", 0);
    let mut scope = Scope::new();
    scope.push("state", Map::new());

    Ok((
        PluginRuntime::Rhai {
            engine: Box::new(engine),
            ast,
            scope,
        },
        has_on_event,
        has_on_disable,
    ))
}

#[cfg(feature = "plugin-lua")]
fn build_lua_runtime(
    id: &str,
    permissions: &[String],
    commands: &[String],
    script: &str,
) -> Result<(PluginRuntime, bool, bool), PluginLoadError> {
    use mlua::{HookTriggers, Lua, LuaOptions, StdLib, VmState};

    let lua = Lua::new_with(StdLib::ALL_SAFE, LuaOptions::default())
        .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
    lua.set_memory_limit(16 * 1024 * 1024)
        .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
    let operation_budget = Arc::new(AtomicU64::new(0));
    let budget = Arc::clone(&operation_budget);
    lua.set_hook(
        HookTriggers::new().every_nth_instruction(1000),
        move |_, _| {
            let used = budget.fetch_add(1000, Ordering::Relaxed) + 1000;
            if used > MAX_OPERATIONS {
                return Err(mlua::Error::RuntimeError(
                    "plugin instruction budget exceeded".into(),
                ));
            }
            Ok(VmState::Continue)
        },
    )
    .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;

    if permissions.iter().any(|permission| permission == "logging") {
        register_lua_logger(&lua, id)?;
    }

    if permissions
        .iter()
        .any(|permission| permission == "services")
    {
        let plugin_id = id.to_owned();
        lua.globals()
            .set(
                "service_register",
                lua.create_function(move |_, (name, value): (String, String)| {
                    if value.len() > MAX_SERVICE_VALUE_BYTES {
                        return Ok(());
                    }
                    let key = format!("{plugin_id}:{name}");
                    if let Ok(mut guard) = shared_services().lock() {
                        if guard.len() >= MAX_SERVICE_ENTRIES && !guard.contains_key(&key) {
                            return Ok(());
                        }
                        guard.insert(key, value);
                    }
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
        lua.globals()
            .set(
                "service_get",
                lua.create_function(|_, name: String| {
                    Ok(shared_services()
                        .lock()
                        .ok()
                        .and_then(|guard| guard.get(&name).cloned())
                        .unwrap_or_default())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
    }

    if permissions
        .iter()
        .any(|permission| permission == "server.info")
    {
        lua.globals()
            .set(
                "server_api_version",
                lua.create_function(|_, ()| Ok(PLUGIN_API_VERSION as i64))
                    .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
        lua.globals()
            .set(
                "server_online_players",
                lua.create_function(|_, ()| {
                    Ok(shared_online_players().load(std::sync::atomic::Ordering::SeqCst) as i64)
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
    }

    if permissions
        .iter()
        .any(|permission| permission == "player.kick")
    {
        lua.globals()
            .set(
                "player_kick",
                lua.create_function(|_, (player_name, reason): (String, String)| {
                    PluginManager::request_kick(&player_name, &reason);
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
    }

    if permissions
        .iter()
        .any(|permission| permission == "world.mutate")
    {
        lua.globals()
            .set(
                "world_set_block",
                lua.create_function(|_, (x, y, z, block_state): (i64, i64, i64, String)| {
                    Ok(PluginManager::world_set_block(
                        x as i32,
                        y as i32,
                        z as i32,
                        &block_state,
                    ))
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
        lua.globals()
            .set(
                "world_get_block",
                lua.create_function(|_, (x, y, z): (i64, i64, i64)| {
                    Ok(PluginManager::world_get_block(x as i32, y as i32, z as i32))
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
        lua.globals()
            .set(
                "world_clear_overlay",
                lua.create_function(|_, ()| {
                    PluginManager::world_clear_overlay();
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
        lua.globals()
            .set(
                "world_resend_chunk",
                lua.create_function(|_, (chunk_x, chunk_z): (i64, i64)| {
                    PluginManager::world_resend_chunk(chunk_x as i32, chunk_z as i32);
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
    }

    if permissions
        .iter()
        .any(|permission| permission == "player.effects")
    {
        lua.globals()
            .set(
                "player_title",
                lua.create_function(|_, (player_name, text): (String, String)| {
                    PluginManager::queue_client_action(
                        &player_name,
                        PluginClientAction::Title(text),
                    );
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
        lua.globals()
            .set(
                "player_actionbar",
                lua.create_function(|_, (player_name, text): (String, String)| {
                    PluginManager::queue_client_action(
                        &player_name,
                        PluginClientAction::ActionBar(text),
                    );
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
        lua.globals()
            .set(
                "player_teleport_spawn",
                lua.create_function(|_, player_name: String| {
                    PluginManager::queue_client_action(
                        &player_name,
                        PluginClientAction::TeleportSpawn,
                    );
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
    }

    if permissions
        .iter()
        .any(|permission| permission == "scheduler")
    {
        let plugin_id = id.to_owned();
        lua.globals()
            .set(
                "schedule_later",
                lua.create_function(move |_, (delay_ms, function_name): (i64, String)| {
                    let delay_ms = delay_ms.max(0) as u64;
                    if let Ok(mut tasks) = shared_schedule().lock() {
                        if tasks.len() < MAX_SCHEDULED_TASKS {
                            tasks.push(ScheduledTask {
                                plugin_id: plugin_id.clone(),
                                function_name,
                                run_at: Instant::now() + Duration::from_millis(delay_ms),
                                interval: None,
                            });
                        }
                    }
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
        let plugin_id = id.to_owned();
        lua.globals()
            .set(
                "schedule_every",
                lua.create_function(move |_, (interval_ms, function_name): (i64, String)| {
                    let interval_ms = interval_ms.max(1) as u64;
                    let interval = Duration::from_millis(interval_ms);
                    if let Ok(mut tasks) = shared_schedule().lock() {
                        if tasks.len() < MAX_SCHEDULED_TASKS {
                            tasks.push(ScheduledTask {
                                plugin_id: plugin_id.clone(),
                                function_name,
                                run_at: Instant::now() + interval,
                                interval: Some(interval),
                            });
                        }
                    }
                    Ok(())
                })
                .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
            )
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;
    }

    lua.load(script)
        .set_name(id)
        .exec()
        .map_err(|error| PluginLoadError::Compile(error.to_string()))?;
    let globals = lua.globals();
    for (name, parameters, required) in [
        ("on_enable", 0, true),
        ("on_command", 3, !commands.is_empty()),
    ] {
        let exists = matches!(
            globals.get::<mlua::Value>(name),
            Ok(mlua::Value::Function(_))
        );
        if required && !exists {
            return Err(PluginLoadError::MissingFunction { name, parameters });
        }
    }

    let has_on_event = matches!(
        globals.get::<mlua::Value>("on_event"),
        Ok(mlua::Value::Function(_))
    );
    let has_on_disable = matches!(
        globals.get::<mlua::Value>("on_disable"),
        Ok(mlua::Value::Function(_))
    );

    Ok((
        PluginRuntime::Lua {
            lua,
            operation_budget,
        },
        has_on_event,
        has_on_disable,
    ))
}

#[cfg(not(feature = "plugin-lua"))]
fn build_lua_runtime(
    _id: &str,
    _permissions: &[String],
    _commands: &[String],
    _script: &str,
) -> Result<(PluginRuntime, bool, bool), PluginLoadError> {
    Err(PluginLoadError::RuntimeUnavailable(
        "lua (enable plugin-lua)".into(),
    ))
}

#[cfg(feature = "plugin-lua")]
fn register_lua_logger(lua: &mlua::Lua, id: &str) -> Result<(), PluginLoadError> {
    let log_id = id.to_owned();
    lua.globals()
        .set(
            "log_info",
            lua.create_function(move |_, message: String| {
                info!(plugin = %log_id, "{message}");
                Ok(())
            })
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
        )
        .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;

    let log_id = id.to_owned();
    lua.globals()
        .set(
            "log_warn",
            lua.create_function(move |_, message: String| {
                warn!(plugin = %log_id, "{message}");
                Ok(())
            })
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
        )
        .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;

    let log_id = id.to_owned();
    lua.globals()
        .set(
            "log_error",
            lua.create_function(move |_, message: String| {
                error!(plugin = %log_id, "{message}");
                Ok(())
            })
            .map_err(|error| PluginLoadError::Runtime(error.to_string()))?,
        )
        .map_err(|error| PluginLoadError::Runtime(error.to_string()))?;

    Ok(())
}

fn validate_manifest(manifest: &PluginManifest, directory: &Path) -> Result<(), PluginLoadError> {
    let id = &manifest.id;
    if id.is_empty()
        || !id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
        })
    {
        return Err(PluginLoadError::Manifest(
            "id must contain only lowercase ASCII letters, digits, '-' or '_'".to_owned(),
        ));
    }

    if directory.file_name().and_then(|name| name.to_str()) != Some(id.as_str()) {
        return Err(PluginLoadError::Manifest(
            "plugin directory name must match manifest id".to_owned(),
        ));
    }
    if manifest.name.trim().is_empty() || manifest.version.trim().is_empty() {
        return Err(PluginLoadError::Manifest(
            "name and version must not be empty".to_owned(),
        ));
    }
    if !matches!(manifest.runtime.as_str(), "rhai" | "lua") {
        return Err(PluginLoadError::Manifest(format!(
            "unsupported runtime '{}'; supported runtimes are rhai and lua",
            manifest.runtime
        )));
    }
    if manifest.api_version != PLUGIN_API_VERSION {
        return Err(PluginLoadError::ApiVersion {
            found: manifest.api_version,
            supported: PLUGIN_API_VERSION,
        });
    }

    let allowed_permissions = [
        "logging",
        "commands.register",
        "player.read",
        "player.message",
        "player.kick",
        "player.effects",
        "server.info",
        "services",
        "scheduler",
        "world.mutate",
        "packets.inbound",
    ];
    let mut seen_permissions = HashSet::new();
    for permission in &manifest.permissions {
        if !allowed_permissions.contains(&permission.as_str()) {
            return Err(PluginLoadError::Manifest(format!(
                "unknown permission '{permission}'"
            )));
        }
        if !seen_permissions.insert(permission) {
            return Err(PluginLoadError::Manifest(format!(
                "duplicate permission '{permission}'"
            )));
        }
    }

    if !manifest.commands.is_empty()
        && !manifest
            .permissions
            .iter()
            .any(|permission| permission == "commands.register")
    {
        return Err(PluginLoadError::Manifest(
            "declaring commands requires the commands.register permission".to_owned(),
        ));
    }

    let mut commands = HashSet::new();
    for command in &manifest.commands {
        if command.is_empty()
            || !command.bytes().all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-' || byte == b'_'
            })
        {
            return Err(PluginLoadError::Manifest(format!(
                "invalid command name '{command}'"
            )));
        }
        if !commands.insert(command) {
            return Err(PluginLoadError::Manifest(format!(
                "duplicate command '{command}'"
            )));
        }
    }

    Ok(())
}

fn create_engine(plugin_id: &str, permissions: &[String]) -> Engine {
    let mut engine = Engine::new();
    engine.disable_symbol("eval");
    engine.set_module_resolver(rhai::module_resolvers::DummyModuleResolver::new());
    engine.set_max_operations(MAX_OPERATIONS);
    engine.set_max_call_levels(32);
    engine.set_max_string_size(MAX_STRING_BYTES);
    engine.set_max_array_size(MAX_COLLECTION_ITEMS);
    engine.set_max_map_size(MAX_COLLECTION_ITEMS);
    engine.set_max_variables(256);
    engine.set_max_functions(64);
    engine.set_max_modules(0);

    if permissions.iter().any(|permission| permission == "logging") {
        engine.on_print(|message| info!(target: "picolimbo::plugin", "{message}"));
        let id = plugin_id.to_owned();
        engine.register_fn("log_info", move |message: &str| {
            info!(plugin = %id, "{message}");
        });
        let id = plugin_id.to_owned();
        engine.register_fn("log_warn", move |message: &str| {
            warn!(plugin = %id, "{message}");
        });
        let id = plugin_id.to_owned();
        engine.register_fn("log_error", move |message: &str| {
            error!(plugin = %id, "{message}");
        });
    } else {
        engine.disable_symbol("print");
        engine.disable_symbol("debug");
    }

    if permissions
        .iter()
        .any(|permission| permission == "services")
    {
        let id = plugin_id.to_owned();
        engine.register_fn("service_register", move |name: &str, value: &str| {
            if value.len() > MAX_SERVICE_VALUE_BYTES {
                return;
            }
            let key = format!("{id}:{name}");
            if let Ok(mut guard) = shared_services().lock() {
                if guard.len() >= MAX_SERVICE_ENTRIES && !guard.contains_key(&key) {
                    return;
                }
                guard.insert(key, value.to_owned());
            }
        });
        engine.register_fn("service_get", |name: &str| -> String {
            shared_services()
                .lock()
                .ok()
                .and_then(|guard| guard.get(name).cloned())
                .unwrap_or_default()
        });
        engine.register_fn("service_list", || -> rhai::Array {
            shared_services()
                .lock()
                .ok()
                .map(|guard| {
                    guard
                        .keys()
                        .cloned()
                        .map(Dynamic::from)
                        .collect::<rhai::Array>()
                })
                .unwrap_or_default()
        });
        engine.register_fn("service_remove", |name: &str| -> bool {
            shared_services()
                .lock()
                .ok()
                .map(|mut guard| guard.remove(name).is_some())
                .unwrap_or(false)
        });
    }

    if permissions
        .iter()
        .any(|permission| permission == "server.info")
    {
        engine.register_fn("server_api_version", || PLUGIN_API_VERSION as i64);
        engine.register_fn("server_online_players", || {
            shared_online_players().load(std::sync::atomic::Ordering::SeqCst) as i64
        });
        engine.register_fn("world_has_schematic", || {
            shared_has_world().load(std::sync::atomic::Ordering::SeqCst)
        });
        engine.register_fn("world_spawn_x", || {
            shared_spawn_position().lock().map(|g| g.0).unwrap_or(0.5)
        });
        engine.register_fn("world_spawn_y", || {
            shared_spawn_position().lock().map(|g| g.1).unwrap_or(64.0)
        });
        engine.register_fn("world_spawn_z", || {
            shared_spawn_position().lock().map(|g| g.2).unwrap_or(0.5)
        });
    }

    if permissions
        .iter()
        .any(|permission| permission == "scheduler")
    {
        let id = plugin_id.to_owned();
        engine.register_fn(
            "schedule_later",
            move |delay_ms: i64, function_name: &str| {
                let delay_ms = delay_ms.max(0) as u64;
                if let Ok(mut tasks) = shared_schedule().lock() {
                    if tasks.len() < MAX_SCHEDULED_TASKS {
                        tasks.push(ScheduledTask {
                            plugin_id: id.clone(),
                            function_name: function_name.to_owned(),
                            run_at: Instant::now() + Duration::from_millis(delay_ms),
                            interval: None,
                        });
                    }
                }
            },
        );
        let id = plugin_id.to_owned();
        engine.register_fn(
            "schedule_every",
            move |interval_ms: i64, function_name: &str| {
                let interval_ms = interval_ms.max(1) as u64;
                let interval = Duration::from_millis(interval_ms);
                if let Ok(mut tasks) = shared_schedule().lock() {
                    if tasks.len() < MAX_SCHEDULED_TASKS {
                        tasks.push(ScheduledTask {
                            plugin_id: id.clone(),
                            function_name: function_name.to_owned(),
                            run_at: Instant::now() + interval,
                            interval: Some(interval),
                        });
                    }
                }
            },
        );
    }

    if permissions
        .iter()
        .any(|permission| permission == "player.kick")
    {
        engine.register_fn("player_kick", |player_name: &str, reason: &str| {
            PluginManager::request_kick(player_name, reason);
        });
    }

    if permissions
        .iter()
        .any(|permission| permission == "world.mutate")
    {
        engine.register_fn(
            "world_set_block",
            |x: i64, y: i64, z: i64, block_state: &str| -> bool {
                PluginManager::world_set_block(x as i32, y as i32, z as i32, block_state)
            },
        );
        engine.register_fn("world_get_block", |x: i64, y: i64, z: i64| -> i64 {
            PluginManager::world_get_block(x as i32, y as i32, z as i32)
        });
        engine.register_fn("world_clear_overlay", || {
            PluginManager::world_clear_overlay();
        });
        engine.register_fn("world_resend_chunk", |chunk_x: i64, chunk_z: i64| {
            PluginManager::world_resend_chunk(chunk_x as i32, chunk_z as i32);
        });
    }

    if permissions
        .iter()
        .any(|permission| permission == "player.effects")
    {
        engine.register_fn("player_title", |player_name: &str, text: &str| {
            PluginManager::queue_client_action(
                player_name,
                PluginClientAction::Title(text.to_owned()),
            );
        });
        engine.register_fn("player_actionbar", |player_name: &str, text: &str| {
            PluginManager::queue_client_action(
                player_name,
                PluginClientAction::ActionBar(text.to_owned()),
            );
        });
        engine.register_fn("player_teleport_spawn", |player_name: &str| {
            PluginManager::queue_client_action(player_name, PluginClientAction::TeleportSpawn);
        });
    }

    engine
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_directory() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        std::env::temp_dir().join(format!("picolimbo-plugins-{nonce}"))
    }

    #[test]
    fn missing_plugin_directory_is_created_and_empty() {
        let directory = temporary_directory();
        let manager = PluginManager::load(&directory).expect("load plugin directory");
        assert!(manager.is_empty());
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn loads_plugin_and_dispatches_declared_command() {
        let directory = temporary_directory();
        let plugin_directory = directory.join("hello");
        fs::create_dir_all(&plugin_directory).expect("create plugin directory");
        fs::write(
            plugin_directory.join("plugin.toml"),
            r#"
id = "hello"
name = "Hello Plugin"
version = "1.0.0"
api_version = 1
commands = ["hello"]
permissions = ["commands.register", "player.read", "player.message"]
"#,
        )
        .expect("write manifest");
        fs::write(
            plugin_directory.join("main.rhai"),
            r#"
fn on_enable() { }
fn on_command(command, player, args) {
    "Hello, " + player + "!"
}
"#,
        )
        .expect("write script");

        let mut manager = PluginManager::load(&directory).expect("load plugin directory");
        assert_eq!(
            manager.execute_command("hello", "PicoPlayer"),
            Some("Hello, PicoPlayer!".to_owned())
        );

        drop(manager);
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn plugin_state_persists_between_command_calls() {
        let directory = temporary_directory();
        let plugin_directory = directory.join("stateful");
        fs::create_dir_all(&plugin_directory).expect("create plugin directory");
        fs::write(
            plugin_directory.join("plugin.toml"),
            r#"
id = "stateful"
name = "Stateful Plugin"
version = "1.0.0"
api_version = 1
commands = ["count"]
permissions = ["commands.register", "player.message"]
"#,
        )
        .expect("write manifest");
        fs::write(
            plugin_directory.join("main.rhai"),
            r#"
fn on_enable() { state["count"] = 0; }
fn on_command(command, player, args) {
    state["count"] += 1;
    if state["count"] == 1 { "first" } else { "again" }
}
"#,
        )
        .expect("write script");

        let mut manager = PluginManager::load(&directory).expect("load plugin directory");
        assert_eq!(
            manager.execute_command("count", "PicoPlayer"),
            Some("first".to_owned())
        );
        assert_eq!(
            manager.execute_command("count", "PicoPlayer"),
            Some("again".to_owned())
        );
        drop(manager);
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[cfg(feature = "plugin-lua")]
    #[test]
    fn loads_lua_plugin_and_dispatches_declared_command() {
        let directory = temporary_directory();
        let plugin_directory = directory.join("lua-hello");
        fs::create_dir_all(&plugin_directory).expect("create plugin directory");
        fs::write(
            plugin_directory.join("plugin.toml"),
            r#"
id = "lua-hello"
name = "Lua Hello"
version = "1.0.0"
api_version = 1
runtime = "lua"
commands = ["hello"]
permissions = ["commands.register", "player.read", "player.message"]
"#,
        )
        .expect("write manifest");
        fs::write(
            plugin_directory.join("main.lua"),
            r#"
function on_enable()
    visits = 0
end
function on_command(command, player, args)
    visits = visits + 1
    return "Hello, " .. player .. " (" .. visits .. ")"
end
"#,
        )
        .expect("write Lua script");

        let mut manager = PluginManager::load(&directory).expect("load plugin directory");
        assert_eq!(
            manager.execute_command("hello", "PicoPlayer"),
            Some("Hello, PicoPlayer (1)".to_owned())
        );
        assert_eq!(
            manager.execute_command("hello", "PicoPlayer"),
            Some("Hello, PicoPlayer (2)".to_owned())
        );

        drop(manager);
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn loads_more_than_sixty_four_plugins_without_an_arbitrary_count_limit() {
        let directory = temporary_directory();
        fs::create_dir_all(&directory).expect("create plugin directory");

        for index in 0..65 {
            let id = format!("plugin-{index:02}");
            let plugin_directory = directory.join(&id);
            fs::create_dir_all(&plugin_directory).expect("create plugin directory");
            fs::write(
                plugin_directory.join("plugin.toml"),
                format!("id = \"{id}\"\nname = \"{id}\"\nversion = \"1.0.0\"\napi_version = 1\n"),
            )
            .expect("write manifest");
            fs::write(plugin_directory.join("main.rhai"), "fn on_enable() { }")
                .expect("write script");
        }

        let manager = PluginManager::load(&directory).expect("load plugin directory");
        assert_eq!(manager.len(), 65);

        drop(manager);
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn command_plugins_must_define_the_command_handler() {
        let directory = temporary_directory();
        let plugin_directory = directory.join("missing-handler");
        fs::create_dir_all(&plugin_directory).expect("create plugin directory");
        fs::write(
            plugin_directory.join("plugin.toml"),
            r#"
id = "missing-handler"
name = "Missing Handler"
version = "1.0.0"
api_version = 1
commands = ["hello"]
permissions = ["commands.register"]
"#,
        )
        .expect("write manifest");
        fs::write(plugin_directory.join("main.rhai"), "fn on_enable() {}").expect("write script");

        let manager = PluginManager::load(&directory).expect("load plugin directory");
        assert!(manager.is_empty());
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }

    #[test]
    fn rejects_parent_directory_entrypoints() {
        let directory = temporary_directory();
        let plugin_directory = directory.join("bad");
        fs::create_dir_all(&plugin_directory).expect("create plugin directory");
        fs::write(
            plugin_directory.join("plugin.toml"),
            r#"
id = "bad"
name = "Bad Plugin"
version = "1.0.0"
api_version = 1
entrypoint = "../outside.rhai"
"#,
        )
        .expect("write manifest");
        fs::write(directory.join("outside.rhai"), "fn on_enable() {}")
            .expect("write outside script");

        let manager = PluginManager::load(&directory).expect("load plugin directory");
        assert!(manager.is_empty());
        fs::remove_dir_all(directory).expect("remove temporary directory");
    }
}
