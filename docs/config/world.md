# World Configuration

PicoLimbo includes experimental world features that allow you to customize the spawn environment and load custom structures using schematic files.

> [!WARNING]
> This feature is work in progress and **only works with Minecraft client version 1.19 and above** as of now. It may
> cause crashes or instability. While bug reports are welcome, expect issues and test thoroughly before production use.

![Limbo's loaded from a schematic file](/world.png)
> Loading of Loohp's Limbo [spawn.schem](https://github.com/LOOHP/Limbo/blob/master/spawn.schem) file inside PicoLimbo.

## Schematic Loading

Load `.schem` files to provide custom world structures. PicoLimbo implements version 2 of
[SpongePowered's schematic specification](https://github.com/SpongePowered/Schematic-Specification).

:::code-group
```toml [server.toml] {2}
[experimental.world]
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
[experimental.world]
schematic_file = ""
```
:::

### Known Limitations

Here's a list of what does not work when loading a schematic:
- **Block entities**: Chests, signs, banners, player heads, and other tile entities
- **Entities**: Armor stands, item frames, mobs, and other entities
- **Light engine**: The world will always be fully lit
- **Movement mechanics**: Ladder climbing seems to work only in 1.21.8
- **Block interactions**: Opening a door only half-opens it, buttons and pressure plates does not reset

## Spawn Position

Customize where players spawn using `[x, y, z]` coordinates. Supports floating point numbers.

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
