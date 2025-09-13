# Message Formatting

PicoLimbo supports **MiniMessage** formatting for styling text messages displayed to players. MiniMessage provides a modern, readable syntax for text formatting that's more intuitive than legacy color codes.

## What is MiniMessage?

MiniMessage is a text formatting system that uses XML-like tags to apply colors and formatting to text. Instead of cryptic codes like `§a` or `&c`, you can use descriptive tags like `<green>` or `<red>`.
Learn more about MiniMessage on [Adventure's documentation](https://docs.advntr.dev/minimessage/index.html).

> [!NOTE]
> MiniMessage is the recommended formatting method. Legacy color codes (like `§a` or `&c`) are still supported but may be deprecated in future versions.

## Basic Syntax

MiniMessage uses angle brackets `<>` to define formatting tags:

:::code-group
```xml
<red>This text is red</red>
<green>This text is green</green>
<bold>This text is bold</bold>
```
:::

## Supported Features

PicoLimbo currently supports a **subset** of MiniMessage features:

### ✅ Supported
- **Colors** - All standard Minecraft colors
- **Formatting** - `<bold>`, `<italic>`, `<underlined>`, `<strikethrough>` and `<obfuscated>`
- **New lines** - `<newline>`

### ❌ Not Yet Supported
- Gradients and custom colors
- Hover events
- Click events
- Custom fonts
- Keybinds
- Translatable components

## Examples

:::code-group
```toml [server.toml]
welcome_message = "<green>Welcome to <bold>PicoLimbo</bold>!</green>"
```
:::
