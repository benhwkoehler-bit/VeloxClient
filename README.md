# VeloxClient

A fast, lightweight Minecraft launcher with its own performance pack.
Based on [OneClient](https://github.com/Polyfrost/OneLauncher) by Polyfrost.

> **Beta:** VeloxClient is currently in beta.

## Download

**Windows:** [Download VeloxClient Beta](https://github.com/benhwkoehler-bit/VeloxClient/releases/tag/beta)
(`VeloxClient-beta-windows-setup.exe`)

The installer is not signed yet. If Windows shows "Windows protected your PC",
click **More info → Run anyway**.

## Features

- **Velox performance pack** for Minecraft 26.2 and 26.3, with separate packs for Windows and macOS.
  Sodium, Lithium, FerriteCore and many more are included out of the box
- **Optional extras** on first start: minimap, world map, AppleSkin, Chat Heads and more
- **Pure Metal** on Apple Silicon Macs (26.3): Minecraft renders through Apple's Metal, alongside Sodium
- **Import your profiles** from Prism, MultiMC, Modrinth App, NoRisk and Lunar. The originals stay untouched
- Older versions (1.8.9, 1.21.x, 26.1.2) run without Velox mods
- Purple design with a pixel font (Monocraft) and a terminal look

## Building

Requires [Rust](https://rustup.rs/) 1.97 or newer.

```sh
# Run the app
cargo run -p oneclient_app

# Release build
cargo build -p oneclient_app --release
```

Installers are built with [cargo-packager](https://github.com/crabnebula-dev/cargo-packager)
(Windows: `nsis`, macOS: `app,dmg`). The Windows installer is built automatically by
GitHub Actions on every push to `main`.

## License

Like OneClient, VeloxClient is licensed under the [GPL-3.0](./LICENSE).
Based on OneClient by Polyfrost. See also [ATTRIBUTION.md](./ATTRIBUTION.md).
Minecraft is a trademark of Mojang Studios. VeloxClient is not an official Minecraft product.
