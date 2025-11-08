# Using PicoLimbo as a Fallback Server for Your Velocity Proxy Network

![Automatic reconnect using Velocity Limbo Handler](/tutorial-automatic-reconnect.gif)

This guide will walk you through setting up PicoLimbo as a fallback server for your Velocity proxy network. We'll cover both basic fallback functionality and automatic reconnection methods.

## Basic Fallback Configuration

Velocity's built-in fallback feature will redirect players to the limbo server when they are kicked or when the main server crashes. From the limbo server, players can manually reconnect using the `/server` Velocity command.

### Server Setup

Assume we have the following servers:
- Velocity proxy on port 25565
- Paper server on port 30066
- PicoLimbo on port 30067

### Configuration

Configure your `velocity.toml` file as follows:

```toml [velocity.toml] {5}
[servers]
survival = "127.0.0.1:30066"
limbo = "127.0.0.1:30067"

try = ["survival", "limbo"]
```

### Forced Hosts Configuration

The same configuration can be applied to forced hosts:

```toml [velocity.toml] {8}
[servers]
survival = "127.0.0.1:30066"
limbo = "127.0.0.1:30067"

try = ["survival", "limbo"]

[forced-hosts]
"survival.example.com" = ["survival", "limbo"]
```

## Automatic Reconnection

Automatic reconnection is superior to basic fallback as it attempts to reconnect players to the main server at regular intervals, useful after a crash.

### Option 1: Velocity Limbo Handler

[Velocity Limbo Handler](https://modrinth.com/plugin/velocity-limbo-handler) is a popular plugin for automatic reconnection.

Configure your `config.yml` file:

```yaml [config.yml] {1,2}
limbo-name: limbo
direct-connect-server: survival
task-interval: 3
queue-enabled: true
queue-notify-interval: 30
disabled-commands:
- server
- lobby
- hub
auth-timeout-seconds: 120
```

### Option 2: VelocityAutoReconnect

[VelocityAutoReconnect](https://github.com/flori4nk/VelocityAutoReconnect) is an alternative plugin that still works well despite not being updated recently.

Configure your `velocityautoreconnect.conf` file:

```properties [velocityautoreconnect.conf] {2,7}
bypasscheck=false
directconnect-server=survival
kick-filter.blacklist=.* ([Bb]anned|[Kk]icked|[Ww]hitelist).*
kick-filter.blacklist.enabled=true
kick-filter.whitelist=Server closed
kick-filter.whitelist.enabled=false
limbo-name=limbo
log.informational=true
message=You will be reconnected soon.
message.enabled=false
pingcheck=true
task-interval-ms=3500
```
