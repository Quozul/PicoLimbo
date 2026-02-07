use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize, Clone)]
pub enum SignColor {
    #[default]
    #[serde(rename = "black")]
    Black,
    #[serde(rename = "white")]
    White,
    #[serde(rename = "orange")]
    Orange,
    #[serde(rename = "magenta")]
    Magenta,
    #[serde(rename = "light_blue")]
    LightBlue,
    #[serde(rename = "yellow")]
    Yellow,
    #[serde(rename = "lime")]
    Lime,
    #[serde(rename = "pink")]
    Pink,
    #[serde(rename = "gray")]
    Gray,
    #[serde(rename = "light_gray")]
    LightGray,
    #[serde(rename = "cyan")]
    Cyan,
    #[serde(rename = "purple")]
    Purple,
    #[serde(rename = "blue")]
    Blue,
    #[serde(rename = "brown")]
    Brown,
    #[serde(rename = "green")]
    Green,
    #[serde(rename = "red")]
    Red,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct SignFace {
    has_glowing_text: bool,
    color: SignColor,
    messages: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub enum SignBlockEntity {
    Legacy {
        #[serde(alias = "GlowingText")]
        glowing_text: bool,
        #[serde(alias = "Color")]
        color: SignColor,
        #[serde(alias = "Text1")]
        text_1: String,
        #[serde(alias = "Text2")]
        text_2: String,
        #[serde(alias = "Text3")]
        text_3: String,
        #[serde(alias = "Text4")]
        text_4: String,
    },
    Modern {
        is_waxed: bool,
        front_text: SignFace,
        back_text: SignFace,
    },
}
