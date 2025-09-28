# Installation

## Pterodactyl (Recommended)

For users running the Pterodactyl panel, deployment is simplified with the provided egg file. This egg is built on the lightweight Alpine base image.

You can find the egg file in the GitHub repository: [egg-pico-limbo.json](https://github.com/Quozul/PicoLimbo/blob/master/pterodactyl/eggs/egg-pico-limbo.json)

The egg supports additional installation configuration through the following environment variable:

- **VERSION**
  Specifies the Git tag of the release to install (e.g., `v1.6.0+mc1.21.9`).
    - Default: `latest`
    - When set to `latest` (or left unset), the installer selects the newest stable release.

> [!WARNING]
> Do not manually upload binary files to your Pterodactyl server. This will not work properly. To update PicoLimbo, you
> must re-install the server through Pterodactyl's installation process.

## Using Docker

The Docker image is multi-platform, supporting both Linux/amd64 and Linux/arm64 architectures. You can start the server using the following command:

```shell
docker run --rm -p "25565:25565" ghcr.io/quozul/picolimbo:latest
```

You can also mount a custom configuration file:

```shell
docker run --rm -p "25565:25565" -v /path/to/your/server.toml:/usr/src/app/server.toml ghcr.io/quozul/picolimbo:latest
```

### Using Docker Compose

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

- **`pico_limbo_linux-x86_64-musl.tar.gz`** - Linux systems with Intel/AMD 64-bit processors (most common)
- **`pico_limbo_linux-aarch64-musl.tar.gz`** - Linux systems with ARM 64-bit processors (e.g., Raspberry Pi 4+, Apple Silicon under emulation)
- **`pico_limbo_macos-aarch64.tar.gz`** - macOS with Apple Silicon (M1/M2/M3 chips)
- **`pico_limbo_windows-x86_64.zip`** - Windows with Intel/AMD 64-bit processors

#### Manual Installation

1. **Download** the appropriate binary for your system from the releases page
2. **Extract** the archive:
    - **Linux/macOS**: `tar -xzf pico_limbo_*.tar.gz`
    - **Windows**: Use your preferred archive tool or built-in extraction
3. **Run** the binary:
    - **Linux/macOS**: `./pico_limbo`
    - **Windows**: Double-click `pico_limbo.exe` or run it from Command Prompt

> [!TIP]
> On Linux systems, you may want to move the binary to a directory in your PATH (like `/usr/local/bin/`) to run it from anywhere, or make it executable with `chmod +x pico_limbo` if needed.

### Compiling from Source

You can compile PicoLimbo from source using either Cargo or Git:

#### Using Cargo

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

#### Using Git

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
