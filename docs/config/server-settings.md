# Server Settings

Representing `server.toml`

## Server Address

The address to bind the server to.

:::code-group
```toml [server.toml]
bind = "0.0.0.0:25565"
```
:::

## Welcome Message

Welcome message displayed to players after joining.
Supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.

:::code-group
```toml [server.toml]
welcome_message = "<green>Welcome to <bold>PicoLimbo</bold>!</green>"
```
:::

You can also use legacy color codes for backward compatibility:

:::code-group
```toml [server.toml]
welcome_message = "§aWelcome to PicoLimbo!"
```
:::

Welcome message can be disabled by setting an empty string:

:::code-group
```toml [server.toml]
welcome_message = ""
```
:::

## Action Bar <Badge type="warning" text="1.8+" />

Action bar message is displayed to players after joining above the hotbar.
Supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.
Please note that for versions prior to 1.11, the action bar message will be sent using legacy color codes.

:::code-group
```toml [server.toml]
action_bar = "<green>Welcome to <bold>PicoLimbo</bold>!</green>"
```
:::

You can also use legacy color codes for backward compatibility:

:::code-group
```toml [server.toml]
action_bar = "§aWelcome to PicoLimbo!"
```
:::

Action bar message can be disabled by setting an empty string:

:::code-group
```toml [server.toml]
action_bar = ""
```
:::

## Default Gamemode

The default game mode for players.

:::code-group
```toml [server.toml]
default_game_mode = "spectator"
```
:::

Possible values:
```
survival
creative
adventure
spectator
```

> [!NOTE]
> For Minecraft versions 1.7.x, the spectator game mode does not exist. If you set `default_game_mode = "spectator"`, it will spawn players in "creative" mode instead.

## Hardcore

Spawns the player in hardcore mode.

:::code-group
```toml [server.toml]
hardcore = true
```
:::

## Reduced Debug Info <Badge type="warning" text="1.8+" />

Whether the debug screen shows all or reduced information; and whether the effects of F3+B (entity hitboxes) and F3+G (chunk boundaries) are shown.

:::code-group
```toml [server.toml]
reduced_debug_info = true
```
:::

## Player Listed

Whether the player should be listed on the player list.

:::code-group
```toml [server.toml]
player_listed = true
```
:::

## Fetch Player Skins <Badge type="warning" text="1.8+" />

Set to true to fetch the player skin textures from Mojang API.  
If set to false, the server **will still send the skins** if the limbo server is running behind a proxy in online mode.

:::code-group
```toml [server.toml]
fetch_player_skins = true
```
:::

> [!WARNING]
> If you expect a large amount of player to connect to your limbo server instance, your server's IP may get black listed from Mojang API.
