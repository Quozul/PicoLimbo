# World Configuration

PicoLimbo includes experimental world features that allow you to customize the spawn environment and load custom structures using schematic files.

> [!WARNING]
> This feature is work in progress and may not work with all Minecraft versions. It may cause crashes or instability.
> While bug reports are welcome, expect issues and test thoroughly before production use.

![Limbo's loaded from a schematic file](/world.png)
> Loading of Loohp's Limbo [spawn.schem](https://github.com/LOOHP/Limbo/blob/master/spawn.schem) file inside PicoLimbo.

## Schematic Loading

Load `.schem` files to provide custom world structures. PicoLimbo
follows [SpongePowered's schematic specification](https://github.com/SpongePowered/Schematic-Specification):

:::code-group
```toml [server.toml] {2}
[experimental.world]
schematic_file = "spawn.schem"
```
:::

To disable schematic loading:

:::code-group
```toml [server.toml] {2}
[experimental.world]
schematic_file = ""
```
:::

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

## Version-Specific Behavior

### Minecraft 1.19 - 1.20.2

No chunks are sent to the client. Players must spawn above y=320 (outside world bounds) to avoid getting stuck on the
loading screen.

### Minecraft 1.20.3+

An empty chunk at position 0,0 is sent to the client. Players must either:

- Spawn above y=320 (outside world bounds), or
- Spawn within chunk 0,0 boundaries (x: 0-15, z: 0-15) and the view distance is greater than 0

## Recommended Configurations

**With schematic file:**

:::code-group
```toml [server.toml] {2-4}
[experimental.world]
spawn_position = [8.0, 64.0, 8.0]
schematic_file = "lobby.schem"
view_distance = 4
```
:::

**Without schematic file:**

:::code-group
```toml [server.toml] {2-3}
[experimental.world]
spawn_position = [0.0, 320.0, 0.0]  # Within chunk 0,0, above world bounds
schematic_file = ""
```
:::
