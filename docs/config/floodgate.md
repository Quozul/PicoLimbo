# Floodgate

PicoLimbo does not run Geyser or Floodgate. Those remain proxy-side components, such as Geyser/Floodgate on Velocity or BungeeCord.

This support exists for a specific proxy setup problem: Geyser/Floodgate can forward a protected Floodgate payload to the backend server. GeyserMC documents this with Floodgate's `send-floodgate-data` option, where the proxy sends Floodgate data to backend servers and the backend normally needs a Floodgate installation with the same `key.pem` to consume it.

PicoLimbo previously could not receive and process that Floodgate data, so proxy setups that depend on `send-floodgate-data` could not use PicoLimbo as the backend/limbo target. This implementation adds native support for receiving, decrypting, validating, and applying that Floodgate data directly in PicoLimbo.

See the GeyserMC documentation:
- https://geysermc.org/wiki/floodgate/setup/proxy-servers/
- https://geysermc.org/wiki/floodgate/api/

## What this changes

The intended setup is:

```
Bedrock
   |
   v
Geyser + Floodgate
   |
   | Floodgate data
   v
PicoLimbo
```

Geyser and Floodgate stay installed on the proxy. **Do not code Geyser or Floodgate in PicoLimbo.** PicoLimbo implements the backend-side Floodgate data handling itself.

This is especially useful for networks where the proxy needs to forward Floodgate data to a backend that cannot have the normal platform-specific Floodgate plugin installed.

## Configuration

Floodgate support is disabled by default. Enable it with:

```toml
[floodgate]
enabled = true
key_file = "key.pem"
username_prefix = "."
replace_spaces = true
education = false
education_username_prefix = "+"
education_uuid_legacy = false
```

The `key_file` must contain the same Floodgate key used by the proxy-side Floodgate installation. GeyserMC requires the key to match between Floodgate instances when Floodgate data is forwarded. Treat this file as a secret and never commit or distribute it.

Set `education = true` when the proxy is forwarding EduGeyser/EduFloodgate data. `education_username_prefix` controls the prefix applied to education players, and `education_uuid_legacy` enables the legacy education UUID format.

## Standard Floodgate

For a normal Geyser/Floodgate proxy setup:

1. Install and configure Geyser and Floodgate on the proxy.
2. Configure Geyser to use Floodgate authentication.
3. Enable Floodgate's `send-floodgate-data` on the proxy when the proxy is forwarding Floodgate data to PicoLimbo.
4. Configure PicoLimbo with the matching Floodgate `key.pem`.
5. Send the proxy connection to PicoLimbo.

The important difference is that PicoLimbo does **not** need the Floodgate plugin installed. It handles the forwarded Floodgate payload natively.

## Proxy forwarding

Floodgate data is carried in the connection hostname alongside the normal proxy forwarding data. PicoLimbo extracts and validates the Floodgate payload, removes it from the hostname, and then continues processing the normal BungeeCord/Velocity forwarding data.

This allows a proxy to forward both Floodgate data and normal proxy player information to PicoLimbo without requiring Floodgate itself to be installed in PicoLimbo.

## Why this is needed

GeyserMC's proxy documentation explicitly describes `send-floodgate-data` as the mechanism for passing Floodgate data from the proxy to backend servers. Their normal backend setup requires Floodgate to be installed on those backend servers when that data/API support is needed.

PicoLimbo is not a normal Spigot/Paper backend and cannot install the platform-specific Floodgate backend plugin. The native implementation in this PR fills that missing backend protocol support instead of trying to run Geyser or Floodgate inside PicoLimbo.

## Testing

At minimum, verify the Rust test suite:

```shell
cargo test -p pico_limbo
```

For an integration test, use a real Geyser/Floodgate proxy setup and test:

1. A normal Java player joining PicoLimbo.
2. A Bedrock player joining through Geyser/Floodgate with `send-floodgate-data` enabled.
3. A Bedrock player with normal BungeeCord or Velocity forwarding enabled at the same time.
4. A Bedrock username containing spaces when `replace_spaces` is enabled.
5. An EduGeyser/EduFloodgate player with `education` enabled.
6. A malformed Floodgate payload.
7. A payload encrypted with the wrong key.
8. A connection without Floodgate data, which should continue to work normally.

The Floodgate key must never be committed to the repository or shared publicly.
