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

### Logging Options

Control the verbosity of server logs:

```bash
# Detailed debug logging
pico_limbo -v

# Trace-level logging (most verbose)
pico_limbo -vv
```

### Port override

Override the port configured in the configuration file:

```bash
pico_limbo --port 30066
```

## Advanced Options

### Version Information

Display version information:

```bash
pico_limbo --version
```

### Hide Banner

When PicoLimbo starts, a banner is displayed to inform users of the server version.
You can hide this banner by passing the `--skip-banner` flag:

```bash
pico_limbo --skip-banner
```

### Help

Show all available options:

```bash
pico_limbo --help
```
