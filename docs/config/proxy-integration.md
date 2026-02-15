# Proxy Integration

PicoLimbo is compatible with popular Minecraft proxies, such as Velocity and BungeeCord, to manage player connections and routing.

> [!TIP]
> Velocity is recommended for most server networks. Velocity is modern and more secure compared to BungeeCord/BungeeGuard.

## Velocity Modern Forwarding <Badge type="warning" text="1.13+" />

Velocity Modern Forwarding is a method of forwarding player connections using the Velocity proxy. To enable Velocity Modern Forwarding, set the following configuration options:

:::code-group

```toml [server.toml] {4-5}
bind = "127.0.0.1:30066"

[forwarding]
method = "MODERN"
secret = "sup3r-s3cr3t"
```

```toml [velocity.toml]
player-info-forwarding-mode = "modern"

[servers]
limbo = "127.0.0.1:30066"
```

```text [forwarding.secret]
sup3r-s3cr3t
```

:::

> [!TIP]
> You can use environment variables to store the forwarding secret.
>
> :::code-group
> 
> ```toml [server.toml] {3}
> [forwarding]
> method = "MODERN"
> secret = "${FORWARDING_SECRET}"
> ```
> 
> :::

## BungeeGuard Authentication

BungeeGuard is an additional security feature that provide token-based authentication for incoming player connections. To enable BungeeGuard authentication, set the following configuration options:

### On Velocity

:::code-group

```toml [server.toml] {4-5}
bind = "127.0.0.1:30066"

[forwarding]
method = "BUNGEE_GUARD"
tokens = ["sup3r-s3cr3t"]
```

```toml [velocity.toml]
player-info-forwarding-mode = "bungeeguard"

[servers]
limbo = "127.0.0.1:30066"
```

```text [forwarding.secret]
sup3r-s3cr3t
```

:::

### On BungeeCord

First, install the [BungeeGuard](https://www.spigotmc.org/resources/bungeeguard.79601/) or [BungeeGuardPlus](https://github.com/nickuc-com/BungeeGuardPlus) plugin.

:::code-group

```toml [server.toml] {4-5}
bind = "127.0.0.1:30066"

[forwarding]
method = "BUNGEE_GUARD"
tokens = ["sup3r-s3cr3t"]
```

```yaml [config.yml]
ip_forward: true
servers:
  limbo:
    address: 127.0.0.1:30066
```

```yaml [plugins/BungeeGuard/token.yml]
token: sup3r-s3cr3t
```

:::

## BungeeCord Legacy Forwarding

To enable BungeeCord forwarding, set the following configuration options:

### On Velocity

:::code-group

```toml [server.toml] {4}
bind = "127.0.0.1:30066"

[forwarding]
method = "LEGACY"
```

```toml [velocity.toml]
player-info-forwarding-mode = "legacy"

[servers]
limbo = "127.0.0.1:30066"
```

:::

### On BungeeCord

:::code-group

```toml [server.toml] {4}
bind = "127.0.0.1:30066"

[forwarding]
method = "LEGACY"
```

```yaml [config.yml]
ip_forward: true
servers:
  limbo:
    address: 127.0.0.1:30066
```

:::

## No Forwarding

To disable forwarding:

:::code-group

```toml [server.toml] {4}
bind = "127.0.0.1:30066"

[forwarding]
method = "NONE"
```

:::

Use this only if PicoLimbo is running standalone or for testing purposes.
