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

If set to true, PicoLimbo will attempt to use the latest protocol version for unsupported versions. Useful for snapshots.

:::code-group
```toml [server.toml]
[connection]
allow_unsupported_versions = false
```
:::
