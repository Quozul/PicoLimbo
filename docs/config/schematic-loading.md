# Schematic Loading <Badge type="warning" text="1.16+" />

PicoLimbo includes experimental world features that allow you to customize the spawn environment and load a custom structure using schematic files.

> [!WARNING]
> This feature is work in progress and **only works with Minecraft client version 1.16 and above** as of now. It may
> cause crashes or instability. While bug reports are welcome, expect issues and test thoroughly before production use.
> Work on getting it to work for older version is still in progress.

![Limbo's loaded from a schematic file](/world.png)
> Loading of Loohp's Limbo [spawn.schem](https://github.com/LOOHP/Limbo/blob/master/spawn.schem) file inside PicoLimbo.

## Schematic File

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
- **Entities**: Armor stands, item frames, mobs, and other entities
- **Light engine**: The world will always be fully lit
- **Movement mechanics**: Ladder climbing or elytra does not work
- **Block interactions**: Opening a door only half-opens it, buttons and pressure plates does not reset
- **Unknown blocks**: If your client doesn't support certain blocks (like newer ones or renamed types such as `grass` → `short_grass` in 1.20.3), they’ll appear as stone blocks instead

## View Distance

Configure how many chunks are sent to clients. Defaults to 2. The view distance should match or exceed your schematic's size in chunks.

:::code-group
```toml [server.toml] {2}
[world.experimental]
view_distance = 2
```
:::
