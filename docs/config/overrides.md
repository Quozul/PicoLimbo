# Overrides

PicoLimbo already comes bundled with all the vanilla reports. This feature is useful for either unsupported versions or modded setups.

## Enabling Overrides

To enable overrides, add an `[overrides]` section to your `server.toml` configuration file.

:::code-group
```toml [server.toml] {1}  
[overrides]
blocks_report = ""
```  
:::

If you want to disable overrides and use the default vanilla reports, you can omit the entire `[overrides]` section.

## Blocks Report

Within the overrides section, you can specify the path to your custom blocks report JSON file using the `blocks_report` key.

:::code-group
```toml [server.toml] {2}  
[overrides]
blocks_report = "/path/to/blocks.json"
```  
:::

Replace `/path/to/blocks.json` with the actual path to your custom blocks report JSON file.

### Disabling Blocks Report Override

If you want to disable the blocks report override and use the default blocks report, you can either omit the entire `[overrides]` section or set the `blocks_report` key to an empty string:

:::code-group
```toml [server.toml] {2}  
[overrides]
blocks_report = ""
```  
:::

By setting the value to an empty string, PicoLimbo will not use any custom blocks report and will rely on the default one.
