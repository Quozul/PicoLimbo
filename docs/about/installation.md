# Installation

## Pterodactyl (Recommended)

For users running the Pterodactyl panel, deployment is simplified with the provided egg file. This egg is built on the lightweight Alpine base image.

The egg supports additional installation configuration through the following environment variables:

- **VERSION**
  Specifies the Git tag of the release to install. This can be a stable or prerelease tag (e.g., `v1.4.0+mc1.21.8`).
    - Default: `latest`
    - When set to `latest` (or left unset without enabling prerelease), the installer selects the newest stable release.

- **USE_PRERELEASE**
  When set to `true`, the installer ignores stable releases and installs the newest prerelease based on publication date.
    - Default: *(empty)*

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

Download pre-compiled binaries for multiple platforms from the [GitHub releases page](https://github.com/Quozul/PicoLimbo/releases).

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
