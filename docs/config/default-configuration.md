# Default Configuration

The default configuration file will be automatically generated the first time you start the server.

```toml
# Server bind address and port
bind = "0.0.0.0:25565"

# Default spawn dimension
# Allowed values: "overworld", "nether", or "end"
spawn_dimension = "overworld"

# Welcome message sent to players after spawning
welcome_message = "Welcome to PicoLimbo!"

# Sets the game mode for new players
# Allowed values: "survival", "creative", or "adventure"
default_game_mode = "spectator"

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
```
