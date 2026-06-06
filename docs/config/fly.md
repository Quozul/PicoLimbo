# Fly

Representing the `[fly]` section in `server.toml`.

## Allow Flight

Whether players can use flight on the server. When set to `false`, the server will not allow players to fly unless they are in spectator mode.

:::code-group
```toml [server.toml] {2}
[fly]
allow_flight = false
```
:::

## Flying

Whether players start in a flying state when they join the server.

> [!NOTE]
> The player can still toggle fly using the `/fly` command. Please refer to the [command documentation](./commands.md) to learn how to disable it.

:::code-group
```toml [server.toml] {3}
[fly]
allow_flight = false
flying = true
```
:::

## Flying Speed

The initial flying speed for players. The value must be between `0.0` (slowest) and `1.0` (fastest).

> [!NOTE]
> The player can still change the flying speed using the `/flyspeed` command. Please refer to the [command documentation](./commands.md) to learn how to disable it.

:::code-group
```toml [server.toml] {4}
[fly]
allow_flight = false
flying = true
flying_speed = 0.05
```
:::

## Tutorials

### Prevent Falling in Survival/Creative/Adventure

To players from falling, set `allow_flight = false` and `flying = true`.

:::code-group
```toml [server.toml] {2}
default_game_mode = "adventure"

[fly]
allow_flight = false
flying = true
flying_speed = 0.05

# Disable the fly comands so that the player cannot use them to stop flying
[commands]
fly = ""
fly_speed = ""
```
:::

### Prevent All Movement

To freeze players in place, set `allow_flight = false`, `flying = true`, and `flying_speed = 0.0`.

:::code-group
```toml [server.toml] {2}
# It is recommended to set the default game mode to spectator with this configuration
default_game_mode = "spectator"

[fly]
allow_flight = false
flying = true
flying_speed = 0.0

# Disable the fly comands so that the player cannot use them to stop flying
[commands]
fly = ""
fly_speed = ""
```
:::
