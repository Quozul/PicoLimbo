# Frequently Asked Questions

## Is PicoLimbo compatible with Bukkit, Spigot, or Paper plugins?

No, PicoLimbo is not compatible with traditional Java-based plugins designed for Bukkit, Spigot, Paper, or other Java Minecraft server implementations. This is because PicoLimbo is not written in Java and does not implement the Bukkit API or similar plugin frameworks.

While adding support for plugins is theoretically possible, this is not currently a planned feature. PicoLimbo is designed to be a lightweight limbo server with minimal resource usage, and plugin support would significantly increase complexity.

## Does PicoLimbo support modded clients?

No, PicoLimbo does not support modded clients. There is an ongoing effort to provide minimal support for Fabric mods, but don't expect anything more.

## Can I use ViaVersion with PicoLimbo?

PicoLimbo already supports all Minecraft versions natively, so there is no need to use ViaVersion for protocol compatibility.

However, if you absolutely want to use ViaVersion in your setup, PicoLimbo is compatible with ViaVersion when it is installed on your proxy server.

## Can I enable online mode authentication?

No, PicoLimbo runs in offline mode by default, and there is currently no way to enable online mode authentication. There are also no plans to add online mode support in the future.

If you need authenticated players, you should handle authentication at the proxy level before players are sent to the PicoLimbo server.

## Can PicoLimbo load worlds or generate terrain?

PicoLimbo cannot load existing worlds or generate terrain. Players connect to a void environment by default.

However, PicoLimbo includes experimental support for loading small structures using `.schem` files. See the [Experimental World Loading](/config/world.html) section for configuration details.
s
## Does PicoLimbo support Bedrock players?

While it is not planned in the near future, I understand the need for such a feature. In the meantime, you can probably install Geyser as a plugin on a Velocity proxy.
