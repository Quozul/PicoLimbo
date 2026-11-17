mod component;
mod mini_message;

pub mod prelude {
    pub use crate::component::{ClickEvent, Component, HoverEvent};
    pub use crate::mini_message::{MiniMessageError, parse_mini_message};
}
