# World Configuration

## Spawn Dimension

Default spawn dimension for new players.

:::code-group
```toml [server.toml]
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
```toml [server.toml]
time_world = "midnight"
```
:::

Possible values:
```
sunrise
noon
sunset
midnight
a specific time in ticks (0-24000)
```

## Lock Time

Lock the time in the world to `time_world` value.

> [!WARNING]
> This feature **only works with Minecraft client version 1.21.5 and above**

:::code-group
```toml [server.toml]
lock_time = true
```
:::