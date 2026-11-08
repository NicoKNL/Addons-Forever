# AGENTS.md

Notes for AI agents working on this repository.

## What this is

Addons Forever: a portable addon manager for World of Warcraft: Forever
(client 1.60.x, interface 16001, Curse flavor 88568). Rust workspace:

- `crates/core` (`addons-forever-core`): config, providers (CurseForge,
  GitHub, Tukui, WoWInterface, WowUp Hub, plain zip), scanning, install,
  update, fingerprint matching, dependency resolution. No UI code.
- `crates/gui` (`addons-forever`): egui/eframe app (`main.rs`, `app.rs`,
  `tabs.rs`), the worker bridge (`worker.rs`), a headless `cli`
  subcommand (`cli.rs`), tray, self-update.

## Design rules (do not break these)

- Privacy by design: no telemetry. Network traffic goes only to provider
  domains on the allowlist in `crates/core/src/net.rs`. The Curse
  community API key is intentionally public.
- Portable: one binary, one `.ini` next to it. The data directory must
  contain only the `.ini`. Addon state lives in memory; user intent is
  persisted as `[addon <provider>:<id>]` sections in the ini. Never add
  state files, log files, or caches to the data dir. Logs exist only
  in memory, saved by an explicit user button.
- Version comparison (`util::version_cmp`): letter suffixes order
  (`1.0.0d > 1.0.0c`), client flavor suffixes are identity
  (`1.60.28 == 1.60.28-forever == 1.60.28-WOTLK`). Keep both properties.
- Dependencies (`engine/deps.rs`): parse the TOC keys `Dependencies`,
  `RequiredDeps`, `OptionalDependencies`, `OptionalDeps`. A dependency is
  satisfied by an installed folder, by a bundle sibling, or by an
  embedded library (sub-folder name or TOC title inside the installed
  addon). Unmet deps are resolved by exact (color-code and marketing
  suffix cleaned) name against CurseForge, WowUp Hub, Tukui, MMOUI,
  filtered to the user's flavor, then presented as checkboxes. Provider
  TOC lines are UTF-8 and contain Cyrillic text: never slice TOC text by
  byte offset without ASCII byte comparison (`strip_prefix_ci`).
- TOC/dependency parsing must never panic on real-world input; the test
  suite has fixtures for hostile TOC files.

## Release workflow

- Do not tag or publish a release on every commit. The human tests
  builds first; cut a release (version bump, tag, push tag) only when
  explicitly asked.
- Release artifacts are built by GitHub Actions on tag push (`.github/
  workflows/release.yml`): Windows from `x86_64-pc-windows-gnu` (MSVC
  builds trigger SmartScreen prompts; keep gnu), Linux, and macOS `.app`
  bundles (ad-hoc signed, archive names end `-macos.zip`/`-macos.tar.gz`,
  never `.app.zip`). Artifact names use the version from `Cargo.toml`,
  no leading `v`, on every platform.
- Linux builds are shipped provided as-is (untested); say so in docs.

## Building and testing

```sh
cargo build --workspace            # dev build
cargo test --workspace             # unit + fixture tests
cargo clippy --workspace --all-targets
FA_LIVE=1 cargo test --workspace -- --include-ignored   # live provider tests
```

- Live tests hit real providers; `FA_LIVE=1` is the opt-in. Add new live
  coverage as `#[ignore]` tests, and keep provider assumptions flavor
  honest: the Forever search indexes are sparse (many addons resolve via
  Tukui/Hub, not Curse).
- GUI smoke tests: run the app under Xvfb with `ADDONS_FOREVER_DATA_DIR`
  pointed at a scratch dir. Screenshots come back Retina-scaled; convert
  click coordinates before driving input.
- `docs/ART.md` and `docs/PLAN.md` are private working docs, gitignored.

## Text style

User-facing strings, README, PRIVACY: plain prose, sentence case, no em
dashes, no marketing vocabulary. Keep changelog-style commit messages
factual.

## Naming

- Project name in prose: "Addons Forever". Repository slug and Cargo
  names use `addons-forever` / `addons-forever-core`.
- Windows release exe: `addons-forever.exe` (no version suffix).
