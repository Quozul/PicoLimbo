# Server List

Configure how your server appears in Minecraft's server list with these settings.

## Max Players

Maximum player count shown in server lists.
This setting controls how many players your server claims to support in the server list. This is purely cosmetic and doesn't affect the actual player limit.

:::code-group
```toml [server.toml] {2}
[server_list]
max_players = 20
```
:::

## Message of the Day

Message of the Day displayed in server lists.
The `message_of_the_day` appears in the server list and supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.

:::code-group
```toml [server.toml] {2}
[server_list]
message_of_the_day = "<gold>A <bold>PicoLimbo</bold> Server</gold>"
```
:::

You can also use legacy color codes for backward compatibility:

:::code-group
```toml [server.toml] {2}
[server_list]
message_of_the_day = "§6A Minecraft Server"
```
:::

## Online Player Count

Show actual online player count in server lists.
When `show_online_player_count` is set to `true`, the server will display the actual number of currently connected players in the server list. If set to `false`, it will always show 0.

:::code-group
```toml [server.toml] {2}
[server_list]
show_online_player_count = true
```
:::
