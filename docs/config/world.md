# World Configuration

Representing the `[world]` section in `server.toml`.

## Dimension

Default spawn dimension for new players.

:::code-group
```toml [server.toml] {2}
[world]
dimension = "overworld"
```
:::

Possible values:
```
overworld
nether
end
```

## Spawn Position

Customize where players spawn using `[x, y, z]` coordinates. Supports floating point numbers.

:::code-group
```toml [server.toml] {2}
[world]
spawn_position = [0.5, 320.0, 0.5]
```
:::

## Spawn Rotation

Customize where players look at spawn using `[yaw, pitch]` coordinates. Supports floating point numbers.

:::code-group
```toml [server.toml] {2}
[world]
spawn_rotation = [90.0, 90.0]
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

## Time

Sets the time in the world.

:::code-group
```toml [server.toml] {2}
[world]
time = "midnight"
```
:::

Possible values:
```
day
noon
night
midnight
a specific time in ticks (0-24000)
```

## Lock Time <Badge type="warning" text="1.21.5+" />

Set to `false` to prevent the client from ticking the time.

:::code-group
```toml [server.toml] {2}
[world.experimental]
lock_time = false
```
:::
