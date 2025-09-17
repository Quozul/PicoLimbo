This document explains the evolution of the data generator formats across versions.

# Format V0
For all versions prior to when the data generator were added, we must manually provide all the necessary files. For now, we only need the `packets.json` file for these older versions.

# Format V1 (since 1.13)
First version to have reports. The only valuable file is the `blocks.json` file.

```
V1_13
├─ data
│  └─ minecraft
│     └─ tags
│        └─ We have sub-directories here...
└─ reports
   ├─ blocks.json
   └─ items.json
```

# Format V2 (since 1.14)
This version adds `registries.json`. This file includes the "protocol_id" for items as well, which were previously defined in the `items.json` file.
Loot tables were also added.

```
V1_14
├─ data
│  └─ minecraft
│     └─ tags
│        └─ We have sub-directories here...
└─ reports
   ├─ blocks.json
   └─ registries.json
```

# Format V3 (since 1.16.2)
This version adds the `biomes` directory, which contains a .json file for every biome in the game.

```
V1_16_2
├─ data
│  └─ minecraft
│     └─ tags
│        └─ We have sub-directories here...
└─ reports
   ├─ biomes
   │  └─ ...
   ├─ blocks.json
   └─ registries.json
```

Here we are missing the information about `dimension_type` which should be provided manually.
Also, biomes should be relocated to the `data/minecraft` directory instead of `reports` to match the format V6.

# Format V4 (since 1.18)
This version adds information for `dimension_types` and relocates the biomes' information.
A lot more data were added into the `worldgen` directory, but only the `biome` and `dimension_type` are relevant information.

```
V1_18
├─ data
│  └─ minecraft
│     └─ tags
│        └─ We have sub-directories here...
└─ reports
   ├─ worldgen
   │  └─ minecraft
   │     ├─ dimension
   │     │  └─ ...
   │     ├─ dimension_type
   │     │  └─ ...
   │     └─ worldgen
   │        ├─ biome
   │        │  └─ ...
   │        └─ ...
   ├─ blocks.json
   └─ registries.json
```

Biomes and dimension types should be relocated to the `data/minecraft` directory instead of `reports` to match the format V6.

# Format V5 (since 1.19)
The reports were relocated.

```
V1_19_3
├─ data
│  └─ minecraft
│     └─ tags
│        └─ We have sub-directories here...
└─ reports
   ├─ minecraft
   │  ├─ chat_type
   │  │  └─ ...
   │  ├─ dimension_type
   │  │  └─ ...
   │  └─ worldgen
   │     ├─ biome
   │     │  └─ ...
   │     └─ ...
   ├─ blocks.json
   └─ registries.json
```

Biomes and dimension types and chat types should be relocated to the `data/minecraft` directory instead of `reports` to match the format V6.

# Format V6 (since 1.19.3)
The reports were relocated to the data directory.

```
V1_19_3
├─ data
│  └─ minecraft
│     ├─ chat_type
│     │  └─ ...
│     ├─ dimension_type
│     │  └─ ...
│     ├─ tags
│     │  └─ We have sub-directories here...
│     └─ worldgen
│        ├─ biome
│        │  └─ ...
│        └─ ...
└─ reports
   ├─ blocks.json
   └─ registries.json
```

# Format V7 (since 1.21)
The `packets.json` report was added. In all previous versions, the `packets.json` file must be manually provided as an override.

```
V1_21
├─ data
│  └─ minecraft
│     ├─ chat_type
│     │  └─ ...
│     ├─ dimension_type
│     │  └─ ...
│     ├─ tags
│     │  └─ We have sub-directories here...
│     └─ worldgen
│        ├─ biome
│        │  └─ ...
│        └─ ...
└─ reports
   ├─ blocks.json
   ├─ packets.json
   └─ registries.json
```