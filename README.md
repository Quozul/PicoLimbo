# PicoLimbo

[![GitHub CI](https://img.shields.io/github/actions/workflow/status/Quozul/PicoLimbo/.github%2Fworkflows%2Fci.yml?branch=master)](https://github.com/Quozul/PicoLimbo/actions)
[![Latest Release](https://img.shields.io/github/v/release/Quozul/PicoLimbo)](https://github.com/Quozul/PicoLimbo/releases)
[![License](https://img.shields.io/github/license/Quozul/PicoLimbo)](LICENSE)
[![Discord](https://img.shields.io/discord/1373364651118694585)](https://discord.gg/M2a9dxJPRy)

An ultra-lightweight, multi-version Minecraft limbo server written in Rust.
It currently supports all Minecraft versions from 1.7.2 through 1.21.6.

---

## Community & Support

If you have any questions or suggestions, join the [Discord server](https://discord.gg/M2a9dxJPRy)!

## Introduction

PicoLimbo is a lightweight [limbo server](https://quozul.dev/posts/2025-05-14-what-are-minecraft-limbo-servers/) written
in Rust, designed primarily as an AFK or waiting server. Its core focus is on efficiency by implementing only essential
packets required for client login and maintaining connection (keep-alive) without unnecessary overhead.

When idle, PicoLimbo uses almost no resources: 0% CPU and less than 10 MB of memory, making it extremely lightweight.

While not aiming to replicate every Minecraft server feature, PicoLimbo supports **all Minecraft versions from 1.7.2
through 1.21.6**, excluding snapshots, with only 27 implemented packets covering over 46 different protocol versions or
74 Minecraft versions.

## Features

### ⚙️ Highly Configurable

Customize your server using a simple TOML configuration file, including welcome message, spawn dimension, server list
MOTD, and more.  
See the [Configuration](#-example-configuration-file) section for full details.

### 🔀 Built-in Proxy Support

Seamlessly integrates with major Minecraft proxies:

- Velocity (Modern Forwarding)
- BungeeCord (Legacy Forwarding)
- BungeeGuard & BungeeGuardPlus authentication

### 🎮 Wide Version Compatibility

Supports all Minecraft versions from **1.7.2 to 1.21.6** natively, no need for ViaVersion or additional compatibility
layers.

### ⚡ Ultra-Lightweight & Highly Scalable

Uses **0% CPU while idle** and under 10 MB RAM, enabling thousands of concurrent players thanks to Rust’s asynchronous
runtime and efficient design.

![PicoLimbo.png](./docs/assets/PicoLimbo.png)  
*The screenshot shows just a few of the supported Minecraft versions.*

---

## Getting Started

### 🚀 Pterodactyl (Recommended)

For users running the Pterodactyl panel, deployment is simplified with the provided [egg file](./pterodactyl/eggs). This
egg is built on the lightweight Alpine base image.

The egg supports additional installation configuration through the following environment variables:

- **VERSION**  
  Specifies the Git tag of the release to install. This can be a stable or prerelease tag (e.g., `v1.21.6`).
    - Default: `latest`
    - When set to `latest` (or left unset without enabling prerelease), the installer selects the newest stable release.

- **USE_PRERELEASE**  
  When set to `true`, the installer ignores stable releases and installs the newest prerelease based on publication
  date.
    - Default: *(empty)*

### 🐋 Using Docker

The Docker image is multi-platform, supporting both Linux/amd64 and Linux/arm64 architectures. You can start the server
using the following command:

```shell
docker run --rm -p "25565:25565" ghcr.io/quozul/picolimbo:master
```

You can also mount a custom configuration file:

```shell
docker run --rm -p "25565:25565" -v /path/to/your/server.toml:/usr/src/app/server.toml ghcr.io/quozul/picolimbo:master
```

> [!NOTE]
> The `master` tag image is updated on every push to the repository. For production or stable setups, consider using a
> fixed version tag instead.  
> A list of available tags can be found on the
> [GitHub Packages page](https://github.com/Quozul/PicoLimbo/pkgs/container/picolimbo/versions?filters%5Bversion_type%5D=tagged).

#### Using Docker Compose

A sample [docker-compose.yml file](./docker-compose.yml) is available in the repository. Download the
`docker-compose.yml` file and run:

```shell
docker compose up
```

### 📦 Binary / Standalone

> [!IMPORTANT]
> Ensure the `assets` directory is placed alongside the PicoLimbo binary, as it contains essential files required for
> server execution.

#### GitHub Releases

Download pre-compiled binaries for multiple platforms from
the [GitHub releases page](https://github.com/Quozul/PicoLimbo/releases). No Java or other dependencies are required.

#### Compiling from Source with Cargo

To build PicoLimbo from source, you can use Cargo:

```bash
cargo install --git https://github.com/Quozul/PicoLimbo.git pico_limbo
```

---

## Documentation

### 🔧 Example Configuration File

A default configuration file will be automatically generated the first time you start the server.

```toml
# Server bind address and port
bind = "0.0.0.0:25565"

# Default spawn dimension: "overworld", "nether", or "end"
spawn_dimension = "overworld"

# Welcome message sent to players after spawning
welcome_message = "Welcome to PicoLimbo!"

[forwarding.velocity]
# Enable Velocity Modern Forwarding
enabled = false
# Shared secret for Velocity proxy
secret = ""

[forwarding.bungee_cord]
# Enable BungeeCord forwarding
enabled = false
# Enable BungeeGuard (requires BungeeCord to be enabled)
bungee_guard = false
# List of valid BungeeGuard tokens for authenticating incoming players
tokens = []

[server_list]
# Maximum count shown in your server list, does not affect the player limit
max_players = 20
# MOTD displayed in server lists
message_of_the_day = "A Minecraft Server"
# Show actual online player count in your server list?
show_online_player_count = true
```

### ⌨️ Command Line Usage

1. Run the server:
   ```bash
   pico_limbo
   ```
2. Use a custom configuration file:
   ```bash
   pico_limbo --config /path/to/custom/config.toml
   ```
3. Enable verbose logging:
   ```bash
   pico_limbo -v  # Debug logging
   pico_limbo -vv # Trace logging
   ```

---

## Similar Projects

- [Limbo](https://github.com/LOOHP/Limbo): Supports only one Minecraft version at a time
- [NanoLimbo](https://github.com/Nan1t/NanoLimbo): Actively maintained
  (see [BoomEaro's fork](https://github.com/BoomEaro/NanoLimbo/tree/feature/1.21.2))
- [TyphoonLimbo](https://github.com/TyphoonMC/TyphoonLimbo): No longer actively maintained
- [LiteLimbo](https://github.com/ThomasOM/LiteLimbo): No longer actively maintained

---

## Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvement, please submit an issue or
pull request on GitHub. Make sure to follow the existing code style and include relevant tests.

1. Fork the repository.
2. Create a new branch `git checkout -b <branch-name>`.
3. Make changes and commit `git commit -m 'Add some feature'`.
4. Push to your fork `git push origin <branch-name>`.
5. Submit a pull request.
