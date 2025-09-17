# World Configuration

PicoLimbo includes experimental world features that allow you to customize the spawn environment and load a custom structure using schematic files.

> [!WARNING]
> This feature is work in progress and **only works with Minecraft client version 1.16 and above** as of now. It may
> cause crashes or instability. While bug reports are welcome, expect issues and test thoroughly before production use.

![Limbo's loaded from a schematic file](/world.png)
> Loading of Loohp's Limbo [spawn.schem](https://github.com/LOOHP/Limbo/blob/master/spawn.schem) file inside PicoLimbo.

## Schematic Loading

Load `.schem` files to customize the spawn location. PicoLimbo implements version 2 of
[SpongePowered's schematic specification](https://github.com/SpongePowered/Schematic-Specification).

:::code-group
```toml [server.toml] {2}
[world.experimental]
schematic_file = "spawn.schem"
```
:::

The schematic will be loaded with its minimum corner placed at world coordinates 0,0,0, extending in the positive x, y, and z directions.

You can create compatible schematic files using WorldEdit with the following command:

```
//schem save <filename> sponge.2
```

To disable schematic loading:

:::code-group
```toml [server.toml] {2}
[world.experimental]
schematic_file = ""
```
:::

### Known Limitations

Here's a list of what does not work when loading a schematic:
- **Block entities**: Chests, signs, banners, player heads, and other tile entities
- **Entities**: Armor stands, item frames, mobs, and other entities
- **Light engine**: The world will always be fully lit
- **Movement mechanics**: Ladder climbing or elytra does not work
- **Block interactions**: Opening a door only half-opens it, buttons and pressure plates does not reset

## View Distance

Configure how many chunks are sent to clients. Defaults to 2. The view distance should match or exceed your schematic's size in chunks.

:::code-group
```toml [server.toml] {2}
[world.experimental]
view_distance = 2
```
:::

## Lock Time

Set to `false` to prevent the client from ticking the time. This only works with Minecraft client version 1.21.5 and above.

:::code-group
```toml [server.toml] {2}
[world.experimental]
lock_time = false
```
:::

## World Boundaries

Control player movement by setting a minimum Y coordinate. When players fall below this level, they'll be teleported back to spawn and receive a configurable message.

### Minimum Y Position

Set the lowest Y coordinate players can reach before being teleported back to spawn. Defaults to -64 (Minecraft's default world bottom).

:::code-group
```toml [server.toml] {2-3}
[world.boundaries]
enabled = true
min_y_pos = -64
```
:::

### Minimum Y Message

Customize the message players receive when they fall below the minimum Y position and are teleported back to spawn. Supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.

:::code-group
```toml [server.toml] {2}
[world.boundaries]
teleport_message = "<red>You have reached the bottom of the world.</red>"
```
:::

The message can be disabled by setting an empty string:

:::code-group
```toml [server.toml] {2}
[world.boundaries]
teleport_message = ""
```
:::
