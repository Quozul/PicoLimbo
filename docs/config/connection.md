# Connection Settings

## Keep Alive Interval <Badge type="warning" text="1.8+" />

Interval, in seconds, between two `keep_alive` packets sent to a client.
The default value of `15` matches the vanilla server.
Lower it (e.g. `10`) if a proxy in front of PicoLimbo has a stricter read-timeout, or if your players have unreliable connections that benefit from more frequent pings.

:::code-group
```toml [server.toml]
[connection]
keep_alive_interval_seconds = 15
```
:::

> [!NOTE]
> Clients on Minecraft 1.7.x use a fixed 2-second ping required by the legacy protocol, regardless of this setting.

## Allow Unsupported Versions

By default (`false`), clients whose protocol version is not explicitly supported by PicoLimbo are rejected during login.

When set to `true`, PicoLimbo instead guesses the closest protocol implementation it knows:

- Protocol numbers not yet explicitly supported (snapshots, pre-releases, release candidates and brand-new stable releases) are served with the latest supported implementation.
- Protocol numbers older than every supported version are served with the oldest supported implementation.

This is mainly useful to connect with snapshots: for example, a `26.3.1` snapshot will likely share the protocol implementation of stable `26.3`, and PicoLimbo will serve it as such, before official support lands.

> [!WARNING]
> There is no guarantee that the guessed implementation is compatible. If Mojang changed the protocol, clients may fail to join, get kicked or crash. This setting is not a replacement for a PicoLimbo update: when a new Minecraft version releases, it is recommended to wait for a PicoLimbo version that explicitly supports it.

:::code-group
```toml [server.toml]
[connection]
allow_unsupported_versions = false
```
:::
