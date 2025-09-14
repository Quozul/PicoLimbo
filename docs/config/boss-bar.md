# Boss Bar Settings

Representing the `[boss_bar]` section in `server.toml`.

> ![NOTE]
> The boss bar will only show to clients after 1.9 (included).

## Boss Bar Title

The title text displayed at the top of the player list.
The title supports [MiniMessage formatting](/customization/message-formatting.html) for colors and styling.

:::code-group
```toml [server.toml] {2}
[boss_bar]
title = "<blue><bold>Welcome to PicoLimbo!</bold></blue>"
```
:::

## Boss Bar Health

The health of the boss bar, represented as a float between `0.0` (empty) and `1.0` (full).

:::code-group
```toml [server.toml] {2}
[boss_bar]
health = 1.0
```
:::

## Boss Bar Color

The color of the boss bar.

:::code-group
```toml [server.toml] {2}
[boss_bar]
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

## Boss Bar Divisions

The number of divisions in the boss bar, affecting its visual segmentation.

:::code-group
```toml [server.toml] {2}
[boss_bar]
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
