//! `PicoLimbo`'s embedded plugin runtime.
//!
//! Plugins run through capability-gated script runtime adapters.
//! No filesystem, network, process, or dynamic module APIs are ambient by default.

use rhai::{AST, Array, Dynamic, Engine, Map, Scope};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path};
#[cfg(feature = "plugin-lua")]
use std::sync::Arc;
#[cfg(feature = "plugin-lua")]
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;
use tracing::{error, info, warn};

const PLUGIN_API_VERSION: u32 = 1;
const MAX_MANIFEST_BYTES: u64 = 16 * 1024;
const MAX_SCRIPT_BYTES: u64 = 256 * 1024;
const MAX_OPERATIONS: u64 = 100_000;
const MAX_STRING_BYTES: usize = 16 * 1024;
const MAX_COLLECTION_ITEMS: usize = 256;

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
    permissions: PluginPermissions,
    hooks: PluginHooks,
    runtime: PluginRuntime,
    enabled: bool,
}

#[derive(Default)]
struct PluginPermissions {
    player_read: bool,
    player_message: bool,
}

struct PluginHooks {
    on_event: bool,
    on_disable: bool,
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

    fn on_event(&mut self, event: &str, player: Option<(&str, &str)>) -> Result<(), String> {
        match self {
            Self::Rhai { engine, ast, scope } => {
                let mut payload = Map::new();
                payload.insert("event".into(), Dynamic::from(event.to_owned()));
                if let Some((name, uuid)) = player {
                    payload.insert("player".into(), Dynamic::from(name.to_owned()));
                    payload.insert("uuid".into(), Dynamic::from(uuid.to_owned()));
                }
                engine
                    .call_fn::<()>(scope, ast, "on_event", (payload,))
                    .map_err(|error| error.to_string())
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
                function
                    .call::<()>(payload)
                    .map_err(|error| error.to_string())
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
}

/// Owns the scripts loaded by one `PicoLimbo` server instance.
#[derive(Default)]
pub struct PluginManager {
    plugins: Vec<LoadedPlugin>,
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
            },
            hooks: PluginHooks {
                on_event: has_on_event,
                on_disable: has_on_disable,
            },
            runtime,
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
    pub fn emit_player_event(&mut self, event: &str, player_name: &str, player_uuid: &str) {
        for plugin in &mut self.plugins {
            if !plugin.enabled || !plugin.hooks.on_event {
                continue;
            }
            let player = plugin
                .permissions
                .player_read
                .then_some((player_name, player_uuid));
            if let Err(err) = plugin.runtime.on_event(event, player) {
                warn!(
                    plugin = %plugin.id,
                    error = %err,
                    "PicoLimbo plugin event handler failed; disabling plugin"
                );
                plugin.enabled = false;
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
