# Message Formatting

PicoLimbo supports **MiniMessage** formatting for text messages displayed to players. MiniMessage provides a readable tag-based syntax for colors, decorations, fonts, events, and Minecraft text components.

Learn more about MiniMessage on [Adventure's documentation](https://docs.advntr.dev/minimessage/index.html).

> [!NOTE]
> MiniMessage is the recommended formatting method for new configuration. Legacy color codes are still supported for compatibility, but are deprecated and planned for removal in a future version.

## Basic Syntax

MiniMessage uses angle brackets to define formatting tags:

:::code-group
```text
<red>This text is red</red>
<green>This text is green</green>
<green><bold>This text is bold green</bold></green>
```
:::

Tags can be nested. Closing a tag restores the formatting that was active before that tag.

PicoLimbo also accepts quoted arguments, which are useful when an argument contains MiniMessage tags or other special characters:

:::code-group
```text
<hover:show_text:'<red>Hover text</red>'>Hover me</hover>
```
:::
## Colors

PicoLimbo supports all standard Minecraft named colors, hexadecimal RGB colors, verbose MiniMessage color tags, and additional RGB/HSL/HSV/HSB color formats.

### Named Colors

The standard Minecraft color names are supported:

- `<black>`
- `<dark_blue>`
- `<dark_green>`
- `<dark_aqua>`
- `<dark_red>`
- `<dark_purple>`
- `<gold>`
- `<gray>`
- `<dark_gray>`
- `<blue>`
- `<green>`
- `<aqua>`
- `<red>`
- `<light_purple>`
- `<yellow>`
- `<white>`

`<grey>` and `<dark_grey>` are also accepted as aliases for `<gray>` and `<dark_gray>`.

:::code-group
```text
<red>Red</red>
<gold>Gold</gold>
<dark_purple>Dark purple</dark_purple>
```
:::

### Hex Colors

Hex colors use the MiniMessage `<#RRGGBB>` form.

:::code-group
```text
<#ff0088>Pink</#ff0088>
<#8b5cf6>Purple</#8b5cf6>
<#ffffff>White</#ffffff>
```
:::

Hex digits are case-insensitive.

For clients before 1.16, PicoLimbo converts RGB colors to the closest legacy Minecraft color before sending the component.

### Verbose Colors

The standard verbose MiniMessage color forms are supported:

:::code-group
```text
<color:red>Red</color>
<color:#ff0088>Pink</color>
<colour:#8b5cf6>Purple</colour>
<c:#ffffff>White</c>
```
:::

`color`, `colour`, and `c` can use either a named color or a hexadecimal color.

### RGB

RGB functional colors are supported as a PicoLimbo extension.

:::code-group
```text
<rgb:255:0:128>RGB</rgb:255:0:128>
<rgb:100%:0%:50%>RGB percentages</rgb:100%:0%:50%>
```
:::

Each channel can be an integer or a percentage. Values are normalized to an RGB color before serialization.

### HSL

HSL is supported as a PicoLimbo extension.

:::code-group
```text
<hsl:330:100:50>HSL</hsl:330:100:50>
<hsl:330deg:100%:50%>HSL with units</hsl:330deg:100%:50%
```
:::

The arguments are hue, saturation, and lightness.

- Hue defaults to degrees.
- Hue also accepts `deg`, `turn`, `rad`, and `grad`.
- Saturation is 0 to 100 percent.
- Lightness is 0 to 100 percent.

### HSV and HSB

HSV and HSB are supported as PicoLimbo extensions. `hsb` is an alias for `hsv`.

:::code-group
```text
<hsv:330:100:100>HSV</hsv:330:100:100>
<hsb:330:100:100>HSB</hsb:330:100:100>
```
:::

The arguments are hue, saturation, and value.

- Hue defaults to degrees.
- Saturation is 0 to 100 percent.
- Value is 0 to 100 percent.

All RGB/HSL/HSV/HSB values are converted to RGB before serialization. Alpha/transparency is not represented by PicoLimbo's current text component color model.

## Decorations

The following decoration tags are supported:

- `<bold>` / `<b>`
- `<italic>` / `<i>` / `<em>`
- `<underlined>` / `<u>`
- `<strikethrough>` / `<st>`
- `<obfuscated>` / `<obf>`

:::code-group
```text
<bold>Bold</bold>
<italic>Italic</italic>
<underlined>Underlined</underlined>
<strikethrough>Strikethrough</strikethrough>
<obfuscated>Obfuscated</obfuscated>
```
:::

Decorations can be negated with `!`.

## Reset

The `reset` tag clears the active MiniMessage styling.

:::code-group
```text
<red>Red <reset>Normal again
```
:::

## Fonts

Use `font` to set the Minecraft font resource location on a component.

:::code-group
```text
<font:uniform>Uniform</font>
<font:minecraft:default>Default Minecraft font</font>
```
:::

PicoLimbo does not provide the font resource itself. The client must have the corresponding font available, normally through a resource pack.

## Insertion

The `insert` tag sets the insertion value on a component.

:::code-group
```text
<insert:'PicoLimbo'>Select this text</insert>
```
:::

## Click Events

Click events use `<click:action:value>text</click>`.

:::code-group
```text
<click:run_command:/spawn>Go to spawn</click>
<click:copy_to_clipboard:PicoLimbo>Copy this</click>
<click:open_url:'https://example.com'>Open website</click>
```
:::

## Hover Events

Hover events support MiniMessage text, item, and entity hover actions.

:::code-group
```text
<hover:show_text:'<red>Hover text</red>'>Hover me</hover>
```
:::

## Keybind Components

Use `key` to create a Minecraft keybind component.

:::code-group
```text
Press <key:key.jump> to jump.
Press <key:key.inventory> to open your inventory.
```
:::

## Translatable Components

Use `lang`, `tr`, or `translate` to create a translatable component. Use `lang_or`, `tr_or`, or `translate_or` for a translation with a fallback.

:::code-group
```text
<lang:chat.type.text:'<red>Hello</red>'>
<lang_or:some.translation:'Fallback text':'<gold>Argument</gold>'>
```
:::

## Selector Components

Use `selector` or `sel` to create a selector component.

:::code-group
```text
<selector:@a>
<selector:@a:'<gray>, </gray>'>
```
:::

The optional separator is parsed as a component.

## NBT Components

Use `nbt` or `data` for NBT components.

:::code-group
```text
<nbt:block:~ ~ ~:Items>
<nbt:entity:@s:Health>
<nbt:storage:example:data:value>
```
:::

The supported source types are `block`, `entity`, and `storage`.

## Newlines

Use `newline` or `br` to insert a line break.

:::code-group
```text
First line<newline>Second line
First line<br/>Second line
```
:::

## Placeholders and Player Context

PicoLimbo provides a context-aware placeholder system for applications that need dynamic messages.

There are three placeholder forms:

- **Component placeholders** insert an existing `Component` without reparsing it.
- **Parsed placeholders** contain MiniMessage text and are parsed when substituted.
- **Unparsed placeholders** insert literal text without interpreting MiniMessage tags.

Player context also provides the built-in `<player>`, `<name>`, and `<uuid>` placeholders when player information is supplied.

> [!NOTE]
> These placeholders are PicoLimbo context features. They are not standard Adventure MiniMessage tags. Applications can provide their own placeholder values.

## Legacy Color Codes

Legacy formatting remains supported for compatibility with existing configurations.

:::code-group
```text
§aGreen text
§cRed text
&aGreen text
&cRed text
```
:::

> [!WARNING]
> Legacy formatting is deprecated and is planned for removal in a future version. New configurations should use MiniMessage instead.

## Examples

:::code-group
```toml [server.toml]
welcome_message = "<green>Welcome to <bold>PicoLimbo</bold>!</green>"
```
:::

:::code-group
```toml [server.toml]
motd = "<#8b5cf6>PicoLimbo <white>limbo server</white></#8b5cf6>"
```
:::
