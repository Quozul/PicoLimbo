use crate::configuration::commands::CommandsConfig;

#[derive(Default)]
pub struct ServerCommands {
    spawn: String,
    fly: String,
    fly_speed: String,
}

impl From<CommandsConfig> for ServerCommands {
    fn from(config: CommandsConfig) -> Self {
        Self {
            spawn: config.spawn,
            fly: config.fly,
            fly_speed: config.fly_speed,
        }
    }
}

pub enum ServerCommand {
    Disabled,
    Enabled { alias: String },
}

impl ServerCommands {
    pub fn spawn(&self) -> ServerCommand {
        Self::server_command(self.spawn.clone())
    }

    pub fn fly(&self) -> ServerCommand {
        Self::server_command(self.fly.clone())
    }

    pub fn fly_speed(&self) -> ServerCommand {
        Self::server_command(self.fly_speed.clone())
    }

    fn server_command(alias: String) -> ServerCommand {
        if alias.is_empty() {
            ServerCommand::Disabled
        } else {
            ServerCommand::Enabled { alias }
        }
    }
}
