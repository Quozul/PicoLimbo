<div align="center">

# PicoLimbo

**An ultra-lightweight, multi-version Minecraft limbo server written in Rust**

*Supporting all Minecraft versions from 1.7.2 through 1.21.11*

[![GitHub CI](https://img.shields.io/github/actions/workflow/status/Quozul/PicoLimbo/.github%2Fworkflows%2Fci.yml?branch=master)](https://github.com/Quozul/PicoLimbo/actions)
[![Latest Release](https://img.shields.io/github/v/release/Quozul/PicoLimbo)](https://github.com/Quozul/PicoLimbo/releases)
[![License](https://img.shields.io/github/license/Quozul/PicoLimbo)](LICENSE)
[![Discord](https://img.shields.io/discord/1373364651118694585)](https://discord.gg/M2a9dxJPRy)

[⭐ Star this repo](https://github.com/Quozul/PicoLimbo) • [💬 Join Discord](https://discord.gg/M2a9dxJPRy) • [📖 Read Docs](https://picolimbo.quozul.dev/)

![PicoLimbo.png](docs/public/world.png)  
*Schematic from [LOOHP/Limbo](https://github.com/LOOHP/Limbo)*

</div>

---

## Community & Support

If you have any questions, suggestions or simply want to stay updated, please join the [Discord server](https://discord.gg/M2a9dxJPRy)!

## Introduction

PicoLimbo is a lightweight [limbo server](https://quozul.dev/posts/2025-05-14-what-are-minecraft-limbo-servers/) written
in Rust, designed primarily as an AFK or waiting server. Its core focus is on efficiency by implementing only essential
packets required for client login and maintaining connection (keep-alive) without unnecessary overhead.

While not aiming to replicate every Minecraft server feature, PicoLimbo supports **all Minecraft versions from 1.7.2
through 1.21.11**, excluding snapshots, with only 44 implemented packets covering over 49 different protocol versions or
77 Minecraft versions.

## Features

### 🎮 Wide Version Compatibility

Supports all Minecraft versions from **1.7.2 to 1.21.11** natively, no need for ViaVersion or additional compatibility
layers.

### ⚡ Ultra-Lightweight & Highly Scalable

Uses **0% CPU while idle** and handles **hundreds of players** under 10 MB RAM.
[View benchmarks](https://picolimbo.quozul.dev/about/benchmarks.html).

### 👤 Skin Support

Player skins are supported. What else to say about that?

### 🔀 Built-in Proxy Support

Integrates with all major Minecraft proxies:

- Velocity (Modern Forwarding)
- BungeeCord (Legacy Forwarding)
- BungeeGuard & BungeeGuardPlus authentication

### ⚙️ Highly Configurable

Customize your server using a simple TOML configuration file, including welcome message, spawn dimension, server list
MOTD, and more. [View configuration docs](https://picolimbo.quozul.dev/config/introduction.html).

### 🌍 Schematic World (Experimental)

Load a custom world from a schematic file and customize spawn location (1.16+ only).

![PicoLimbo.png](docs/public/PicoLimbo.png)  
*The screenshot shows just a few of the supported Minecraft versions.*

---

## Quick Start

### Docker

```shell
docker run --rm -p "25565:25565" ghcr.io/quozul/picolimbo:latest
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
  (see [BoomEaro's fork](https://github.com/BoomEaro/NanoLimbo))

---

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=Quozul/PicoLimbo&type=Date)](https://star-history.com/#Quozul/PicoLimbo&Date)

## Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvement, please submit an issue or
pull request on GitHub. Make sure to follow the existing code style and include relevant tests.

1. Fork the repository.
2. Create a new branch `git checkout -b <branch-name>`.
3. Make changes and commit `git commit -m 'Add some feature'`.
4. Push to your fork `git push origin <branch-name>`.
5. Submit a pull request.
