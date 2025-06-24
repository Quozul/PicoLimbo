use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ServerListConfig {
    /// Maximum amount of player displayed in the server list.
    pub max_players: u32,

    /// Description of the server displayed in the server list.
    pub message_of_the_day: String,

    /// Set to false to always show 0 online players
    pub show_online_player_count: bool,
}

impl Default for ServerListConfig {
    fn default() -> Self {
        Self {
            max_players: 20,
            message_of_the_day: "A Minecraft Server".into(),
            show_online_player_count: true,
        }
    }
}
