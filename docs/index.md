---
# https://vitepress.dev/reference/default-theme-home-page
layout: home

hero:
  name: PicoLimbo
  text: The lightweight limbo server
  tagline: An ultra-lightweight, multi-version Minecraft limbo server written in Rust
  actions:
    - theme: brand
      text: Download
      link: https://github.com/Quozul/PicoLimbo/releases/latest
    - theme: alt
      text: Documentation
      link: /about/introduction/
    - theme: alt
      text: Community & Support
      link: https://discord.gg/M2a9dxJPRy

features:
  - icon: ⚙️
    title: Highly Configurable
    details: Customize your server using a simple TOML configuration file, including welcome message, spawn dimension, server list MOTD, and more.
    link: /config/introduction/
    linkText: Read the documentation
  - icon: 🔀
    title: Built-in Proxy Support
    details: "Seamlessly integrates with major Minecraft proxies: Velocity, BungeeCord and BungeeGuard authentication"
    link: /config/proxy-integration
    linkText: Read the documentation
  - icon: 🎮
    title: Wide Version Compatibility
    details: Supports all Minecraft versions starting from 1.7.2 natively, no need for ViaVersion or additional compatibility layers
    link: /about/supported-versions.html
    linkText: See all supported versions
  - icon: ⚡
    title: Ultra-Lightweight & Highly Scalable
    details: Uses 0% CPU while idle and under 10 MB RAM, enabling thousands of concurrent players thanks to Rust’s asynchronous runtime and efficient design
    link: /about/benchmarks.html
    linkText: See benchmarks
---
