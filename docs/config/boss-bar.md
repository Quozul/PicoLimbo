# Boss Bar <Badge type="warning" text="1.9+" />

Representing the `[boss_bar]` section in `server.toml`.

## Enabled

When setting `enabled` to `true` you must ensure all the properties are defined as well.

:::code-group
```toml [server.toml] {2}
[boss_bar]
enabled = true
color = "blue"
division = 0
health = 1.0
title = "<blue><bold>Welcome to PicoLimbo!</bold></blue>"
```
:::

You can disable the tab list feature completely by setting `enabled` to `false`. In this case, you don't have to define the properties.

:::code-group
```toml [server.toml] {2}
[boss_bar]
enabled = false
```
:::

## Color

The color of the boss bar.

:::code-group
```toml [server.toml] {3}
[boss_bar]
enabled = true
color = "blue"
```
:::

Possible values:
```
blue
green
pink
purple
red
white
yellow
```

## Division

The number of divisions in the boss bar, affecting its visual segmentation.

:::code-group
```toml [server.toml] {3}
[boss_bar]
enabled = true
division = 0
```
:::

Possible values:
```
0   - No divisions
6   - 6 segments
10  - 10 segments
12  - 12 segments
20  - 20 segments
```

## Health

The health of the boss bar, represented as a float between `0.0` (empty) and `1.0` (full).

:::code-group
```toml [server.toml] {3}
[boss_bar]
enabled = true
health = 1.0
```
:::

## Title

The title text displayed at the top of the player list.
The title supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.

:::code-group
```toml [server.toml] {3}
[boss_bar]
enabled = true
title = "<blue><bold>Welcome to PicoLimbo!</bold></blue>"
```
:::
