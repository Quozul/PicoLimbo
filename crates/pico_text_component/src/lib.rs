mod component;
mod mini_message;

pub mod prelude {
    pub use crate::component::Component;
    pub use crate::mini_message::{MiniMessageError, parse_mini_message};
}
