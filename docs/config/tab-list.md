# Tab List Settings

Representing the `[tab_list]` section in `server.toml`.

Both the header and the footer supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.

## Header

The header text displayed at the top of the player list.

:::code-group
```toml [server.toml] {2}
[tab_list]
header = "<bold>Welcome to PicoLimbo</bold>"
```
:::

The header can be disabled by setting an empty string:

:::code-group
```toml [server.toml] {2}
[tab_list]
header = ""
```
:::

## Footer

The footer text displayed at the bottom of the player list.

:::code-group
```toml [server.toml] {2}
[tab_list]
footer = "<green>Enjoy your stay!</green>"
```
:::

The footer can be disabled by setting an empty string:

:::code-group
```toml [server.toml] {2}
[tab_list]
footer = ""
```
:::
