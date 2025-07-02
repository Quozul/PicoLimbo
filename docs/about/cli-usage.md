# Command Line Interface (CLI) Usage

PicoLimbo offers a flexible command line interface for running and configuring the server in standalone mode. This page covers all available CLI options and their usage.

## Basic Usage

To start the server with default settings:

```bash
pico_limbo
```

## Configuration Options

### Custom Configuration File

Specify a custom configuration file path:

```bash
pico_limbo --config /path/to/your/config.toml
```

### Data Directory

Set the directory containing packet maps and game registries:

```bash
pico_limbo --data-dir /path/to/your/assets
```

By default, PicoLimbo looks for assets in the `./assets` directory relative to where the binary is executed. This option allows you to specify a different location for these essential files.

If you are running PicoLimbo from the Git repository, you have to specify where are the assets located. The assets are located in the `data/generated` directory relative to the repository's root. See [installation with Git](./installation.html#using-git) for more information.

### Logging Options

Control the verbosity of server logs:

```bash
# Detailed debug logging
pico_limbo -v

# Trace-level logging (most verbose)
pico_limbo -vv
```

## Advanced Options

### Version Information

Display version information:

```bash
pico_limbo --version
```

### Help

Show all available options:

```bash
pico_limbo --help
```
