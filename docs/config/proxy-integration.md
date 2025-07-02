# Proxy Integration

PicoLimbo is compatible with popular Minecraft proxies, such as Velocity and BungeeCord, to manage player connections and routing.

> [!TIP]
> Velocity is recommended for most server networks. Velocity is modern and more secure compared to BungeeCord/BungeeGuard.

## Velocity Modern Forwarding

Velocity Modern Forwarding is a method of forwarding player connections using the Velocity proxy. To enable Velocity Modern Forwarding, set the following configuration options:

:::code-group
```toml [server.toml] {2-3}
[forwarding.velocity]
enabled = true
secret = "<your-secret>"
```
:::

Replace `<your-secret>` with the forwarding secret of your Velocity proxy.

## BungeeCord Legacy Forwarding

BungeeCord Legacy Forwarding is a method of forwarding player connections using the BungeeCord proxy. To enable BungeeCord forwarding, set the following configuration options:

:::code-group
```toml [server.toml] {2}
[forwarding.bungee_cord]
enabled = true
```
:::

## BungeeGuard Authentication

BungeeGuard and BungeeGuardPlus are additional security features for BungeeCord that provide token-based authentication for incoming player connections. To enable BungeeGuard authentication, set the following configuration options:

:::code-group
```toml [server.toml] {3-5}
[forwarding.bungee_cord]
enabled = true
bungee_guard = true
tokens = ["<token1>", "<token2>", ...]
```
:::

Replace `<token1>`, `<token2>`, etc., with valid BungeeGuard tokens for your BungeeCord proxy.
