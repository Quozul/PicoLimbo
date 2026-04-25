# Pick Item From Block <Badge type="warning" text="1.16+" />

Representing the `[pick_item]` section in `server.toml`.

This feature controls the behavior of middle-click block picking.

## Enable / Disable

Enable or disable pick item from block globally.

:::code-group

```toml [server.toml] {2}
[pick_item]
enabled = true
stack_size = 1
```

:::

If `enabled = false`, PicoLimbo ignores this action.

## Stack Size

Number of items given when a block is picked.

:::code-group

```toml [server.toml] {3}
[pick_item]
enabled = true
stack_size = 2
```

:::

Default value:

```toml
[pick_item]
enabled = false
stack_size = 1
```

## Notes

- Item resolution uses PicoLimbo's existing generated registries for the player's protocol version.
- Some special blocks are mapped to equivalent items (for example wall variants to their regular item forms).
- If no matching item is found for a block, no item is given.
