# PicoLimbo

PicoLimbo is a **high-performance Minecraft limbo server** built entirely in **Rust**, offering an ultra-lightweight
solution for temporary player holding in Minecraft networks. It's designed to be fast, efficient, and compatible with
all Minecraft versions from 1.7.2 through the [latest supported version](./supported-versions).

![Multiple versions in one image](../assets/PicoLimbo.png)

## What is a Limbo Server?

A **limbo server** is a minimal, often void-world server environment used to temporarily hold players instead of
disconnecting them. It serves as a "waiting room" for players during:

- Server restarts or maintenance
- AFK (Away From Keyboard) management
- Lobby overflow situations
- Graceful handling of server crashes

Unlike traditional servers, limbo servers are designed to be **resource-efficient**, maintaining player connections
without consuming significant system resources.

## Why Choose PicoLimbo?

- **Performance**: Leverages Rust's async runtime for maximum efficiency
- **Compatibility**: Supports all Minecraft versions from 1.7.2 through
  the [latest supported version](./supported-versions) natively
- **Flexibility**: Highly configurable with a simple TOML configuration file

## What PicoLimbo Will Not Do

- Replace your main game server
- Support plugins or mods for other servers
- Replicate all Minecraft features
- Generate, load or interact with a world

> [!IMPORTANT]
> PicoLimbo is currently under active development. Check out
> our [GitHub repository](https://github.com/Quozul/PicoLimbo) to see the latest progress and contribute.

## Why PicoLimbo Over Other Alternatives?

While there are other limbo server solutions available, PicoLimbo stands out with:

- **True multi-version support** (no need for compatibility layers)
- **Exceptional performance** with 0% CPU usage when idle
- **Modern proxy integration** with Velocity and BungeeCord
- **Rust-based reliability** and memory safety
- **Active development** with regular updates

Join our [Discord community](https://discord.gg/M2a9dxJPRy) for support and discussions!
