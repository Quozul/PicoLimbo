# Troubleshooting and Common Issues

## Network Protocol Error when using ViaVersion

When ViaVersion is installed on the proxy, you may encounter "Network Protocol Error" when logging in PicoLimbo.
In the Velocity section of your ViaVersion's `config.yml`, ensure the protocol version number is set to `-1`, at least for the limbo server:
```yaml
velocity-servers:
  default: 775
  limbo: -1
```

## MiniMessage Parsing Error

If you see an error like `Failed to parse MiniMessage: ill-formed document: entity or character reference not closed: ; not found before end of input`, it is likely caused by an unescaped `&` symbol in your configuration.

MiniMessage uses `&` for entity references. To use a literal `&` symbol, you must escape it as `&amp;`.

