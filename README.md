# PicoLimbo

[![GitHub CI](https://img.shields.io/github/actions/workflow/status/Quozul/PicoLimbo/.github%2Fworkflows%2Fci.yml?branch=master)](https://github.com/Quozul/PicoLimbo/actions)
[![Latest Release](https://img.shields.io/github/v/release/Quozul/PicoLimbo)](https://github.com/Quozul/PicoLimbo/releases)
[![License](https://img.shields.io/github/license/Quozul/PicoLimbo)](LICENSE)
[![Discord](https://img.shields.io/discord/1373364651118694585)](https://discord.gg/M2a9dxJPRy)

An ultra-lightweight, multi-version Minecraft limbo server written in Rust.
It currently supports all Minecraft versions from 1.7.2 through 1.21.8.

---

## Community & Support

If you have any questions or suggestions, join the [Discord server](https://discord.gg/M2a9dxJPRy)!

## Introduction

PicoLimbo is a lightweight [limbo server](https://quozul.dev/posts/2025-05-14-what-are-minecraft-limbo-servers/) written
in Rust, designed primarily as an AFK or waiting server. Its core focus is on efficiency by implementing only essential
packets required for client login and maintaining connection (keep-alive) without unnecessary overhead.

When idle, PicoLimbo uses almost no resources: 0% CPU and less than 10 MB of memory, making it extremely lightweight.

While not aiming to replicate every Minecraft server feature, PicoLimbo supports **all Minecraft versions from 1.7.2
through 1.21.8**, excluding snapshots, with only 28 implemented packets covering over 47 different protocol versions or
75 Minecraft versions.

## Features

### ⚙️ Highly Configurable

Customize your server using a simple TOML configuration file, including welcome message, spawn dimension, server list
MOTD, and more.

### 🔀 Built-in Proxy Support

Seamlessly integrates with major Minecraft proxies:

- Velocity (Modern Forwarding)
- BungeeCord (Legacy Forwarding)
- BungeeGuard & BungeeGuardPlus authentication

### 🎮 Wide Version Compatibility

Supports all Minecraft versions from **1.7.2 to 1.21.8** natively, no need for ViaVersion or additional compatibility
layers.

### ⚡ Ultra-Lightweight & Highly Scalable

Uses **0% CPU while idle** and under 10 MB RAM, enabling thousands of concurrent players thanks to Rust’s asynchronous
runtime and efficient design.

![PicoLimbo.png](./docs/assets/PicoLimbo.png)  
*The screenshot shows just a few of the supported Minecraft versions.*

---

## Quick Start

### Docker

```shell
docker run --rm -p "25565:25565" ghcr.io/quozul/picolimbo:master
```

### Binary

Download from [GitHub Releases](https://github.com/Quozul/PicoLimbo/releases)

For more detailed installation and configuration instructions, please refer to
the [documentation website](https://picolimbo.quozul.dev/).

## Documentation

**Complete documentation is available at [https://picolimbo.quozul.dev/](https://picolimbo.quozul.dev/)**

For detailed installation instructions, configuration options, and usage examples, please visit the documentation
website.

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
