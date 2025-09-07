# World Configuration

## Spawn Dimension

Default spawn dimension for new players.

:::code-group
```toml [server.toml] {2}
[world]
spawn_dimension = "overworld"
```
:::

Possible values:
```
overworld
nether
end
```

## Time in the world

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

## Lock Time

Lock the time in the world to the `time` value.

> [!WARNING]
> This feature **only works with Minecraft client version 1.21.5 and above**.

:::code-group
```toml [server.toml] {2}
[world]
lock_time = true
```
:::