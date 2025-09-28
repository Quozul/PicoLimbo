# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.6.0+mc1.21.9] - 2025-09-30

### Added

- Added support for 1.21.9
- Player teleportation system when falling below world boundaries with configurable minimum Y position and teleport message
- Time configuration for all versions with lock time for 1.21.5+ clients
- Tab list header/footer (1.8+)
- Player shows up in the tab list
- Player skins (1.8+)
- Boss bar (1.9+)
- Configurable server icon
- Network compression options
- MiniMessage formatting support for MOTD and welcome messages
- Title and subtitle (1.8+)
- Action bar message (1.8+)

### Changed

- Removed maximum view distance limit
- Spawn position is now a persistent world setting
- Updated forwarding configuration format (see documentation)
- Renamed and relocated spawn dimension setting to `world.dimension`
- Schematic is sent to clients starting from 1.16

### Fixed

- High memory usage when sending a large schematic over the network

## [1.5.2+mc1.21.8] - 2025-09-06

### Fixed

- Handle invalid Unicode strings by replacing it with the replacement characters �

## [1.5.1+mc1.21.8] - 2025-09-01

### Changed

- Major schematic performance optimizations reducing memory, CPU, and network usage for significantly faster schematic loading
- World height for overworld dimension now fixed at 256 blocks across all versions

### Removed

- Removed unused registry entries

## [1.5.0+mc1.21.8] - 2025-08-30

### Added

- Allow customization of the spawn position in the configuration
- View distance can now be customized
- Hardcore mode can be defined in configuration
- Support for loading schematics

### Fixed

- NBT strings and arrays should be prefixed with a UShort
- Send correct version of the game in KnownPacks
- Player not spawning due to view distance being too small in certain versions
- Set center chunk to prevent player getting stuck in loading world screen
- Void chunk is always send for all versions after 1.19
- Empty configuration file never gets filled
- Send correct amount of chunk section given a dimension
- Clouds not rendering for versions after 1.21.6
- Connection from BungeeCord were rejected when BungeeCord is running in offline mode
- Wrong yaw being sent for versions after 1.21.2

## [1.4.0+mc1.21.8] - 2025-08-23

### Changed

- Use the same binary reader and writer for NBT and packets, reducing duplicated code
- Simplified implementation of the Velocity secret key check
- Renamed `nbt` crate to `pico_nbt`
- Bundled packets' protocol IDs into the binary
- Pre-compiled and bundled registries into the binary

### Fixed

- Specify the correct `latest` tag for the Docker image in the documentation
- Shutdown signal is now properly handled on Docker

### Removed

- Removed `pico_ping` utility crate
- Runtime parsing of JSON (registries and packet reports) files

## [1.3.2+mc1.21.8] - 2025-07-30

### Changed

- If the game mode is set to `spectator` in the configuration file, players in 1.7.x will spawn in creative instead of survival

### Fixed

- Keep alive packet not properly sending for 1.7.x

### Removed

- Removed unused `PlayerPositionPacket`

## [1.3.1+mc1.21.8] - 2025-07-19

### Changed

- Updated versioning scheme to adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
  - We'll start with 1.3.1 as we had 3 versions with minor changes before and this one fixes a compatibility issue with ViaVersion

### Fixed

- Error getting displayed when running behind ViaVersion due to the -1 protocol version number

## [v1.21.8] - 2025-07-19

### Added

- Support for 1.21.8

### Changed

- Refactor to simplify the Server struct
- Updated the README and the documentation

### Fixed

- Invalid login start packet between 1.19 and 1.20.1
- Do not always serialize as dynamic list if possible for >=1.21.5, which could cause incompatibilities with some proxy plugins (e.g. PacketEvents)
- UUID is misencoded for <1.7.6 preventing connection to PicoLimbo through Velocity when using 1.7.6 or older clients
- Invalid string decoding could cause a crash of the server if a player tries to connect with a Unicode username
- Accept -1 protocol version number during handshake to improve support with ViaVersion

### Removed

- Removed the build script from pico_limbo binary for faster build times, this removes the detailed version number available when using the help command


## [v1.21.7] - 2025-06-30

### Added

- Support for 1.21.7
- Customizable default game mode in configuration
- Commands auto-completion when running behind a proxy #16
- Added error message in server's console when modern forwarding failed to help debug issues

### Changed

- Improved de-serialization of spawn dimension configuration

### Fixed

- Send correct biome index in minecraft:login play packet
- Correctly implement the palette container data type according to 1.21.5 specs
- Send correct dimension type for 1.20.5, resulting in correct world height and clouds being visible

## [v1.21.6] - 2025-06-23

### Added

- Command-line argument to configure the data directory path
- Introduced a configuration file for easier setup
- Configurable default spawn dimension in the configuration file
- Customizable server Message of the Day (MOTD) and maximum player count (display only) through the configuration file.
- Configurable welcome message sent to players upon login
- Support for BungeeCord and BungeeGuard forwarding
- Added support for 1.21.6

### Changed

- Improved documentation in the README and CLI help
- Online player count is now included in the server's status response
- The Pterodactyl egg file includes additional environment variables to easily configure
- Docker images and standalone binaries are now available for **Linux/arm64**, in addition to Linux/amd64, Windows, and macOS (M-series Macs)
- The default listening address is now set to 0.0.0.0
- Improved error logging for clearer diagnostics
- Direct connection kicks for pre-1.13 clients when modern forwarding is enabled but they attempt to bypass the proxy
- Refined login sequence to strictly follow Minecraft standards required by BungeeCord

### Fixed

- Fixed issue where the server brand was not sent to clients prior to Minecraft 1.20.2
- Resolved an issue that caused crashes whenever a null byte was sent to the server during handshake
- Fixed incorrect Docker image tag in README and docker-compose.yml
- Removed invalid CLI argument in Dockerfile preventing the server from starting
- Enhanced stability and reduced server crashes (panics)
- `worldgen/biome` registry not being sent when running on windows causing a Network Protocol Error on the client

## [v1.21.5-4] - 2025-05-15

- Fixed Docker image not including the assets directory
- Update Pterodactyl egg to use Alpine

## [v1.21.5-2] - 2025-05-15

- Add project license
- Enable LTO and set codegen-units to 1 for optimized builds
- Build for musl Linux to be used in Pterodactyl
- Remove build for Apple Intel because it is an aging platform
- Assets is no longer bundled in the binary

## [v1.21.5-2] - 2025-05-07

- Bundle assets into the binary

## [v1.21.5-1] - 2025-05-07

- First official release of PicoLimbo.
