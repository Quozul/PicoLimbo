# Frequently Asked Questions

## Is PicoLimbo compatible with Bukkit, Spigot, or Paper plugins?

No, PicoLimbo does not load Bukkit, Spigot, Paper, or other Java server plugins. It has its own lightweight plugin API for Rhai scripts. Plugins live in the `plugins/` directory and are loaded independently; a fresh installation includes no plugins. See [Plugin Development](/developer/plugins.html) to create one.

## Does PicoLimbo support modded clients?

No, PicoLimbo does not support modded clients. There is an ongoing effort to provide minimal support for Fabric mods, but don't expect anything more.

## Can I use ViaVersion with PicoLimbo?

PicoLimbo already supports all Minecraft versions natively, so there is no need to use ViaVersion for protocol compatibility.

However, if you absolutely want to use ViaVersion in your setup, PicoLimbo is compatible with ViaVersion when it is installed on your proxy server.

If you have issues with ViaVersion, see [Troubleshooting](./troubleshooting).

## Can I enable online mode authentication?

No, PicoLimbo runs in offline mode by default, and there is currently no way to enable online mode authentication. There are also no plans to add online mode support in the future.

If you need authenticated players, you should handle authentication at the proxy level before players are sent to the PicoLimbo server. See the [Authentication](../tutorials/authentication.md) tutorial.

## Can PicoLimbo load worlds or generate terrain?

PicoLimbo cannot load existing worlds or generate terrain. Players connect to a void environment by default.

However, PicoLimbo includes experimental support for loading small structures using `.schem` files. See the [Experimental World Loading](/config/world.html) section for configuration details.

## Does PicoLimbo support Bedrock players?

While it is not planned in the near future, I understand the need for such a feature. In the meantime, you can probably install Geyser as a plugin on a Velocity proxy.

## Can we see other players?

No. This defeats the whole purpose of a limbo server.
