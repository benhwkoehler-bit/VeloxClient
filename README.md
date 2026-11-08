# VeloxClient

A fast, lightweight Minecraft launcher with its own performance pack.
Based on [OneClient](https://github.com/Polyfrost/OneLauncher) by Polyfrost.

> **Beta:** VeloxClient is currently in beta.

> **New look:** VeloxClient is now black and white. The purple design is gone, and the logo and app icon are black and white too.

## Download

[Download VeloxClient Beta](https://github.com/benhwkoehler-bit/VeloxClient/releases/tag/beta)

| System | File |
| --- | --- |
| Windows | `VeloxClient-beta-windows-setup.exe` |
| macOS (Apple Silicon, M1 or newer) | `VeloxClient-beta-macos-apple-silicon.dmg` |
| macOS (Intel) | `VeloxClient-beta-macos-intel.dmg` |

The builds are not signed yet:

- **Windows:** if "Windows protected your PC" appears, click **More info → Run anyway**.
- **macOS:** open the `.dmg` and drag VeloxClient into Applications. On first launch,
  right-click the app and choose **Open**, or allow it under
  **System Settings → Privacy & Security**.

## Features

- **Velox performance pack** for Minecraft 26.2 and 26.3, with separate packs for Windows and macOS.
  Sodium, Lithium, FerriteCore and many more are included out of the box
- **Optional extras** on first start: minimap, world map, AppleSkin, Chat Heads and more
- **Pure Metal** on Apple Silicon Macs (26.3): Minecraft renders through Apple's Metal, alongside Sodium
- **Import your profiles** from Prism, MultiMC, Modrinth App, NoRisk and Lunar. The originals stay untouched
- Older versions (1.8.9, 1.21.x, 26.1.2) run without Velox mods
- Black and white design with liquid glass, a pixel font (Monocraft) and a terminal look

## Building

Requires [Rust](https://rustup.rs/) 1.97 or newer.

```sh
# Run the app
cargo run -p oneclient_app

# Release build
cargo build -p oneclient_app --release
```

Installers are built with [cargo-packager](https://github.com/crabnebula-dev/cargo-packager)
(Windows: `nsis`, macOS: `app,dmg`). The Windows installer and the macOS disk images are built
automatically by GitHub Actions on every push to `main`.

## License

Like OneClient, VeloxClient is licensed under the [GPL-3.0](./LICENSE).
Based on OneClient by Polyfrost. See also [ATTRIBUTION.md](./ATTRIBUTION.md).
Minecraft is a trademark of Mojang Studios. VeloxClient is not an official Minecraft product.
