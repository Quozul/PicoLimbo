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
