# Compression <Badge type="warning" text="1.8+" />

Representing the `[compression]` section in `server.toml`.

## Threshold

How big should a packet be to be compressed.

:::code-group
```toml [server.toml] {2}
[compression]
threshold = 256
```
:::

Compress all the packets by setting a value of `0`.

:::code-group
```toml [server.toml] {2}
[compression]
threshold = 0
```
:::

You can disable the compression by setting a negative value.

:::code-group
```toml [server.toml] {2}
[compression]
threshold = -1
```
:::

## Level

The integer is on a scale of 0-9 where 0 means "no compression" and 9 means "take as long as you'd like".
This setting is only used when threshold is a positive integer.
Default and recommended value is 6.

:::code-group
```toml [server.toml] {2}
[compression]
level = 6
```
:::
