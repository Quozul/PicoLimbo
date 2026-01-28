# PicoLimbo Configuration Guide

Welcome to the PicoLimbo configuration documentation. This guide will help you customize your server to meet your specific needs.

## Version Support

Some features display a badge with a version number, such as <Badge type="warning" text="1.13+" />. This indicates the minimum client version required by the feature.

If no badge is present, the feature is supported across all versions.

## Configuration Overview

PicoLimbo uses a simple TOML configuration file to manage all server settings. The configuration is organized into several sections:

- [Boss Bar](./boss-bar) - Customize the boss bar
- [Commands](./commands) - Configure the commands
- [Compression](./compression) - Configure the compression
- [Proxy Integration](./proxy-integration) - Configure proxy integration
- [Schematic Loading](./schematic-loading) - Experimental features for world customization
- [Server List](./server-list) - Customize your server's appearance in the Minecraft client
- [Server Settings](./server-settings) - Core server configuration
- [Tab List](./tab-list) - Configure the tab list
- [Title](./title) - Configure titles
- [World](./world) - Configure the world

## Environment Variables

You can use environment variable placeholders everywhere in the configuration file using the `${VARIABLE_NAME}` syntax.
