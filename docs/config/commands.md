# Commands

Representing the `[commands]` section in `server.toml`.

## Spawn Command

The `/spawn` command teleports the player to the server's spawn location. You can customize the command alias or disable it entirely.

:::code-group
```toml [server.toml] {2}
[commands]
spawn = "spawn"
```
:::

## Fly Command

The `/fly` command allows players to toggle flight on and off. This command is not affected by the Allow Flight setting.

:::code-group
```toml [server.toml] {2}
[commands]
fly = "fly"
```
:::

## Fly Speed Command

The `/flyspeed` command allows players to adjust their flight speed with a `speed` float argument. The speed value must be between `0.0` and `1.0`.

:::code-group
```toml [server.toml] {2}
[commands]
fly_speed = "flyspeed"
```
:::

## Transfer Command <Badge type="warning" text="1.20.5+" />

The `/transfer` command allows players to transfer to another server by specifying its `hostname` and optionally a `port`. If a port is not specified the Minecraft default of 25565 is used. 

> [!NOTE]
> The destination server must have [accepts-transfers](https://minecraft.wiki/w/Server.properties#Keys) set to `true` in its server.properties.

:::code-group
```toml [server.toml] {2}
[commands]
transfer = "transfer"
```
:::

## Disabling Commands

Any command can be disabled by setting its value to an empty string `""`. This prevents players from using that command entirely.

:::code-group
```toml [server.toml] {2}
[commands]
spawn = ""
fly = "fly"
fly_speed = ""
transfer = ""
```
:::

## Renaming Commands

You can rename any command to a custom alias by changing its value. For example, you could rename multiple commands for your server's theme or language preferences.

:::code-group
```toml [server.toml] {2}
[commands]
spawn = "home"
fly = "soar"
fly_speed = "speed"
transfer = "server"
```
:::