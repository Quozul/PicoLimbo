# World Configuration

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
