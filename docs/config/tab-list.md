# Tab List Settings <Badge type="warning" text="1.8+" />

Representing the `[tab_list]` section in `server.toml`.

Both the header and the footer supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.

## Enabled

When setting `enabled` to `true` you must ensure both the header and footer are defined.

:::code-group
```toml [server.toml] {2}
[tab_list]
enabled = true
header = "<bold>Welcome to PicoLimbo</bold>"
footer = "<green>Enjoy your stay!</green>"
```
:::

You can disable the tab list feature completely by setting `enabled` to `false`. In this case, you don't have to define the header and the footer.

:::code-group
```toml [server.toml] {2}
[tab_list]
enabled = false
```
:::

## Header

The header text displayed at the top of the player list.

:::code-group
```toml [server.toml] {3}
[tab_list]
enabled = true
header = "<bold>Welcome to PicoLimbo</bold>"
```
:::

The header can be disabled by setting an empty string:

:::code-group
```toml [server.toml] {3}
[tab_list]
enabled = true
header = ""
```
:::

## Footer

The footer text displayed at the bottom of the player list.

:::code-group
```toml [server.toml] {3}
[tab_list]
enabled = true
footer = "<green>Enjoy your stay!</green>"
```
:::

The footer can be disabled by setting an empty string:

:::code-group
```toml [server.toml] {3}
[tab_list]
enabled = true
footer = ""
```
:::
