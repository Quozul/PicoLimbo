use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
pub struct ServerListConfig {
    pub reply_to_status: bool,

    /// Maximum amount of player displayed in the server list.
    pub max_players: u32,

    /// Description of the server displayed in the server list.
    pub message_of_the_day: String,

    /// Set to false to always show 0 online players
    pub show_online_player_count: bool,

    pub server_icon: PathBuf,
}

impl Default for ServerListConfig {
    fn default() -> Self {
        Self {
            reply_to_status: true,
            max_players: 20,
            message_of_the_day: "A Minecraft Server".into(),
            show_online_player_count: true,
            server_icon: PathBuf::from("server-icon.png"),
        }
    }
}
