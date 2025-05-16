# PicoLimbo

[![GitHub CI](https://img.shields.io/github/actions/workflow/status/Quozul/PicoLimbo/.github%2Fworkflows%2Fci.yml?branch=master)](https://github.com/Quozul/PicoLimbo/actions)
[![Latest Release](https://img.shields.io/github/v/release/Quozul/PicoLimbo)](https://github.com/Quozul/PicoLimbo/releases)
[![License](https://img.shields.io/github/license/Quozul/PicoLimbo)](LICENSE)
[![Discord](https://img.shields.io/discord/1373364651118694585)](https://discord.gg/M2a9dxJPRy)

An ultra-lightweight, multi-version Minecraft limbo server written in Rust.
It currently supports all Minecraft versions from 1.7.2 through 1.21.5.

---

## Community & Support

If you have any questions or suggestions, join the [Discord server](https://discord.gg/M2a9dxJPRy)!

## Introduction

PicoLimbo is a lightweight [limbo server](https://quozul.dev/posts/2025-05-14-what-are-minecraft-limbo-servers/) written
in Rust, designed primarily as an AFK or waiting server. Its core focus is on efficiency by implementing only essential
packets required for client login and maintaining connection (keep-alive) without unnecessary overhead.

When idle, PicoLimbo uses almost no resources: 0% CPU and less than 10 MB of memory, making it extremely lightweight.

While not aiming to replicate every Minecraft server feature, PicoLimbo supports **all Minecraft versions from 1.7.2
through 1.21.5**, excluding snapshots, with only 24 implemented packets covering over 45 Minecraft versions.

## Features

### Velocity Modern Forwarding

Supports Velocity Modern Forwarding, allowing it to receive forwarded player information from the Velocity
proxy. To enable this, pass the secret key as a command line argument to the `pico_limbo` binary:

```shell
pico_limbo --address 127.0.0.1:25565 --secret-key MyForwardingSecret
```

### Multiple Version Support

Supports all major Minecraft versions from 1.7.2 to 1.21.5 with a single binary, no need for ViaVersion or
ViaBackwards. Snapshots are not supported.

![PicoLimbo.png](./docs/assets/PicoLimbo.png)  
_Only a few of the supported versions are on the above screenshot._

---

## Getting Started

### Pterodactyl (recommended)

For users of the Pterodactyl panel, deployment is simplified with the included [egg file](./pterodactyl/eggs).  
Velocity Modern Forwarding can be enabled by setting the corresponding environment variable from Pterodactyl panel.

### Using Docker

Start the server easily with Docker:

```shell
docker run --rm -p "25565:25565" ghcr.io/quozul/picolimbo:latest
```

### Using Docker Compose

For a more managed and scalable setup, use Docker Compose. A sample [docker-compose.yml file](./docker-compose.yml) is
available in the repository. Simply download the `docker-compose.yml` file and run:

```shell
docker compose up
```

### Binary / Standalone

> [!IMPORTANT]
> Ensure the `assets` directory is placed alongside the PicoLimbo binary, as it contains essential files required for
> server execution.

#### GitHub Releases

Download pre-compiled binaries for multiple platforms from
the [GitHub releases page](https://github.com/Quozul/PicoLimbo/releases). No Java or other dependencies required.

#### Compiling from Source with Cargo

To build PicoLimbo from source, you can use Cargo:

```bash
cargo install --git https://github.com/Quozul/PicoLimbo.git pico_limbo
```

---

## Similar Projects

- [Limbo](https://github.com/LOOHP/Limbo) — Supports only one Minecraft version at a time
- [NanoLimbo](https://github.com/Nan1t/NanoLimbo) — Actively maintained
  (see [BoomEaro's fork](https://github.com/BoomEaro/NanoLimbo/tree/feature/1.21.2))
- [TyphoonLimbo](https://github.com/TyphoonMC/TyphoonLimbo) — No longer actively maintained
- [LiteLimbo](https://github.com/ThomasOM/LiteLimbo) — No longer actively maintained

---

## Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvement, please submit an issue or
pull request on GitHub. Make sure to follow the existing code style and include relevant tests.

1. Fork the repository.
2. Create a new branch `git checkout -b <branch-name>`.
3. Make changes and commit `git commit -m 'Add some feature'`.
4. Push to your fork `git push origin <branch-name>`.
5. Submit a pull request.
