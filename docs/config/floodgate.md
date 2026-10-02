# Floodgate

Representing the `[floodgate]` section in `server.toml`.

PicoLimbo supports Floodgate for Bedrock Edition players and EduFloodgate for Education Edition players. Floodgate is handled by the proxy, while PicoLimbo uses the shared `key.pem` to accept the forwarded player data.

## Enabled

Floodgate support is disabled by default.

:::code-group
```toml [server.toml] {2}
[floodgate]
enabled = true
```
:::

When enabled, PicoLimbo uses the other Floodgate settings with their default values unless you override them.

## Key File

The `key_file` setting specifies the Floodgate key file used by PicoLimbo.

:::code-group
```toml [server.toml] {3}
[floodgate]
enabled = true
key_file = "key.pem"
```
:::

The path is relative to the PicoLimbo server's working directory. It is **not** a path to a file on your proxy.

For example, if your PicoLimbo server is running with this layout:

```text
PicoLimbo/
├── server.toml
├── key.pem
└── PicoLimbo
```

then `key_file = "key.pem"` will use the `key.pem` stored beside `server.toml`.

Copy the `key.pem` from your Floodgate proxy to the PicoLimbo server, then point `key_file` to the local copy. The file contents must be the same key used by the proxy.

> [!WARNING]
> Keep your Floodgate key private. Anyone who obtains it can forge Floodgate player data.

See the [Floodgate proxy setup](https://geysermc.org/wiki/floodgate/setup/proxy-servers/) documentation for the proxy-side setup.

## Username Prefix

The prefix added to normal Bedrock usernames.

:::code-group
```toml [server.toml] {3}
[floodgate]
enabled = true
username_prefix = "."
```
:::

The default is `.`.

For example, a Bedrock player named `Steve` will appear as `.Steve`.

## Replace Spaces

Whether spaces in Bedrock usernames are replaced with underscores.

:::code-group
```toml [server.toml] {3}
[floodgate]
enabled = true
replace_spaces = true
```
:::

The default is `true`.

## Education Edition

Enable support for players connecting from Minecraft Education Edition through EduGeyser/EduFloodgate.

:::code-group
```toml [server.toml] {3}
[floodgate]
enabled = true
education = true
```
:::

Normal Bedrock players continue to work when Education support is enabled.

## Education Username Prefix

The prefix added to Education Edition usernames.

:::code-group
```toml [server.toml] {3}
[floodgate]
enabled = true
education_username_prefix = "+"
```
:::

The default is `+`.

## Education UUID Scheme

Select which UUID scheme PicoLimbo uses for Education Edition players.

:::code-group
```toml [server.toml] {3}
[floodgate]
enabled = true
education_uuid_legacy = false
```
:::

The default is `false`, which uses the current Education UUID scheme. Set it to `true` to use the legacy scheme.

> [!WARNING]
> This setting must match the UUID scheme configured by EduGeyser/EduFloodgate. Changing it can change the UUID used for Education Edition players and may affect existing player data.

## Proxy Setup

Floodgate must be configured on the proxy that handles Bedrock connections.

On the proxy:

1. Install and configure Geyser with Floodgate authentication.
2. Enable `send-floodgate-data` in the Floodgate configuration.
3. Copy the proxy's `key.pem` to the PicoLimbo server.
4. Set `key_file` to the local copy on PicoLimbo.

The proxy and PicoLimbo must use the same Floodgate key.

See the [Floodgate proxy setup](https://geysermc.org/wiki/floodgate/setup/proxy-servers/) documentation for the proxy configuration.

## Velocity Modern Forwarding

Floodgate is compatible with Velocity Modern Forwarding.

Configure Velocity Modern Forwarding as described in [Proxy Integration](./proxy-integration.html). Floodgate and Velocity forwarding can be enabled together.

