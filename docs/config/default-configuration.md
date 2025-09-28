# Default Configuration

The default configuration file will be automatically generated the first time you start the server.
If it is not generated, you can copy the following code block in your configuration file or in `server.toml` next to PicoLimbo's executable.

:::code-group
```toml [server.toml]
# Server bind address and port
bind = "0.0.0.0:25565"
# Welcome message sent to players after spawning
welcome_message = "Welcome to PicoLimbo!"
action_bar = "Welcome to PicoLimbo!"
# Sets the game mode for new players
# Allowed values: "survival", "creative", "adventure", or "spectator"
default_game_mode = "spectator"
# If set to true, will spawn the player in hardcode mode
hardcore = false
# Set to true to fetch the skin textures from Mojang API
fetch_player_skins = false
# Enable compression
compression_threshold = -1

[forwarding]
# Disable forwarding
method = "NONE"

[tab_list]
# Enable tab list customization
enabled = true
# The header text displayed at the top of the player list
header = "<bold>Welcome to PicoLimbo</bold>"
# The footer text displayed at the bottom of the player list
footer = "<green>Enjoy your stay!</green>"

[server_list]
# Maximum count shown in your server list, does not affect the player limit
max_players = 20
# MOTD displayed in server lists
message_of_the_day = "A Minecraft Server"
# Show actual online player count in your server list?
show_online_player_count = true

[world]
# Custom spawn position as [x, y, z] coordinates
spawn_position = [0.0, 320.0, 0.0]
# Default spawn dimension
# Allowed values: "overworld", "nether", or "end"
dimension = "end"
# Sets the time in the world
# Allowed values: "day", "noon", "night", "midnight", or a specific time in ticks (0-24000)
time = "day"

[world.boundaries]
# Enable world boundaries
enabled = true
# Minimum Y position, players below this will be teleported back to spawn
min_y = -64
# Message displayed when a player reaches the minimum Y position
teleport_message = "You have reached the bottom of the world."

[world.experimental]
# Configure how many chunks are sent to clients
view_distance = 2
# Path to schematic file for custom world structures
# Leave empty to disable schematic loading
schematic_file = ""
# Lock the time in the world to `world.time` value
lock_time = false

[boss_bar]
# Enable boss bar display
enabled = true
# Boss bar title displayed to players
title = "<blue><bold>Welcome to PicoLimbo!</bold></blue>"
# Boss bar health (0.0 to 1.0, where 1.0 is full health)
health = 1
# Boss bar color
# Allowed values: "blue", "green", "pink", "purple", "red", "white", or "yellow"
color = "blue"
# Boss bar style
# Allowed values: 0, 6, 10, 12 or 20, representing the number of segments
division = 0

[title]
enabled = true
title = "<bold>Welcome!</bold>"
subtitle = "Enjoy your stay"
fade_in = 10
stay = 70
fade_out = 20
```
:::
