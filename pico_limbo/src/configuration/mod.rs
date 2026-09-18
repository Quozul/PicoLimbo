pub mod boss_bar;
pub mod commands;
mod compression;
pub mod config;
mod connection_config;
mod env_placeholders;
pub mod floodgate;
pub mod fly_config;
mod forwarding;
mod game_mode_config;
mod require_boolean;
mod server_list;
pub mod tab_list;
pub mod title;
pub mod world_config;

pub use floodgate::{EducationConfig, FloodgateConfig};
pub use forwarding::TaggedForwarding;
