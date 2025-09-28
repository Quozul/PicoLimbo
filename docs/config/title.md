# Title <Badge type="warning" text="1.8+" />

Represents the `[title]` section in `server.toml` for controlling title/subtitle effects.

## Enabled

When setting `enabled` to `true` you must define all title properties.

:::code-group
```toml [server.toml] {2}
[title]
enabled = true
title = "<bold>Welcome!</bold>"
sub_title = "Enjoy your stay"
fade_in = 10
stay = 70
fade_out = 20
```
:::

You can disable the title feature completely by setting `enabled` to `false`. In this case, you don't need to define any other properties.

:::code-group
```toml [server.toml] {2}
[title]
enabled = false
```
:::

## Title

The main title text displayed to players. Supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.

:::code-group
```toml [server.toml] {3}
[title]
enabled = true
title = "<green><italic>Welcome to PicoLimbo!</italic></green>"
```
:::

## Sub Title

The subtitle text displayed below the main title. Supports [MiniMessage formatting](/customization/message-formatting.html).

:::code-group
```toml [server.toml] {3}
[title]
enabled = true
sub_title = "Enjoy your stay"
```
:::

## Fade In

The number of ticks (1 second = 20 ticks) to fade in the title effect.  
Default: `10` (0.5 seconds)

:::code-group
```toml [server.toml] {3}
[title]
enabled = true
fade_in = 10
```
:::

## Stay

The number of ticks the title stays visible at full opacity.  
Default: `70` (3.5 seconds)

:::code-group
```toml [server.toml] {3}
[title]
enabled = true
stay = 70
```
:::

## Fade Out

The number of ticks to fade out the title effect.  
Default: `20` (1 second)

:::code-group
```toml [server.toml] {3}
[title]
enabled = true
fade_out = 20
```
:::
