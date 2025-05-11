# PicoLimbo

> [!WARNING]
> This software is highly experimental. Use at your own risks and report any bugs by submitting an issue on GitHub.

An attempt at writing a lightweight Minecraft server from scratch in Rust. Currently, supports 1.7.2 up to 1.21.5.

## Introduction

This project is a lightweight Minecraft server written in Rust designed to serve as an AFK or waiting server. Its
primary focus is on efficiency, implementing only the essential packets required for client login and maintaining
connection (keep alive) without unnecessary overhead.

On idle, the server uses 0% of CPU time and less than 10 MB of memory!

The server does not aim to replicate every feature or packet supported by Minecraft servers. However, it aims to support
all Minecraft versions from 1.7.2 up to the most recent ones. It does not support snapshots.

This project only implements 24 different packets and support more than 45 different Minecraft versions.

## Getting Started

### Pterodactyl (recommended)

For those using the Pterodactyl panel, you can simplify deployment by importing the [egg file](./pterodactyl/eggs) into
your panel.

### Using Docker

If you prefer to use Docker, you can run the following command to start the service:

```shell
docker run --rm -p "25565:25565" ghcr.io/quozul/picolimbo:master
```

### Using Docker Compose

For a more managed and scalable setup, use Docker Compose. A sample [docker-compose.yml file](./docker-compose.yml) is
available in the repository. Simply download the `docker-compose.yml` file and run:

```shell
docker compose up
```

### Binary / Standalone

For those who prefer a traditional binary installation, you can download the standalone binary for your operating system
from the [GitHub releases](https://github.com/Quozul/PicoLimbo/releases) page. Follow these steps:

1. Navigate to the [latest release](https://github.com/Quozul/PicoLimbo/releases/latest).
2. Download the appropriate binary for your operating system.
3. Make the binary executable (if necessary) and run it.

No additional dependencies nor Java are required to run this server.
The binary will extract an `assets` directory in the current working directory, containing all necessary files for its
execution.

#### Example Commands for Binary Installation

- **Linux/macOS**:
  ```shell
  chmod +x pico_limbo
  ./pico_limbo
  ```

- **Windows**:
  Simply run the downloaded `.exe` file.

## Features

### Velocity Modern Forwarding

PicoLimbo supports Velocity Modern Forwarding, to enable it, pass the secret key as a command line argument ot the
pico_limbo binary.

```shell
pico_limbo --address 127.0.0.1:25565 --secret-key MyForwardingSecret
```

## Similar Projects

- [Limbo](https://github.com/LOOHP/Limbo) - Supports only one version of the game at a time
- [NanoLimbo](https://github.com/Nan1t/NanoLimbo) - Maintained
  by [BoomEaro's fork](https://github.com/BoomEaro/NanoLimbo/tree/feature/1.21.2)
- [TyphoonLimbo](https://github.com/TyphoonMC/TyphoonLimbo) - No longer active
- [LiteLimbo](https://github.com/ThomasOM/LiteLimbo) - No longer active

## Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvement, please submit an issue or
pull request on GitHub. Make sure to follow the existing code style and include relevant tests.

1. Fork the repository.
2. Create a new branch `git checkout -b <branch-name>`.
3. Make changes and commit `git commit -m 'Add some feature'`.
4. Push to your fork `git push origin <branch-name>`.
5. Submit a pull request.
