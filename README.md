# VeloxClient

Ein schneller, schlanker Minecraft-Launcher mit eigenem Performance-Pack.
Basiert auf [OneClient](https://github.com/Polyfrost/OneLauncher) von Polyfrost.

> **Beta:** VeloxClient ist gerade in der Testphase für Freunde.

## Download

**Windows:** [VeloxClient Beta herunterladen](https://github.com/benhwkoehler-bit/VeloxClient/releases/tag/beta)
(`VeloxClient-beta-windows-setup.exe`)

Der Installer ist noch nicht signiert. Wenn Windows „Der Computer wurde durch Windows geschützt“
anzeigt: **Weitere Informationen → Trotzdem ausführen**.

## Was VeloxClient kann

- **Velox Performance-Pack** für Minecraft 26.2 und 26.3 – eigene Packs für Windows und macOS,
  Sodium, Lithium, FerriteCore und vieles mehr sind schon drin
- **Wählbare Extras** beim ersten Start: Minimap, Weltkarte, AppleSkin, Chat Heads und mehr
- **Pure Metal** auf Macs mit Apple Silicon (26.3): Minecraft läuft über Apples Metal, zusammen mit Sodium
- **Profile übernehmen** aus Prism, MultiMC, Modrinth App, NoRisk und Lunar – die Originale bleiben unverändert
- Ältere Versionen (1.8.9, 1.21.x, 26.1.2) laufen ohne Velox-Mods
- Lila Design mit Pixel-Schrift (Monocraft) und Terminal-Look

## Selbst bauen

Benötigt [Rust](https://rustup.rs/) 1.97 oder neuer.

```sh
# App starten
cargo run -p oneclient_app

# Release-Build
cargo build -p oneclient_app --release
```

Installer werden mit [cargo-packager](https://github.com/crabnebula-dev/cargo-packager) gebaut
(Windows: `nsis`, macOS: `app,dmg`). Der Windows-Installer entsteht automatisch über
GitHub Actions bei jedem Push auf `main`.

## Lizenz

VeloxClient steht wie OneClient unter der [GPL-3.0](./LICENSE).
Based on OneClient by Polyfrost – siehe auch [ATTRIBUTION.md](./ATTRIBUTION.md).
Minecraft ist eine Marke von Mojang Studios. VeloxClient ist kein offizielles Minecraft-Produkt.
