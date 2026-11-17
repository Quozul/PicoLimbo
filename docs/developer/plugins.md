# Plugin Development

PicoLimbo has a small embedded plugin runtime. A fresh installation creates a `plugins/` directory, but **ships with no installed plugins**. Plugins are independent packages: they are not compiled into the PicoLimbo executable and can be added or removed without rebuilding the server.

The default runtime is [Rhai](https://rhai.rs/), a Rust-embedded scripting language. The plugin API is versioned separately from PicoLimbo's Minecraft protocol support.

Lua is also available as an optional runtime. Build PicoLimbo with the `plugin-lua` Cargo feature to enable Lua 5.4 scripts. The Lua VM uses the safe standard-library set, a 16 MiB VM memory limit, and an instruction budget. Filesystem, process, and ambient network access are not implicitly exposed. Both runtimes use the same plugin lifecycle and host API.

The current host API provides plugin lifecycle callbacks, player-login events, manifest-declared commands, logging, player information, and MiniMessage replies. It does not provide Bukkit compatibility or general-purpose world, packet, or networking APIs.

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

## Runtime selection

Rhai is the default and needs no extra build feature:

```toml
runtime = "rhai"
entrypoint = "main.rhai"
```

Lua plugins use the optional Lua runtime:

```toml
runtime = "lua"
entrypoint = "main.lua"
```

Lua is enabled at build time with `cargo build --features plugin-lua`. If the runtime was not compiled in, PicoLimbo reports that clearly and skips that plugin rather than silently selecting another engine.

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

Unknown manifest fields and permissions are rejected. Entry points are canonicalized and must remain inside the plugin directory. The machine-readable [Plugin API v1 JSON Schema](/schemas/plugin-manifest-v1.schema.json) can validate the parsed TOML manifest and can be used by editor integrations.

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

Current events: `player_login`, `player_quit`, `player_command`.

Handlers may return `true` to cancel the event (ignored for Monitor priority). Void/legacy handlers are treated as not cancelling.

Event handlers run in priority order (Highest → Lowest). Script plugins currently share Normal priority; load order is influenced by `depends_on` / `soft_depends`.

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
| `server.info` | `server_api_version()`, `server_online_players()`, `world_has_schematic()`, `world_spawn_x/y/z()` |
| `services` | `service_register`, `service_get`, `service_list`, `service_remove` (string KV, namespaced by plugin on register) |
| `scheduler` | `schedule_later(ms, fn_name)` and `schedule_every(ms, fn_name)` |
| `player.kick` | `player_kick(name, reason)` queues a disconnect processed on the client loop |
| `packets.inbound` | Receive cancellable `packet:<name>` events for play-state serverbound packets |
| `player.effects` | `player_title`, `player_actionbar`, `player_teleport_spawn` |

No filesystem, network, process execution, native library loading, or dynamic module resolver is exposed to scripts. Permissions are default-deny. Do not assume a permission exists just because a future API version may add it.

## Runtime limits

Each plugin gets its own Rhai engine. The host applies limits to script operations, call depth, string size, arrays, maps, variables and functions. Manifests and scripts also have size limits. These are resource controls, not a claim that an embedded scripting language is a complete security boundary; only install plugins you trust.

## Compatibility

A plugin must declare the exact Plugin API version it targets. PicoLimbo skips plugins built for an unsupported API version and reports the reason in the server log. API changes should be documented and versioned independently from Minecraft protocol versions.

## Contributing

Community plugins can be maintained in their own repositories and shared through the PicoLimbo community projects page. Keep optional functionality in plugins instead of adding feature-specific code to PicoLimbo's core.


## Scheduler

With the `scheduler` permission, Rhai plugins can register named callbacks:

```rhai
fn on_enable() {
    schedule_later(5000, "later_hello");
    schedule_every(30000, "heartbeat");
}

fn later_hello() {
    log_info("5 seconds later");
}

fn heartbeat() {
    log_info("heartbeat; online=" + server_online_players());
}
```

Tasks are owned by the plugin id. Failing callbacks disable the plugin. The host drains due tasks on the keep-alive path.


## Player facade (limbo-scoped)

Limbo does not expose inventory/world mutation. Available player interactions:

- Event payload fields `player` / `uuid` when `player.read` is granted
- Command reply MiniMessage when `player.message` is granted
- `player_kick(name, reason)` when `player.kick` is granted (applied on the next client processing tick)
- Built-in `/transfer` remains a server command, not a plugin host helper yet

World/packet interception APIs are intentionally out of scope for this foundation.


## Mini-games on limbo

PicoLimbo is a lightweight void limbo, not a full world server. For mini-game style logic:

- Use `packets.inbound` and cancel events like `packet:minecraft:move_player_pos` / `packet:minecraft:chat_command` when needed.
- Use `player.effects` for titles, action bars, and teleport-to-spawn.
- Use scheduler + services for match timers and shared state.

### World mutation (honest limits)

True block/entity world mutation is **not** available yet: the packet crate does not currently expose generic block-change packets, and the server serves static limbo chunks. Building a real mini-game *arena world* needs additional protocol support (block updates, optional custom chunk providers) as a follow-up milestone—not something to fake in the plugin host.

What you *can* do today for mini-game UX on limbo: movement intercept, commands, kicks, titles/actionbars, teleports to configured spawn, timed logic, and transfers to another server for the real arena.


## Memory and performance

The plugin host is designed so a normal limbo process can stay well under **1 GiB** even with several scripts loaded:

- Instruction budgets and sandboxing on Rhai/Lua (existing)
- Caps: scheduled tasks (256), service entries (512), service value size (4 KiB), kick queue (128), client effects per player (16)
- `packet:*` events are delivered only to plugins with `packets.inbound`
- World schematic data stays in the existing `World` structure; plugins get read-only spawn/world metadata helpers under `server.info`

### World protocol / mutation

PicoLimbo already loads optional schematics into `pico_structures::World` and streams chunk data. **Runtime block mutation** (set block → multi-version block-change packets → resend sections) is a separate protocol milestone: the packet crate does not yet expose general block-update packets. Do not expect `set_block` until that lands.

Available now under `server.info`:

- `world_has_schematic()`
- `world_spawn_x()` / `world_spawn_y()` / `world_spawn_z()`
