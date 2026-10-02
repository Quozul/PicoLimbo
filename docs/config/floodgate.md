# Floodgate

PicoLimbo can accept the authenticated Floodgate data that Geyser forwards in the Java handshake. Geyser and Floodgate stay on your proxy; PicoLimbo only needs the same Floodgate key used by that proxy.

## Configuration

Floodgate is disabled by default:

```toml
[floodgate]
enabled = false
```

To enable it:

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

The Floodgate settings are optional after `enabled = true`. Missing values use the defaults shown above.

- `key_file` is the Floodgate `key.pem` shared with the proxy.
- `username_prefix` is applied to normal Bedrock usernames.
- `replace_spaces` converts spaces in Bedrock usernames to underscores.
- `education` accepts EduFloodgate/EduGeyser Education Edition payloads.
- `education_username_prefix` is applied to Education Edition usernames.
- `education_uuid_legacy` keeps the old tenant-and-username UUID scheme. Leave it false for new EduFloodgate installations.

The key file is sensitive. Do not commit it to your repository or distribute it.

## Proxy setup

Install Geyser and Floodgate on the proxy and configure Geyser to use Floodgate authentication. For proxy-to-backend Floodgate data, enable `send-floodgate-data` in Floodgate and use the same `key.pem` on the proxy and PicoLimbo. GeyserMC documents this backend setup and the shared-key requirement in its Floodgate setup guide.

PicoLimbo removes the authenticated Floodgate payload from the hostname before normal proxy forwarding is processed. This means the remaining hostname is still available to the existing BungeeCord forwarding parser.

Floodgate data can also be used alongside PicoLimbo's Velocity Modern Forwarding. Floodgate supplies the verified Bedrock identity, while the Velocity forwarding query is still authenticated separately. PicoLimbo preserves the Floodgate profile when the Velocity response is processed.

## Education Edition

For EduGeyser/EduFloodgate, set `education = true` and make sure the UUID scheme used by EduGeyser and EduFloodgate matches PicoLimbo's selected scheme. The modern scheme derives the identity from the Microsoft-verified Entra OID; the legacy scheme derives it from `tenantId:username`. The modern scheme should be used for new deployments.

```toml
[floodgate]
enabled = true
education = true
education_username_prefix = "+"
education_uuid_legacy = false
```

EduFloodgate also carries the Education Edition fields in the extended Floodgate payload. PicoLimbo validates the extended field count and Education flag before accepting the identity.

## Username limits

PicoLimbo keeps the resulting Java username within the 16-byte limit without splitting UTF-8 characters. Prefixes are limited to 16 bytes as well.

## Troubleshooting

If a Bedrock player is rejected:

1. Confirm Geyser is using Floodgate authentication.
2. Confirm the proxy Floodgate configuration has `send-floodgate-data` enabled when the data is being sent to a backend.
3. Confirm PicoLimbo's `key_file` points at the same Floodgate key used by the proxy.
4. Confirm `education = true` when using EduGeyser/EduFloodgate.
5. Check PicoLimbo's log for an invalid key, unsupported Floodgate data version, or invalid payload field count.
