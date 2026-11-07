# Addons Forever artwork

Artwork requested in the project's art brief (docs/ART.md, not published), generated with the built-in
`image_gen` tool. The final prompts are recorded in [prompts.json](prompts.json).
All PNGs are RGBA with real alpha transparency.

| Files | Pixel dimensions / representations |
| --- | --- |
| `icon-16.png`, `icon-24.png`, `icon-32.png`, `icon-48.png`, `icon-64.png`, `icon-128.png`, `icon-256.png`, `icon-512.png`, `icon-1024.png` | Square app icons at the named sizes |
| `app.ico` | 16, 24, 32, 48, 64, 128, 256; 32-bit RGBA PNG entries |
| `app.icns` | 16, 32, 64, 128, 256, 512, 1024 pixels, including Retina representations |
| `tray-16.png`, `tray-32.png` | 16×16 and 32×32 colored tray icons |
| `tray-mono.png` | 32×32 white silhouette for macOS template use |
| `empty-discover.png`, `empty-addons.png`, `empty-log.png` | 96×96 transparent line-art illustrations |
| `badge-update.png`, `badge-update-32.png` | 16×16 and 32×32 update badges |
| `splash.png` | 1024×512 first-run artwork |

The ICO directory format supports dimensions through 256 pixels. The requested
512- and 1024-pixel versions are supplied as PNGs and in the ICNS container.
PNG exports use Lanczos downsampling.

Integration (v0.2.3): `app.ico` is compiled into the Windows `.exe` resources
(`crates/gui/build.rs`); window icon = `icon-32.png` and tray icon = `tray-32.png`,
embedded and decoded at startup (`crates/gui/src/icon.rs`); `empty-*.png` illustrate the
empty My Addons / Discover / Log states and `badge-update.png` marks pending updates in
the status bar (`crates/gui/src/tabs.rs`); `app.icns` is used for the macOS `.app`
bundle; `splash.png` and `hero.png` are spare artwork, not shown in the app.

