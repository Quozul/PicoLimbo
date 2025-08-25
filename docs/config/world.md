# World Configuration

PicoLimbo includes experimental world features that allow you to customize the spawn position.

> [!WARNING]
> This feature is work in progress and **only works with Minecraft client version 1.19 and above** as of now. It may
> cause crashes or instability. While bug reports are welcome, expect issues and test thoroughly before production use.

## Spawn Position

Customize where players spawn using `[x, y, z]` coordinates. Supports floating point numbers:

:::code-group
```toml [server.toml] {2}
[experimental.world]
spawn_position = [0.5, 320.0, 0.5]
```
:::

## View Distance

Configure how many chunks are sent to clients. Defaults to 2, with a range of 0-32. Values outside this range are clamped. The view distance should match or exceed your schematic's size in chunks.

:::code-group
```toml [server.toml] {2}
[experimental.world]
view_distance = 2
```
:::
