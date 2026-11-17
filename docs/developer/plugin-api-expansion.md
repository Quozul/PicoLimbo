# PicoLimbo Plugin API: Expansion Architecture

> Status: design target for the plugin API expansion. This document distinguishes the intended API from functionality already implemented. It is not a claim that every API below is available in the current runtime.

PicoLimbo should offer the flexibility developers expect from Bukkit/Paper without pretending that PicoLimbo is a full Bukkit server. Its API must expose the real capabilities of the limbo server, provide safe extension points for new capabilities, and keep advanced access available to trusted plugin authors.

## Design principles

1. **No arbitrary plugin-count limit.** The loader must not silently stop at a fixed number of plugins. Administrators can still configure resource budgets (memory, CPU/fuel, task count, network concurrency, and world-edit limits) without imposing a product-level plugin-count ceiling.
2. **One API, multiple runtimes.** Script engines and compiled plugins use the same versioned host API and event model.
3. **Progressive power.** Simple plugins use concise high-level helpers. Advanced plugins can access typed protocol packets, raw packet frames, and explicitly granted host capabilities.
4. **Asynchronous by default where I/O is involved.** Network, storage, schematic parsing, and bulk world edits must not block packet-processing tasks.
5. **Stable contracts.** Public API packages are versioned independently from Minecraft protocol versions. Protocol-specific types expose capability checks and version-aware codecs.
6. **Observable failures.** Every plugin has structured logs, lifecycle state, health metrics, and actionable load/runtime errors. One plugin failure must not crash unrelated plugins.
7. **Portable distribution.** No paid marketplace or proprietary runtime is required to build, install, or distribute plugins.

## Runtime strategy

Use runtime adapters over one host API. A plugin manifest selects its runtime and entry point.

| Runtime | Purpose | Distribution |
|---|---|---|
| Rhai | Default, low-friction scripting and configuration-driven plugins | Source |
| Lua / Luau | Familiar scripting, coroutine support, dynamic modules | Source |
| JavaScript | JS/TS ecosystem and developer familiarity | Source or bundled JS |
| WebAssembly Component Model | Language-neutral compiled plugins with capability-based imports | .wasm component |
| Java API | JVM plugins and a practical path for ports of Java projects | .jar |
| Native Rust/C ABI | Expert-only, explicitly trusted extensions | Platform binary |

Rhai remains the lightweight default. Lua is a strong second scripting runtime because mlua supports multiple Lua versions, Luau, async functions, userdata, and feature selection. JavaScript is an optional runtime: Boa is embeddable but its own documentation describes it as experimental, so it must not become a default dependency until its maturity and performance are evaluated. Wasmtime is the isolation boundary for portable compiled plugins; its fuel and resource-limiter facilities can enforce CPU and linear-memory budgets.

References:
- https://rhai.rs/book/
- https://github.com/mlua-rs/mlua
- https://boajs.dev/docs/intro
- https://docs.wasmtime.dev/

Do not load arbitrary native shared libraries by default. Rust's native ABI is not a stable plugin contract. Native extensions require an explicitly enabled trusted mode and a versioned C ABI/function table. WASM is the preferred compiled, portable, sandboxed format.

## Host API surface

The API should be organized into discoverable namespaces. Names and signatures should be consistent across Java, scripting, and WASM bindings.

### Lifecycle and plugin services

- Plugin.onLoad(context), onEnable(), onDisable(), onReload()
- Dependency declarations, soft dependencies, load-before/load-after ordering, and cycle diagnostics
- Plugin metadata, logger, data folder, config, resource extraction, and persistent key/value storage
- Service registry for cross-plugin interfaces and capability discovery
- Plugin manager: inspect, enable, disable, reload, and query plugins
- Optional plugin-owned classpath/dependency resolver for JVM plugins
- Per-plugin metrics, health state, structured diagnostics, and crash isolation

### Events

- Typed events with documented fields and stable event identifiers
- Listener registration with priority, ignore-cancelled, one-shot, and plugin ownership
- Cancellable events where the underlying operation can actually be cancelled
- Synchronous pre-events for decisions; asynchronous post-events for observation and I/O
- Custom events so plugins such as permissions, chat, moderation, and cosmetics can interoperate
- Event tracing and listener timing to diagnose hot-path lag

Initial event families: server lifecycle, player pre-login/login/quit, chat, commands, movement, teleport, packet receive/send, plugin messages, world/chunk/block operations, and permission checks.

Events must document thread affinity. Packet hot-path events must be opt-in/filterable and must not run arbitrary blocking work.

### Players and server

- Player lookup by UUID/name, online-player snapshots, profile, protocol, locale, ping, and connection state
- Message/component, action-bar, title, boss-bar, tab-list, sound, and kick helpers
- Teleport with position, rotation, dimension, relative flags, and completion/error result
- Plugin messaging and custom payload channels
- Per-player metadata and persistent data
- Server information, online/max counts, MOTD, maintenance state, and lifecycle

A limbo server does not currently provide a full survival entity simulation, inventory engine, or general-purpose world tick loop. API methods must be backed by implemented server behavior rather than fake Bukkit-shaped methods that do nothing.

### Commands and permissions

- Dynamic command registration and unregistration
- Argument trees, typed arguments, suggestions, aliases, usage, permission requirements, and rich help
- Player, console, and proxy command senders
- Permission provider SPI with cached checks, wildcard semantics, groups, inheritance, contexts, metadata, and change events
- Optional LuckPerms-compatible adapter/API facade where semantics can be implemented faithfully

### Protocol and raw packets

Provide two levels:

1. **Typed protocol API:** version-aware packet models and codecs, protocol capability checks, packet builders, and safe send helpers.
2. **Raw packet API:** receive/send hooks exposing direction, protocol state, packet ID, packet bytes, and cancellation/replacement where safe.

Raw packet access requires explicit permissions such as protocol.raw.read or protocol.raw.write. The API must preserve framing, compression, encryption state, and protocol-state invariants. Plugins must not be encouraged to hard-code packet IDs across versions; expose named packet types and protocol registries.

Packet hooks need deterministic ordering, copy-on-write payload handling, bounded packet sizes, and per-plugin time budgets. A malformed packet must fail the plugin operation, not corrupt another client's stream.

### World, blocks, and schematics

The current PicoLimbo world is loaded from schematic data and exposed as an immutable shared world. Full Bukkit-style block editing therefore requires a real world-mutation layer, not just script bindings.

Target API:

- World.getBlock(position), setBlock(position, blockData), breakBlock(position), placeBlock(position, blockData)
- Batch edits and transactions with bounds, chunk loading, rollback/error reporting, and rate limits
- Schematic load, transform, rotate, mirror, paste, preview, and async paste jobs
- Block entities/NBT with typed helpers and raw NBT escape hatch
- Chunk snapshots and block-change events
- Player teleport and safe-spawn resolution against the mutable world
- Version-aware block-state registry and validation

Implementation prerequisite: replace the immutable Arc<World> assumption with a chunk/section mutation model or overlay. Changed sections must be re-encoded and sent to viewers, with lighting and block entities handled correctly. Until this exists, the API must clearly report world editing as unsupported instead of silently claiming success.

### Networking and external libraries

Network access is a capability, not an ambient default:

- network.http: async HTTP client with timeout, body-size, redirect, and concurrency limits
- network.tcp / network.udp: separately granted raw socket access for advanced integrations
- Optional hostname allowlists and administrator-defined egress policy
- Deny loopback, link-local, private, and metadata-service destinations by default unless explicitly allowed
- Revalidate resolved IPs after redirects and DNS changes to mitigate SSRF and DNS-rebinding
- No shell execution or arbitrary filesystem access as an accidental side effect of enabling networking

Dynamic libraries/packages should be resolved from a plugin-local dependency manifest and cached per plugin. Dependency resolution must support pinned versions, hashes, offline mode, and administrator policy. A script importing a package must not silently gain new host permissions.

### Storage and configuration

- YAML/TOML/JSON configuration with schema validation and comments-preserving edits where practical
- Plugin-owned data directory and resource files
- SQLite built-in optional storage; database drivers behind optional features
- Async storage APIs and transactions
- Namespaced persistent data for players/world positions
- Migrations and backup/restore hooks

## Permissions model

Permissions are explicit and granular. Example manifest:

~~~toml
id = "example-world-tools"
name = "World Tools"
version = "1.0.0"
api_version = 2
runtime = "rhai"
entrypoint = "main.rhai"

[capabilities]
players = ["read", "message", "teleport"]
protocol = ["typed.read", "raw.read", "raw.write"]
world = ["read", "blocks.write", "schematics.paste"]
network = ["http"]
storage = ["plugin-data", "sqlite"]
~~~

This is an illustrative future manifest, not the schema accepted by API v1. Admins should be able to grant, deny, or narrow capabilities per plugin. For example, HTTP can be restricted to specific domains and world editing to a region or maximum number of blocks per operation.

## Compatibility and ports

A Bukkit/Paper API is much larger than a plugin loader. Binary compatibility requires matching class names, method behavior, event timing, scheduler semantics, command APIs, server internals, and expected entity/world implementations. PicoLimbo is a limbo server, so it cannot honestly promise that an arbitrary Bukkit plugin JAR will work unchanged.

The supported route is to port functionality against PicoLimbo's API, with compatibility facades where semantics match:

- **TAB:** a dedicated adapter/plugin for tab list, header/footer, sorting, placeholders, conditions, animations, and scoreboard-like overlays where protocol support exists.
- **EssentialsX-style features:** commands, spawn/warp homes, messaging, moderation helpers, kits and teleport requests, implemented only where the limbo model supports them.
- **LuckPerms-style features:** a permissions provider API and storage/group model, then a compatible adapter. Do not claim LuckPerms binary compatibility until its required Bukkit/Velocity platform contracts are implemented.
- **RustyConnector:** a separate integration module using its actual public module API.

Each port must retain upstream license notices and comply with the source project's license. A clean-room implementation against public behavior is often simpler than transplanting platform-specific code.

## Paid and obfuscated plugins

Support source scripts, Java JARs, and WASM artifacts. Plugin authors may distribute compiled artifacts and obfuscate Java bytecode or ship compiled WASM. The loader should support signed manifests, checksums, publisher identity, license metadata, and optional online/offline license verification hooks.

Obfuscation is a distribution option, not a security boundary. Do not promise that plugin code can never be reverse-engineered. Server owners must retain control over what executes on their machines.

## Versioning and release policy

- Plugin API semantic versioning independent from Minecraft protocol versions.
- Plugin API compatibility ranges in manifests, with clear minimum/maximum support.
- Deprecation windows and migration guides.
- Protocol-specific API capabilities reported at runtime.
- Contract tests for every runtime adapter and API binding.
- Example plugins that exercise every documented API.
- A generated API reference and machine-readable schema for AI coding tools.
- No plugin count ceiling; resource quotas are administrator-controlled and per-plugin.

## Implementation order

1. Remove the arbitrary plugin-count cap and add a regression test.
2. Introduce a public, versioned host API crate and a shared plugin lifecycle/event model.
3. Implement the player/server facade, command registry, scheduler, and service registry.
4. Implement typed packet events and raw packet read/write APIs with protocol-state safety.
5. Build the mutable chunk/section overlay and transactional world-edit/schematic API.
6. Add explicit async HTTP capability and external package/dependency resolution.
7. Add Lua runtime adapter; evaluate Boa for JavaScript and Wasmtime Component Model for compiled portable plugins.
8. Add the JVM plugin SDK and compatibility facades.
9. Port TAB, Essentials-style utilities, LuckPerms-style permissions, and RustyConnector as separate projects/plugins with independent tests.

The default installation remains empty: the host creates plugins/, but no feature plugins are bundled or auto-installed.
