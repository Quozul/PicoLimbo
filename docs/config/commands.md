# Commands

Representing the `[commands]` section in `server.toml`.

## Overview

PicoLimbo currently supports one command that allows players to teleport back to the spawn point.

## Spawn Command

The spawn command teleports the player back to the spawn location.

:::code-group
```toml [server.toml] {2}
[commands]
spawn = "spawn"
```
:::

### Enable

To enable the spawn command with the default name:

:::code-group
```toml [server.toml] {2}
[commands]
spawn = "spawn"
```
:::

### Rename

You can rename the spawn command to a custom alias:

:::code-group
```toml [server.toml] {2}
[commands]
spawn = "limbo"
```
:::

Players would then use `/limbo` instead of `/spawn` to teleport back to spawn.

### Disable

To disable the spawn command completely, set it to an empty string:

:::code-group
```toml [server.toml] {2}
[commands]
spawn = ""
```
:::

When disabled, players will not be able to the spawn command.
