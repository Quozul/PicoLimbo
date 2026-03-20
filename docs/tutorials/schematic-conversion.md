# Convert Schematics to use with PicoLimbo

PicoLimbo supports **Sponge schematic format version 2 and version 3** (`.schem`). The version 2 was introduced in WorldEdit 7 (Minecraft 1.13+) to support modern block states and NBT data.

### Supported Formats
- **Sponge v2** (`.schem`)
- **Sponge v3** (`.schem`) - Supported by PicoLimbo since version 1.12.0

## Convert

### Using SchemConvert

1. Download [SchemConvert](https://github.com/PiTheGuy/SchemConvert/releases/latest) from GitHub
2. Start it using the CLI or by double-clicking on the downloaded JAR file:
   ```shell
   java -jar SchemConvert-1.2.5-all.jar
   ```
3. Select your input, output schematic and the `.schem` "Convert Type"
4. Click "Convert"

### Using WorldEdit
1. Load the schematic on a Minecraft server:
   ```
   //schem load <filename>
   ```
2. Re-export the schematic using the correct format:
   ```
   //schem save <filename> sponge.2
   ```
   This ensures the output is in Sponge v2 format.
