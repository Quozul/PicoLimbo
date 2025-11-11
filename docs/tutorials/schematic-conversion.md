# Convert Schematics to use with PicoLimbo

PicoLimbo requires schematics in **Sponge schematic format version 2** (`.schem`). This format was introduced in WorldEdit 7 (Minecraft 1.13+) to support modern block states and NBT data.

### Supported Formats
- **Sponge v2** (`.schem`) - Required by PicoLimbo

## Convert

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
