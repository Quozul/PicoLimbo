# PicoLimbo

PicoLimbo is a **high-performance Minecraft limbo server** built entirely in **Rust**, offering an ultra-lightweight
solution for temporary player holding in Minecraft networks. It's designed to be fast, efficient, and compatible with
all Minecraft versions from 1.7.2 through the [latest supported version](./supported-versions).

![Multiple versions in one image](/PicoLimbo.png)

## What is a Limbo Server?

A **limbo server** is a minimal, often void-world server environment used to temporarily hold players instead of
disconnecting them. Some example use cases includes:

- Authentication server
- Server restarts or maintenance
- AFK (Away From Keyboard) management
- Lobby overflow situations
- Graceful handling of server crashes

Unlike traditional servers, limbo servers are designed to be **resource-efficient**, maintaining player connections
without consuming significant system resources.

## What PicoLimbo Will Not Do

- Replace your main game server
- Support plugins or mods
- Replicate all Minecraft features
- Generate, load or interact with a world

> [!IMPORTANT]
> PicoLimbo is currently under active development. Check out
> our [GitHub repository](https://github.com/Quozul/PicoLimbo) to see the latest progress and contribute.

## Why PicoLimbo Over Other Alternatives?

* Compare supported features in our [comparison table](./comparison).
* PicoLimbo is also extremely lightweight, as shown in [the benchmarks](./benchmarks).
* Every feature can be customized via the [configuration file](/config/introduction).
