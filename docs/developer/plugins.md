# Plugin Development

PicoLimbo has a small embedded plugin runtime. A fresh installation creates a `plugins/` directory, but **ships with no installed plugins**. Plugins are independent packages: they are not compiled into the PicoLimbo executable and can be added or removed without rebuilding the server.

The first runtime is [Rhai](https://rhai.rs/), a Rust-embedded scripting language with a syntax that is approachable for people and code-generation tools. The host API is versioned separately from PicoLimbo's Minecraft protocol support.

## Install a plugin

Create one directory per plugin under `plugins/`. The directory name must match the plugin ID.

```text
plugins/
└── hello-world/
    ├── plugin.toml
    └── main.rhai
```

Copy the working example from `examples/plugins/hello-world/`, then restart PicoLimbo. The server scans plugin directories at startup, loads valid plugins in alphabetical order, and skips a broken plugin without preventing the server from starting.

Use `--plugins-dir <path>` to select another plugin directory.

## Manifest

Every plugin has a `plugin.toml` file:

```toml
id = "hello-world"
name = "Hello World"
version = "1.0.0"
api_version = 1
entrypoint = "main.rhai"
commands = ["hello"]
permissions = [
    "logging",
    "commands.register",
    "player.read",
    "player.message",
]
```

| Field | Required | Description |
| --- | --- | --- |
| `id` | Yes | Lowercase plugin identifier; must match its directory name |
| `name` | Yes | Human-readable plugin name |
| `version` | Yes | Plugin version |
| `api_version` | Yes | PicoLimbo Plugin API version |
| `entrypoint` | No | Script path relative to the plugin directory; defaults to `main.rhai` |
| `commands` | No | Commands registered by the plugin, without a leading slash |
| `permissions` | No | Host capabilities explicitly requested by the plugin |

Unknown manifest fields and permissions are rejected. Entry points are canonicalized and must remain inside the plugin directory. The machine-readable [Plugin API v1 JSON Schema](/schemas/plugin-manifest-v1.schema.json) can validate the parsed TOML manifest and is suitable for editor and AI-tool integrations.

## Lifecycle and events

The entrypoint may define these functions:

```rhai
fn on_enable() {
    log_info("Plugin enabled");
}

fn on_event(event) {
    // event is a map. For player_login:
    // event["event"], event["player"], event["uuid"]
}

fn on_disable() {
    log_info("Plugin disabled");
}
```

`on_enable` is required. `on_event` and `on_disable` are optional. A plugin that throws an error in an event or command callback is disabled so one faulty plugin cannot repeatedly break the server.

Every plugin receives a mutable `state` map. Values persist between that plugin's callbacks for the lifetime of the server process, but are reset when PicoLimbo restarts. Use it for counters, settings and other small in-memory plugin state; it is not a disk-backed database.

Current event: `player_login`.

## Commands

Declare command names in the manifest and request `commands.register`. Implement one callback:

```rhai
fn on_command(command, player, args) {
    "Hello, " + player + "!"
}
```

- `command` is the command name without `/`.
- `player` is the player's username. It is populated only with `player.read`; otherwise it is an empty string.
- `args` is an array of string arguments.
- Return a MiniMessage string to reply to the player. A response is sent only with `player.message`.
- Return an empty string for no reply.

Plugin commands appear in the client's command tree as simple commands. Built-in PicoLimbo commands take precedence over plugin commands.

## Host API and permissions

The v1 runtime exposes only a deliberately small host API:

| Permission | Host functions / behavior |
| --- | --- |
| `logging` | `log_info(message)`, `log_warn(message)`, `log_error(message)` |
| `commands.register` | Allows the plugin to declare manifest commands |
| `player.read` | Makes the invoking player's username available to `on_command` and adds `player` / `uuid` to player event payloads |
| `player.message` | Allows a command callback's returned MiniMessage to be sent to the player |

No filesystem, network, process execution, native library loading, or dynamic module resolver is exposed to scripts. Permissions are default-deny. Do not assume a permission exists just because a future API version may add it.

## Runtime limits

Each plugin gets its own Rhai engine. The host applies limits to script operations, call depth, string size, arrays, maps, variables and functions. Manifests and scripts also have size limits. These are resource controls, not a claim that an embedded scripting language is a complete security boundary; only install plugins you trust.

## AI-assisted plugin authoring

The API is intentionally small and explicit so an AI coding tool can generate plugins reliably. Give your coding tool:

1. This page and the `examples/plugins/hello-world/` directory.
2. The plugin's goal and the Minecraft protocol versions your server supports.
3. The exact permissions the plugin needs.

Ask it to create `plugin.toml` and `main.rhai` for API version 1, use only documented host functions, avoid filesystem/network access, and explain every permission it requests. Have it validate the manifest and test the plugin on a staging server before production.

## Compatibility

A plugin must declare the exact Plugin API version it targets. PicoLimbo skips plugins built for an unsupported API version and reports the reason in the server log. API changes should be documented and versioned independently from Minecraft protocol versions.

## Contributing

Community plugins can be maintained in their own repositories and shared through the PicoLimbo community projects page. Keep optional functionality in plugins instead of adding feature-specific code to PicoLimbo's core.
