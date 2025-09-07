# Proxy Integration

PicoLimbo is compatible with popular Minecraft proxies, such as Velocity and BungeeCord, to manage player connections and routing.

> [!TIP]
> Velocity is recommended for most server networks. Velocity is modern and more secure compared to BungeeCord/BungeeGuard.

## Velocity Modern Forwarding

Velocity Modern Forwarding is a method of forwarding player connections using the Velocity proxy. To enable Velocity Modern Forwarding, set the following configuration options:

:::code-group
```toml [server.toml] {2-3}
[forwarding]
method = "MODERN"
secret = "<your-secret>"
```
:::

Replace `<your-secret>` with the forwarding secret of your Velocity proxy.

## BungeeGuard Authentication

BungeeGuard is an additional security feature that provide token-based authentication for incoming player connections. To enable BungeeGuard authentication, set the following configuration options:

:::code-group
```toml [server.toml] {2-3}
[forwarding]
method = "BUNGEE_GUARD"
tokens = ["<token1>", "<token2>", ...]
```
:::

Replace `<token1>`, `<token2>`, etc., with valid BungeeGuard tokens for your BungeeCord proxy.

## BungeeCord Legacy Forwarding

To enable BungeeCord forwarding, set the following configuration options:

:::code-group
```toml [server.toml] {2}
[forwarding]
method = "LEGACY"
```
:::

## None

To disable forwarding altogether:

:::code-group
```toml [server.toml] {2}
[forwarding]
method = "NONE"
```
:::
