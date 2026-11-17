# Lua Hello World

A small Lua 5.4 plugin using the same PicoLimbo Plugin API lifecycle and command contract as Rhai plugins.

## Build

Build PicoLimbo with the optional Lua runtime:

~~~sh
cargo build --release --features plugin-lua
~~~

## Install

Copy this directory into the runtime plugins/ folder. The directory name must match the manifest ID:

~~~text
plugins/
└── lua-hello-world/
    ├── plugin.toml
    └── main.lua
~~~

Restart PicoLimbo and run /luahello. The runtime exposes only the safe Lua standard library set. It does not grant filesystem, process, or network access implicitly.
