use crate::prelude::{ClickEvent, Component, HoverEvent};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, Default)]
pub struct PlayerContext {
    pub name: Option<String>,
    pub uuid: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct MiniMessageContext {
    pub player: Option<PlayerContext>,
    pub placeholders: HashMap<String, Component>,
    pub parsed_placeholders: HashMap<String, String>,
    pub unparsed_placeholders: HashMap<String, String>,
}

#[derive(Clone, Default)]
struct Style {
    tag: String,
    color: Option<String>,
    font: Option<String>,
    insertion: Option<String>,
    click_event: Option<ClickEvent>,
    hover_event: Option<HoverEvent>,
    bold: bool,
    italic: bool,
    underlined: bool,
    strikethrough: bool,
    obfuscated: bool,
}

#[derive(Debug, Error)]
pub enum MiniMessageError {
    #[error("Invalid MiniMessage tag: {tag}")]
    InvalidTag { tag: String },
}

fn split_arguments(input: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;

    for ch in input.chars() {
        if escaped {
            current.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }

        if let Some(q) = quote {
            if ch == q {
                quote = None;
            } else {
                current.push(ch);
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch == ':' {
            values.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(ch);
        }
    }

    values.push(current.trim().to_string());
    values
}

fn find_tag_end(input: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;

    for (offset, ch) in input[start..].char_indices() {
        if offset == 0 {
            continue;
        }

        if escaped {
            escaped = false;
            continue;
        }

        if ch == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }

        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
        } else if ch == '\'' || ch == '"' {
            quote = Some(ch);
        } else if ch == '>' {
            return Some(start + offset);
        }
    }

    None
}

fn parse_tag(raw: &str) -> (bool, bool, Vec<String>) {
    let mut value = raw.trim();
    let closing = value.starts_with('/');
    if closing {
        value = &value[1..];
    }

    let self_closing = value.ends_with('/');
    if self_closing {
        value = value[..value.len() - 1].trim_end();
    }

    (closing, self_closing, split_arguments(value))
}

fn color_from_tag(tag: &str) -> Option<String> {
    let color = tag
        .strip_prefix("color:")
        .or_else(|| tag.strip_prefix("colour:"))
        .or_else(|| tag.strip_prefix("c:"))
        .unwrap_or(tag);

    match color {
        "black" | "dark_blue" | "dark_green" | "dark_aqua" | "dark_red" | "dark_purple"
        | "gold" | "gray" | "dark_gray" | "blue" | "green" | "aqua" | "red" | "light_purple"
        | "yellow" | "white" | "grey" | "dark_grey" => Some(
            match color {
                "grey" => "gray",
                "dark_grey" => "dark_gray",
                _ => color,
            }
            .to_string(),
        ),
        _ => crate::component::normalize_color(color),
    }
}

fn decoration(tag: &str, value: Option<&str>) -> Option<(&'static str, bool)> {
    let enabled = if tag.starts_with('!') {
        false
    } else {
        value != Some("false")
    };

    match tag.strip_prefix('!').unwrap_or(tag) {
        "bold" | "b" => Some(("bold", enabled)),
        "italic" | "i" | "em" => Some(("italic", enabled)),
        "underlined" | "u" => Some(("underlined", enabled)),
        "strikethrough" | "st" => Some(("strikethrough", enabled)),
        "obfuscated" | "obf" => Some(("obfuscated", enabled)),
        _ => None,
    }
}

fn parse_hover(action: &str, args: &[String], context: &MiniMessageContext) -> Option<HoverEvent> {
    match action {
        "show_text" => {
            let value = args.join(":");
            let component = parse_mini_message_with_context(&value, context).ok()?;
            Some(HoverEvent {
                action: action.to_string(),
                contents: serde_json::to_value(component).ok()?,
            })
        }
        "show_item" => {
            let id = args.first()?.clone();
            let mut contents = serde_json::Map::new();
            contents.insert("id".to_string(), serde_json::Value::String(id));

            if let Some(count) = args.get(1).and_then(|value| value.parse::<i32>().ok())
                && count > 0
            {
                contents.insert("count".to_string(), serde_json::Value::from(count));
            }

            Some(HoverEvent {
                action: action.to_string(),
                contents: serde_json::Value::Object(contents),
            })
        }
        "show_entity" => {
            let entity_type = args.first()?.clone();
            let uuid = args.get(1)?.clone();
            let mut contents = serde_json::Map::new();
            contents.insert("type".to_string(), serde_json::Value::String(entity_type));
            contents.insert("id".to_string(), serde_json::Value::String(uuid));

            if let Some(name) = args.get(2) {
                let name = parse_mini_message_with_context(name, context).ok()?;
                contents.insert("name".to_string(), serde_json::to_value(name).ok()?);
            }

            Some(HoverEvent {
                action: action.to_string(),
                contents: serde_json::Value::Object(contents),
            })
        }
        _ => None,
    }
}

fn styled_text(text: String, style: &Style) -> Component {
    Component {
        text,
        color: style.color.clone(),
        bold: style.bold,
        italic: style.italic,
        underlined: style.underlined,
        strikethrough: style.strikethrough,
        obfuscated: style.obfuscated,
        font: style.font.clone(),
        insertion: style.insertion.clone(),
        click_event: style.click_event.clone(),
        hover_event: style.hover_event.clone(),
        ..Component::default()
    }
}

fn placeholder_component(name: &str, context: &MiniMessageContext) -> Option<Component> {
    if let Some(component) = context.placeholders.get(name) {
        return Some(component.clone());
    }

    if let Some(value) = context.unparsed_placeholders.get(name) {
        return Some(Component::new(value));
    }

    if let Some(value) = context.parsed_placeholders.get(name) {
        return parse_mini_message_with_context(value, context).ok();
    }

    match name {
        "player" | "name" => context
            .player
            .as_ref()
            .and_then(|player| player.name.clone())
            .map(Component::new),
        "uuid" => context
            .player
            .as_ref()
            .and_then(|player| player.uuid.clone())
            .map(Component::new),
        _ => None,
    }
}

fn parse_inline_component(value: &str, context: &MiniMessageContext) -> Option<Component> {
    parse_mini_message_with_context(value, context).ok()
}

fn append_component(output: &mut Vec<Component>, component: Component, style: &Style) {
    if component == Component::default() {
        return;
    }

    let mut component = component;
    if component.color.is_none() {
        component.color = style.color.clone();
    }
    if component.font.is_none() {
        component.font = style.font.clone();
    }
    if component.insertion.is_none() {
        component.insertion = style.insertion.clone();
    }
    if component.click_event.is_none() {
        component.click_event = style.click_event.clone();
    }
    if component.hover_event.is_none() {
        component.hover_event = style.hover_event.clone();
    }

    output.push(component);
}

fn handle_insert_tag(
    args: &[String],
    style: &Style,
    context: &MiniMessageContext,
    output: &mut Vec<Component>,
) -> bool {
    let Some(name) = args.first() else {
        return false;
    };

    if let Some(component) = placeholder_component(name, context) {
        append_component(output, component, style);
        return true;
    }

    match name.as_str() {
        "newline" | "br" => {
            output.push(styled_text("\n".to_string(), style));
            true
        }
        "key" if args.len() >= 2 => {
            output.push(Component {
                keybind: Some(args[1].clone()),
                color: style.color.clone(),
                font: style.font.clone(),
                insertion: style.insertion.clone(),
                click_event: style.click_event.clone(),
                hover_event: style.hover_event.clone(),
                ..Component::default()
            });
            true
        }
        "lang" | "tr" | "translate" if args.len() >= 2 => {
            let with = args[2..]
                .iter()
                .filter_map(|value| parse_inline_component(value, context))
                .collect();

            output.push(Component {
                translate: Some(args[1].clone()),
                with,
                color: style.color.clone(),
                font: style.font.clone(),
                insertion: style.insertion.clone(),
                ..Component::default()
            });
            true
        }
        "lang_or" | "tr_or" | "translate_or" if args.len() >= 3 => {
            let with = args[3..]
                .iter()
                .filter_map(|value| parse_inline_component(value, context))
                .collect();

            output.push(Component {
                translate: Some(args[1].clone()),
                fallback: Some(args[2].clone()),
                with,
                color: style.color.clone(),
                font: style.font.clone(),
                insertion: style.insertion.clone(),
                ..Component::default()
            });
            true
        }
        "selector" | "sel" if args.len() >= 2 => {
            let separator = args
                .get(2)
                .and_then(|value| parse_inline_component(value, context))
                .map(Box::new);

            output.push(Component {
                selector: Some(args[1].clone()),
                separator,
                color: style.color.clone(),
                font: style.font.clone(),
                insertion: style.insertion.clone(),
                ..Component::default()
            });
            true
        }
        "nbt" | "data" if args.len() >= 4 => {
            let source = args[1].as_str();
            let id = args[2].clone();
            let path = args[3].clone();
            let mut component = Component {
                nbt: Some(path),
                color: style.color.clone(),
                font: style.font.clone(),
                insertion: style.insertion.clone(),
                ..Component::default()
            };

            match source {
                "block" => component.block = Some(id),
                "entity" => component.entity = Some(id),
                "storage" => component.storage = Some(id),
                _ => return false,
            }

            component.interpret = args.iter().any(|value| value == "interpret");
            if let Some(separator) = args.get(4).filter(|value| value.as_str() != "interpret") {
                component.separator = parse_inline_component(separator, context).map(Box::new);
            }

            output.push(component);
            true
        }
        _ => false,
    }
}

fn apply_tag(
    tag: &str,
    args: &[String],
    style_stack: &mut Vec<Style>,
    context: &MiniMessageContext,
    output: &mut Vec<Component>,
) -> bool {
    let normalized = tag.to_ascii_lowercase();

    if normalized == "reset" {
        style_stack.clear();
        style_stack.push(Style::default());
        return true;
    }

    let color_tag = if args.len() > 1 {
        std::iter::once(normalized.as_str())
            .chain(args[1..].iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(":")
    } else {
        normalized.clone()
    };

    if let Some(color) = color_from_tag(&color_tag) {
        let mut style = style_stack.last().cloned().unwrap_or_default();
        style.tag = normalized.clone();
        style.color = Some(color);
        style_stack.push(style);
        return true;
    }

    if (normalized == "hsl" || normalized == "rgb" || normalized == "hsv" || normalized == "hsb")
        && let Some(color) =
            crate::component::normalize_color(&format!("{}({})", normalized, args[1..].join(", ")))
    {
        let mut style = style_stack.last().cloned().unwrap_or_default();
        style.tag = normalized.clone();
        style.color = Some(color);
        style_stack.push(style);
        return true;
    }

    if let Some((name, enabled)) = decoration(&normalized, args.get(1).map(String::as_str)) {
        let mut style = style_stack.last().cloned().unwrap_or_default();
        style.tag = normalized.clone();
        match name {
            "bold" => style.bold = enabled,
            "italic" => style.italic = enabled,
            "underlined" => style.underlined = enabled,
            "strikethrough" => style.strikethrough = enabled,
            "obfuscated" => style.obfuscated = enabled,
            _ => {}
        }
        style_stack.push(style);
        return true;
    }

    if normalized == "font" && args.len() >= 2 {
        let mut style = style_stack.last().cloned().unwrap_or_default();
        style.tag = normalized.clone();
        style.font = Some(args[1..].join(":"));
        style_stack.push(style);
        return true;
    }

    if normalized == "insert" && args.len() >= 2 {
        let mut style = style_stack.last().cloned().unwrap_or_default();
        style.tag = normalized.clone();
        style.insertion = Some(args[1].clone());
        style_stack.push(style);
        return true;
    }

    if normalized == "click" && args.len() >= 3 {
        let mut style = style_stack.last().cloned().unwrap_or_default();
        style.tag = normalized.clone();
        style.click_event = Some(ClickEvent {
            action: args[1].clone(),
            value: args[2].clone(),
        });
        style_stack.push(style);
        return true;
    }

    if normalized == "hover" && args.len() >= 3 {
        let mut style = style_stack.last().cloned().unwrap_or_default();
        style.tag = normalized.clone();
        style.hover_event = parse_hover(&args[1], &args[2..], context);
        if style.hover_event.is_some() {
            style_stack.push(style);
            return true;
        }
    }

    if handle_insert_tag(
        args,
        &style_stack.last().cloned().unwrap_or_default(),
        context,
        output,
    ) {
        return true;
    }

    false
}

pub fn parse_mini_message(input: &str) -> Result<Component, MiniMessageError> {
    parse_mini_message_with_context(input, &MiniMessageContext::default())
}

pub fn parse_mini_message_with_context(
    input: &str,
    context: &MiniMessageContext,
) -> Result<Component, MiniMessageError> {
    let mut output = Vec::new();
    let mut style_stack = vec![Style::default()];
    let mut cursor = 0;

    while cursor < input.len() {
        let Some(relative_start) = input[cursor..].find('<') else {
            let text = input[cursor..].replace("\\<", "<");
            if !text.is_empty() {
                output.push(styled_text(text, style_stack.last().unwrap()));
            }
            break;
        };

        let start = cursor + relative_start;
        if start > cursor {
            let text = input[cursor..start].replace("\\<", "<");
            if !text.is_empty() {
                output.push(styled_text(text, style_stack.last().unwrap()));
            }
        }

        if start > 0 && input.as_bytes()[start - 1] == b'\\' {
            output.push(styled_text(
                "<".to_string(),
                &style_stack.last().cloned().unwrap_or_default(),
            ));
            cursor = start + 1;
            continue;
        }

        let Some(end) = find_tag_end(input, start) else {
            output.push(styled_text(
                input[start..].to_string(),
                &style_stack.last().cloned().unwrap_or_default(),
            ));
            break;
        };

        let raw = &input[start + 1..end];
        let (closing, self_closing, args) = parse_tag(raw);

        if args.is_empty() {
            cursor = end + 1;
            continue;
        }

        let tag = args[0].to_ascii_lowercase();

        if closing {
            if let Some(position) = style_stack.iter().rposition(|style| style.tag == tag) {
                style_stack.truncate(position);
                if style_stack.is_empty() {
                    style_stack.push(Style::default());
                }
            } else if !raw.starts_with('/') {
                output.push(styled_text(
                    input[start..=end].to_string(),
                    style_stack.last().unwrap(),
                ));
            }
        } else if !apply_tag(&tag, &args, &mut style_stack, context, &mut output) {
            if let Some(component) = placeholder_component(&tag, context) {
                append_component(
                    &mut output,
                    component,
                    &style_stack.last().cloned().unwrap_or_default(),
                );
            } else {
                output.push(styled_text(
                    input[start..=end].to_string(),
                    style_stack.last().unwrap(),
                ));
            }
        }

        if self_closing && style_stack.len() > 1 {
            style_stack.pop();
        }

        cursor = end + 1;
    }

    Ok(Component {
        extra: output,
        ..Component::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_style_stack() {
        let result = parse_mini_message(
            "<red><bold><font:uniform><insert:hello>Hello</insert></font></bold></red>",
        )
        .unwrap();

        let text = &result.extra[0];
        assert_eq!(text.color, Some("red".to_string()));
        assert!(text.bold);
        assert_eq!(text.font, Some("uniform".to_string()));
        assert_eq!(text.insertion, Some("hello".to_string()));
    }

    #[test]
    fn test_click_event() {
        let result = parse_mini_message("<click:run_command:/spawn>Spawn</click>").unwrap();
        assert_eq!(
            result.extra[0].click_event,
            Some(ClickEvent {
                action: "run_command".to_string(),
                value: "/spawn".to_string(),
            })
        );
    }

    #[test]
    fn test_hover_event() {
        let result = parse_mini_message("<hover:show_text:'<red>hello'>Hover</hover>").unwrap();
        assert!(result.extra[0].hover_event.is_some());
    }

    #[test]
    fn test_hover_item() {
        let result = parse_mini_message("<hover:show_item:diamond:2>Item</hover>").unwrap();
        let hover = result.extra[0].hover_event.as_ref().unwrap();
        assert_eq!(hover.action, "show_item");
        assert_eq!(
            hover.contents["id"],
            serde_json::Value::String("diamond".to_string())
        );
        assert_eq!(hover.contents["count"], serde_json::Value::from(2));
    }

    #[test]
    fn test_hover_entity() {
        let result = parse_mini_message(
            "<hover:show_entity:pig:00000000-0000-0000-0000-000000000000:'Pig'>Entity</hover>",
        )
        .unwrap();
        let hover = result.extra[0].hover_event.as_ref().unwrap();
        assert_eq!(hover.action, "show_entity");
        assert_eq!(
            hover.contents["type"],
            serde_json::Value::String("pig".to_string())
        );
        assert_eq!(
            hover.contents["id"],
            serde_json::Value::String("00000000-0000-0000-0000-000000000000".to_string())
        );
        assert_eq!(
            hover.contents["name"]["extra"][0]["text"],
            serde_json::Value::String("Pig".to_string())
        );
    }

    #[test]
    fn test_keybind() {
        let result = parse_mini_message("Press <key:key.jump>!").unwrap();
        assert_eq!(result.extra[1].keybind, Some("key.jump".to_string()));
    }

    #[test]
    fn test_translate() {
        let result = parse_mini_message("<lang:chat.type.text:'<red>Hello'>").unwrap();
        assert_eq!(
            result.extra[0].translate,
            Some("chat.type.text".to_string())
        );
        assert_eq!(
            result.extra[0].with[0].extra[0].color,
            Some("red".to_string())
        );
    }

    #[test]
    fn test_player_placeholder() {
        let context = MiniMessageContext {
            player: Some(PlayerContext {
                name: Some("THEMPGUY".to_string()),
                uuid: Some("00000000-0000-0000-0000-000000000000".to_string()),
            }),
            ..MiniMessageContext::default()
        };

        let result = parse_mini_message_with_context("Hello <player>!", &context).unwrap();
        assert_eq!(result.extra[1].text, "THEMPGUY");
    }

    #[test]
    fn test_decoration_can_be_disabled_with_false() {
        let result = parse_mini_message("<red><bold:false>Normal</bold></red>").unwrap();
        assert!(!result.extra[0].bold);
        assert_eq!(result.extra[0].color, Some("red".to_string()));
    }

    #[test]
    fn test_decoration_can_be_negated() {
        let result = parse_mini_message("<bold><!bold>Normal</bold></bold>").unwrap();
        assert!(!result.extra[0].bold);
    }

    #[test]
    fn test_namespaced_font() {
        let result = parse_mini_message("<font:minecraft:default>Hello</font>").unwrap();
        assert_eq!(result.extra[0].font, Some("minecraft:default".to_string()));
    }

    #[test]
    fn test_component_placeholder() {
        let mut context = MiniMessageContext::default();
        context.placeholders.insert(
            "server".to_string(),
            Component {
                text: "RealmsNetwork".to_string(),
                color: Some("#ff00ff".to_string()),
                ..Component::default()
            },
        );

        let result = parse_mini_message_with_context("Hello <server>!", &context).unwrap();
        assert_eq!(result.extra[1].text, "RealmsNetwork");
    }
}
