# Floodgate

PicoLimbo can accept Floodgate data directly from Geyser without installing Floodgate or Geyser in PicoLimbo.

## Configuration

Floodgate is disabled by default. Enable it with:

```toml
[floodgate]
method = "ENABLED"
key_file = "key.pem"
username_prefix = "."
replace_spaces = true

[floodgate.education]
enabled = false
username_prefix = "+"
uuid_legacy = false
```

The `key_file` must point to the same Floodgate `key.pem` used by the Geyser/Floodgate side. Treat this file as a secret.

Set `floodgate.education.enabled = true` when accepting EduGeyser/EduFloodgate data. The education username prefix and UUID compatibility mode can then be configured independently.

## Proxy forwarding

Floodgate data is read from the handshake hostname and removed before the normal BungeeCord/Velocity forwarding parser runs. This allows Floodgate and existing forwarding to be used together.

## Testing

At minimum, verify the normal Rust test suite:

```shell
cargo test -p pico_limbo
```

For an integration test, run a Geyser/Floodgate setup with the same `key.pem` configured for both sides and point Geyser at PicoLimbo. Test:

1. A Java player joining normally.
2. A Bedrock player joining through Geyser with standard Floodgate data.
3. A Bedrock player joining while BungeeCord or Velocity forwarding is enabled.
4. A Bedrock username containing spaces when `replace_spaces` is enabled.
5. An EduGeyser/EduFloodgate player with education mode enabled.
6. A malformed or incorrectly encrypted Floodgate payload, which should be rejected.

The Floodgate key must never be committed to the repository or shared publicly.
