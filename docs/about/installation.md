# Installation

## Pterodactyl

For users running the Pterodactyl panel, deployment is simplified with the provided egg files. These eggs are built on lightweight base images.

You can find the egg files in the GitHub repository:
- **Alpine-based (recommended):** [egg-pico-limbo.json](https://github.com/Quozul/PicoLimbo/blob/master/pterodactyl/eggs/egg-pico-limbo.json)
- **Debian-based:** [egg-pico-limbo--debian.json](https://github.com/Quozul/PicoLimbo/blob/master/pterodactyl/eggs/egg-pico-limbo--debian.json)

The eggs support additional installation configuration through the following environment variable:

- **VERSION**  
  Specifies the Git tag of the release to install (e.g., `v1.10.0+mc1.21.11`).
    - Default: `latest`
    - When set to `latest` (or left unset), the installer selects the newest stable release.

### Custom Binary Upload

If you want to upload a custom binary file, you can modify the startup command in your server settings:

```
chmod +x pico_limbo && ./pico_limbo
```

> [!WARNING]  
> Uploading custom binary files is **not recommended**. The automatic installation through Pterodactyl's built-in process is the preferred method for reliability and updates. Only use this approach if you have specific requirements that prevent using the standard egg installation.

## Using Docker

The Docker image is multi-platform, supporting both Linux/amd64 and Linux/arm64 architectures. You can start the server using the following command:

```shell
docker run --rm -p "25565:25565" ghcr.io/quozul/picolimbo:latest
```

You can also mount a custom configuration file:

```shell
docker run --rm -p "25565:25565" -v /path/to/your/server.toml:/usr/src/app/server.toml ghcr.io/quozul/picolimbo:latest
```

## Using Docker Compose

Here's the complete docker-compose.yml file:

```yaml
services:
  pico-limbo:
    image: ghcr.io/quozul/picolimbo:latest
    container_name: picolimbo
    restart: unless-stopped
    ports:
      - "25565:25565"
    volumes:
      - ./server.toml:/usr/src/app/server.toml
```

To use this configuration:
1. Create a new directory for your PicoLimbo installation
2. Create a `docker-compose.yml` file with the content above
3. Create a `server.toml` file with your configuration
4. Run `docker compose up -d` to start the server

## Binary / Standalone

### GitHub Releases

For the easiest installation, use the one-line install script:

```bash
curl -fsSL https://picolimbo.quozul.dev/pico_limbo_installation.sh | bash
```

**Requirements:** Linux, curl, and bash

If you cannot use the installation script due to missing dependencies or unsupported platform, you can manually download the appropriate binary from the [GitHub releases page](https://github.com/Quozul/PicoLimbo/releases).

#### Choosing the Right Binary

Select the binary that matches your system:

##### Recommended
| Binary                                    | OS            | Architecture     | Use Case                |
|-------------------------------------------|---------------|------------------|-------------------------|
| **`pico_limbo_linux-x86_64-musl.tar.gz`** | Linux (musl)  | Intel/AMD 64-bit | Best for most users     |
| **`pico_limbo_linux-aarch64-gnu.tar.gz`** | Linux (glibc) | ARM 64-bit       | Armbian on Raspberry Pi |

##### Other Builds
| Binary                                     | OS            | Architecture     |
|--------------------------------------------|---------------|------------------|
| **`pico_limbo_linux-x86_64-gnu.tar.gz`**   | Linux (glibc) | Intel/AMD 64-bit |
| **`pico_limbo_linux-aarch64-musl.tar.gz`** | Linux (musl)  | ARM 64-bit       |
| **`pico_limbo_macos-aarch64.tar.gz`**      | macOS         | Apple Silicon    |
| **`pico_limbo_windows-x86_64.zip`**        | Windows       | Intel/AMD 64-bit |

#### Manual Installation

1. **Download** the appropriate binary for your system from the releases page
2. **Extract** the archive:
    - **Linux/macOS**: `tar -xzf pico_limbo_*.tar.gz`
    - **Windows**: Use your preferred archive tool or built-in extraction
3. **Run** the binary:
    - **Linux/macOS**: `./pico_limbo`
    - **Windows**: Double-click `pico_limbo.exe` or run it from Command Prompt

> [!TIP]
> On Unix systems (Linux and macOS), you may want to move the binary to a directory in your PATH (like `/usr/local/bin/`) to run it from anywhere, or make it executable with `chmod +x pico_limbo` if needed.

## Java Wrapper

A Java wrapper for PicoLimbo is available on [Modrinth](https://modrinth.com/plugin/picolimbo-java-wrapper). This wrapper allows you to run PicoLimbo as a standalone application or as a plugin for Velocity or BungeeCord proxies.

> [!WARNING]
> The Java wrapper is **not the recommended way** of running PicoLimbo. To get maximum performance, users are encouraged to use the binary directly. The Java wrapper is provided to reach more people that have limited setup options.

### Platform Compatibility

Since the Java wrapper uses native code, it cannot run on all platforms. It is only compatible with:

- **GNU/Linux** (e.g., Debian, Ubuntu) - x64 CPUs (AMD/Intel) or arm64 CPUs (e.g., Raspberry Pi)
- **Windows** - x64 CPUs (AMD/Intel)
- **macOS** - M-series chips (M1/M2/M3+)

If you are unsure whether it'll work on your system, try it. Most hosting providers use GNU/Linux with x64 CPUs, so you should be fine.

> [!NOTE]
> Since the wrapper uses glibc builds of PicoLimbo, Alpine Linux is **not supported** by the wrapper.

### Installation on Proxies

Simply drag the jar file into your proxy's `plugins` folder and start the proxy. The plugin will create a configuration file at `plugins/pico_limbo_java_wrapper/server.toml`. Installation is identical for both BungeeCord and Velocity.

#### Configuring the Listening Address

PicoLimbo requires manual configuration after the first startup. Follow these steps:

1. Start the proxy once to generate the configuration file
2. Open `plugins/pico_limbo_java_wrapper/server.toml`
3. Update the `bind` address to a free port on localhost (e.g., `127.0.0.1:30066`)
    - Use a different port than your proxy's listening address to avoid conflicts
4. Configure your proxy to route players to this address
5. Set up [forwarding](../config/proxy-integration) if using a proxy

> [!NOTE]
> Automatic configuration and registration when running inside a proxy may be added as a feature in future versions.

### Standalone Installation

To run PicoLimbo without a proxy, simply execute:

::: code-group

```shell [Terminal]
java -jar pico_limbo_java_wrapper.jar
```

:::

The configuration file will be created in `server.toml`. You can configure forwarding using the same methods as the proxy installation if needed.

## Compiling from Source

You can compile PicoLimbo from source using either Cargo or Git:

### Using Cargo

To install PicoLimbo directly from the repository using Cargo:

```bash
cargo install --git https://github.com/Quozul/PicoLimbo.git pico_limbo
```

The binary will be installed to your Cargo bin directory (typically `~/.cargo/bin/pico_limbo`). Make sure this directory is in your PATH to run the command from anywhere:

```bash
# Run PicoLimbo
pico_limbo

# Or with full path if not in PATH
~/.cargo/bin/pico_limbo
```

> [!NOTE]
> This method requires Rust and Cargo to be installed on your system. If you don't have them installed,
> visit [rustup.rs](https://rustup.rs/) for installation instructions.

### Using Git

To clone and build PicoLimbo from source:

1. First, install Git and Rust (with Cargo) if you haven't already
2. Clone the repository:
   ```bash
   git clone https://github.com/Quozul/PicoLimbo.git
   cd PicoLimbo
   ```

3. Build the project:
   ```bash
   cargo build --release
   ```

4. The compiled binary will be in the `target/release` directory. You can run it with:
   ```bash
   ./target/release/pico_limbo
   ```
