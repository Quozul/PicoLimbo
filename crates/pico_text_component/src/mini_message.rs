use crate::prelude::Component;
use quick_xml::Reader;
use quick_xml::events::Event;
use thiserror::Error;

#[derive(Default, Clone)]
struct Style {
    color: Option<String>,
    bold: bool,
    italic: bool,
    underlined: bool,
    strikethrough: bool,
    obfuscated: bool,
}

#[derive(Debug, Error)]
pub enum MiniMessageError {
    #[error(transparent)]
    QuickXml(#[from] quick_xml::Error),
    #[error(transparent)]
    Encoding(#[from] quick_xml::encoding::EncodingError),
}

pub fn parse_mini_message(input: &str) -> Result<Component, MiniMessageError> {
    let wrapped_input = format!("<root>{}</root>", input);
    let mut reader = Reader::from_str(&wrapped_input);

    let mut flat_components = Vec::new();
    let mut style_stack: Vec<Style> = vec![Style::default()];

    loop {
        match reader.read_event()? {
            Event::Start(e) => {
                let mut new_style = style_stack.last().cloned().unwrap_or_default();
                let tag_name = String::from_utf8(e.name().as_ref().to_vec())
                    .unwrap_or_else(|_| "".to_string());

                match tag_name.as_str() {
                    "black" | "dark_blue" | "dark_green" | "dark_aqua" | "dark_red"
                    | "dark_purple" | "gold" | "gray" | "dark_gray" | "blue" | "green" | "aqua"
                    | "red" | "light_purple" | "yellow" | "white" => {
                        new_style.color = Some(tag_name);
                    }
                    "bold" | "b" => new_style.bold = true,
                    "italic" | "i" | "em" => new_style.italic = true,
                    "underlined" | "u" => new_style.underlined = true,
                    "strikethrough" | "st" => new_style.strikethrough = true,
                    "obfuscated" | "obf" => new_style.obfuscated = true,
                    "root" => {}
                    _ => {}
                }
                style_stack.push(new_style);
            }
            Event::End(_) => {
                if style_stack.len() > 1 {
                    style_stack.pop();
                }
            }
            Event::Text(e) => {
                let text = e.decode()?.to_string();
                if text.is_empty() {
                    continue;
                }

                let current_style = style_stack.last().unwrap();

                flat_components.push(Component {
                    text: text.to_string(),
                    color: current_style.color.clone(),
                    bold: current_style.bold,
                    italic: current_style.italic,
                    underlined: current_style.underlined,
                    strikethrough: current_style.strikethrough,
                    obfuscated: current_style.obfuscated,
                    extra: vec![],
                });
            }
            Event::Eof => break,
            _ => (),
        }
    }

    if flat_components.is_empty() {
        Ok(Component::default())
    } else {
        Ok(Component {
            extra: flat_components,
            ..Component::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_from_prompt_nested() {
        let input = "<red><bold>Hello,</bold></red> <blue>world!</blue>";
        let result = parse_mini_message(input).unwrap();

        let expected = Component {
            extra: vec![
                Component {
                    text: "Hello,".to_string(),
                    color: Some("red".to_string()),
                    bold: true,
                    ..Component::default()
                },
                Component {
                    text: " ".to_string(),
                    ..Component::default()
                },
                Component {
                    text: "world!".to_string(),
                    color: Some("blue".to_string()),
                    ..Component::default()
                },
            ],
            ..Component::default()
        };

        assert_eq!(result, expected);
    }

    #[test]
    fn test_json_serialization_nested() {
        let input = "<red><bold>Hello,</bold></red> <blue>world!</blue>";
        let component = parse_mini_message(input).unwrap();
        let json_output = serde_json::to_string(&component).unwrap();

        // Note: The order of fields in JSON is not guaranteed, but serde usually serializes in order.
        // A more robust test would parse the JSON back and compare, but for this case, string comparison is fine.
        let expected_json = r#"{"text":"","extra":[{"text":"Hello,","color":"red","bold":true},{"text":" "},{"text":"world!","color":"blue"}]}"#;

        assert_eq!(json_output, expected_json);
    }

    #[test]
    fn test_plain_text_nested() {
        let input = "Just some plain text.";
        let result = parse_mini_message(input).unwrap();
        let expected = Component {
            extra: vec![Component {
                text: "Just some plain text.".to_string(),
                ..Component::default()
            }],
            ..Component::default()
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_nested_tags_nested() {
        let input = "<red>This is red, <bold>and this is bold red.</bold> Back to red.</red>";
        let result = parse_mini_message(input).unwrap();
        let expected = Component {
            text: String::new(),
            extra: vec![
                Component {
                    text: "This is red, ".to_string(),
                    color: Some("red".to_string()),
                    ..Component::default()
                },
                Component {
                    text: "and this is bold red.".to_string(),
                    color: Some("red".to_string()),
                    bold: true,
                    ..Component::default()
                },
                Component {
                    text: " Back to red.".to_string(),
                    color: Some("red".to_string()),
                    ..Component::default()
                },
            ],
            ..Component::default()
        };
        assert_eq!(result, expected);
    }

    #[test]
    fn test_empty_input_nested() {
        let input = "";
        let result = parse_mini_message(input).unwrap();
        let expected = Component::default();
        assert_eq!(result, expected);
    }
}
