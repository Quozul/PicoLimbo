# Hello World plugin

A minimal PicoLimbo plugin demonstrating the manifest, command registration, lifecycle hooks, player events, logging and MiniMessage response path.

## Install

Copy this directory into the server's `plugins/` directory. The directory name must match the manifest `id`:

```text
plugins/
└── hello-world/
    ├── plugin.toml
    └── main.rhai
```

Restart PicoLimbo and run `/hello`. The example uses only the v1 host API.
