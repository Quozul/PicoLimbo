# World Configuration

PicoLimbo includes experimental world features that allow you to customize the spawn position.

> [!WARNING]
> This feature is work in progress and may not work with all Minecraft versions. It may cause crashes or instability.
> While bug reports are welcome, expect issues and test thoroughly before production use.

## Spawn Position

Customize where players spawn using `[x, y, z]` coordinates. Supports floating point numbers:

:::code-group
```toml [server.toml] {2}
[experimental.world]
spawn_position = [0.5, 320.0, 0.5]
```
:::

## Version-Specific Behavior

### Minecraft 1.19 - 1.20.2

No chunks are sent to the client. Players must spawn above y=320 (outside world bounds) to avoid getting stuck on the
loading screen.

### Minecraft 1.20.3+

An empty chunk at position 0,0 is sent to the client. Players must either:

- Spawn above y=320 (outside world bounds), or
- Spawn within chunk 0,0 boundaries (x: 0-15, z: 0-15)
