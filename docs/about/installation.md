# Installation

## Pterodactyl (Recommended)

For users running the Pterodactyl panel, deployment is simplified with the provided egg file. This egg is built on the lightweight Alpine base image.

The egg supports additional installation configuration through the following environment variables:

- **VERSION**
  Specifies the Git tag of the release to install. This can be a stable or prerelease tag (e.g., `v1.21.8`).
    - Default: `latest`
    - When set to `latest` (or left unset without enabling prerelease), the installer selects the newest stable release.

- **USE_PRERELEASE**
  When set to `true`, the installer ignores stable releases and installs the newest prerelease based on publication date.
    - Default: *(empty)*

## Using Docker

The Docker image is multi-platform, supporting both Linux/amd64 and Linux/arm64 architectures. You can start the server using the following command:

```shell
docker run --rm -p "25565:25565" ghcr.io/quozul/picolimbo:master
```

You can also mount a custom configuration file:

```shell
docker run --rm -p "25565:25565" -v /path/to/your/server.toml:/usr/src/app/server.toml ghcr.io/quozul/picolimbo:master
```

> [!NOTE]
> The `master` tag image is updated on every push to the repository. For production or stable setups, consider using a fixed version tag instead.

### Using Docker Compose

Here's the complete docker-compose.yml file:

```yaml
services:
  picolimbo:
    image: ghcr.io/quozul/picolimbo:master
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

Download pre-compiled binaries for multiple platforms from the GitHub releases page. These releases include the required assets directory in the compressed archive (tar.gz for Unix and zip for Windows).

### Compiling from Source

> [!IMPORTANT]
> For manual setups (using Cargo or Git), you must download the "data/generated" directory from the GitHub repository, rename it to "assets", and place it alongside the binary. This directory contains essential files required for server execution.
>
> The path to the assets directory can be customized using command line arguments. For more details on these arguments, please refer to the [Command Line Interface (CLI) Usage documentation](./cli-usage).

You can compile PicoLimbo from source using either Cargo or Git:

#### Using Cargo

To build PicoLimbo from source using Cargo:

```bash
cargo install --git https://github.com/Quozul/PicoLimbo.git pico_limbo
```

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
   ./target/release/pico_limbo --data-dir data/generated
   ```
