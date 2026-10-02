# Floodgate

Representing the `[floodgate]` section in `server.toml`.

PicoLimbo can validate and accept the authenticated Floodgate data that Geyser forwards to a backend server. PicoLimbo does not need Geyser or Floodgate installed itself; it only needs the shared Floodgate key when this integration is enabled.

## Disabled

Floodgate support is disabled by default:

~~~toml [server.toml]
[floodgate]
enabled = false
~~~

## Enabled

When setting `enabled` to `true`, PicoLimbo loads the shared Floodgate key and accepts authenticated Floodgate handshake data:

~~~toml [server.toml]
[floodgate]
enabled = true
key_file = "key.pem"
username_prefix = "."
replace_spaces = true
education = false
education_username_prefix = "+"
education_uuid_legacy = false
~~~

All properties other than `enabled` have defaults, so they may be omitted.

## Key File

The `key_file` option points to the same Floodgate `key.pem` used by the Geyser/Floodgate proxy.

~~~toml [server.toml]
[floodgate]
enabled = true
key_file = "key.pem"
~~~

The key is a shared secret used to authenticate the encrypted Floodgate player data. Floodgate uses a 128-bit AES key, so PicoLimbo expects the key file to contain the same raw 16-byte key as the proxy. Geyser loads the Floodgate key from the configured key path and uses it for Floodgate encryption. See the [Floodgate proxy setup](https://geysermc.org/wiki/floodgate/setup/proxy-servers/) documentation for the shared-key setup.

**Never commit or distribute this key to untrusted systems.** Geyser requires backend and proxy Floodgate keys to be identical when Floodgate data is forwarded to a backend. See the [Floodgate setup](https://geysermc.org/wiki/floodgate/setup/) documentation.

## Username Prefix

The `username_prefix` is added to normal Bedrock usernames.

~~~toml [server.toml]
[floodgate]
enabled = true
username_prefix = "."
~~~

The default is `.`, so a Bedrock player named `Steve` becomes `.Steve`.

## Replace Spaces

Set `replace_spaces` to `true` to replace spaces in Bedrock usernames with underscores.

~~~toml [server.toml]
[floodgate]
enabled = true
replace_spaces = true
~~~

The default is `true`.

## Education Edition

Set `education` to `true` to accept EduGeyser/EduFloodgate Education Edition player data:

~~~toml [server.toml]
[floodgate]
enabled = true
education = true
education_username_prefix = "+"
education_uuid_legacy = false
~~~

Education Edition players use a separate username prefix. The default is `+`.

EduFloodgate uses the extended Floodgate player-data format for both ordinary Bedrock and Education players. The Education flag in that payload is `0` for a normal Bedrock player and `1` for an Education player. Therefore, enabling or disabling Education support does **not** prevent ordinary Bedrock players from joining.

## Education UUID Scheme

The `education_uuid_legacy` option selects the UUID scheme used for Education Edition players.

~~~toml [server.toml]
[floodgate]
enabled = true
education = true
education_uuid_legacy = false
~~~

When `false`, PicoLimbo uses the modern Microsoft-verified identity scheme. When `true`, it uses the legacy tenant-and-username scheme.

This setting must match the UUID scheme configured by EduGeyser/EduFloodgate. Changing the scheme can change the Java UUID used for Education players, so existing player data may no longer match.

## Proxy Setup

Floodgate data is sent by the proxy to the backend server through the Minecraft handshake.

### Geyser and Floodgate on Velocity

Install Geyser and Floodgate on the Velocity proxy and configure Geyser to use Floodgate authentication.

When Floodgate data needs to be forwarded to PicoLimbo, enable `send-floodgate-data` in the proxy Floodgate configuration and copy the proxy `key.pem` to the PicoLimbo server. The same key must be used on both sides. See the [Floodgate proxy setup](https://geysermc.org/wiki/floodgate/setup/proxy-servers/) documentation.

PicoLimbo validates and removes the authenticated Floodgate payload before the remaining hostname is processed.

### Velocity Modern Forwarding

Floodgate can be used together with Velocity Modern Forwarding.

With Modern Forwarding enabled, PicoLimbo performs the Velocity forwarding exchange during login. The final Java profile comes from the authenticated Velocity forwarding response; the Floodgate payload is still validated during the initial handshake.

This means `education = true` is still required when EduGeyser/EduFloodgate sends an Education player, even when Modern Forwarding is enabled.

For the Velocity Modern Forwarding configuration itself, see the [Proxy Integration](./proxy-integration.md) documentation.

## Security

Floodgate data is authenticated with the shared key before PicoLimbo accepts the player identity. PicoLimbo does not trust a Bedrock username or XUID supplied without a valid Floodgate payload.

Keep the Floodgate key private. Anyone who obtains it can forge authenticated Floodgate player data.

## Troubleshooting

### `Floodgate authentication failed`

Check that `key_file` points to the exact same `key.pem` used by the proxy Floodgate installation.

### `Invalid Education Floodgate flag`

Check that the forwarded EduFloodgate payload uses `0` for ordinary Bedrock or `1` for Education Edition. Ordinary Bedrock players using the extended 15-field format are valid with `education = false`.

### Education players are rejected

Set:

~~~toml [server.toml]
[floodgate]
enabled = true
education = true
~~~

Also make sure the Education UUID scheme matches the scheme used by EduGeyser/EduFloodgate.

### Floodgate data is not received

On the proxy, verify that Floodgate has `send-floodgate-data` enabled when the backend is expected to receive Floodgate data.
