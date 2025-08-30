# Default Configuration

The default configuration file will be automatically generated the first time you start the server.
If it is not generated, you can copy the following code block in your configuration file or in `server.toml` next to PicoLimbo's executable.

:::code-group
```toml [server.toml]
# Server bind address and port
bind = "0.0.0.0:25565"

# Default spawn dimension
# Allowed values: "overworld", "nether", or "end"
spawn_dimension = "overworld"

# Welcome message sent to players after spawning
welcome_message = "Welcome to PicoLimbo!"

# Sets the game mode for new players
# Allowed values: "survival", "creative", "adventure", or "spectator"
default_game_mode = "spectator"

# If set to true, will spawn the player in hardcode mode
hardcore = false

[forwarding.velocity]
# Enable Velocity Modern Forwarding
enabled = false
# Shared secret for Velocity proxy
secret = ""

[forwarding.bungee_cord]
# Enable BungeeCord forwarding
enabled = false
# Enable BungeeGuard (requires BungeeCord to be enabled)
bungee_guard = false
# List of valid BungeeGuard tokens for authenticating incoming players
tokens = []

[server_list]
# Maximum count shown in your server list, does not affect the player limit
max_players = 20
# MOTD displayed in server lists
message_of_the_day = "A Minecraft Server"
# Show actual online player count in your server list?
show_online_player_count = true

[experimental.world]
# Custom spawn position as [x, y, z] coordinates
spawn_position = [0.0, 320.0, 0.0]
# Configure how many chunks are sent to clients
view_distance = 2
# Path to schematic file for custom world structures
# Leave empty to disable schematic loading
schematic_file = ""
```
:::
