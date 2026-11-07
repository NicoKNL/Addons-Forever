# Engine spec (M3): scan/ingest, install pipeline, update queue

Files: `crates/core/src/engine/{scan,install,update}.rs` (replace stubs).
Read `types.rs`, `store.rs`, `config.rs`, `fingerprint.rs`, `toc.rs`, `net.rs`,
`providers/mod.rs` first — the contract. Providers implement `Provider` (trait in
`types.rs`); engine consumes `Registry` (`providers/mod.rs`). Do not modify provider
files (other agents own them) — code against the trait.

## scan.rs — folder scanning + ingest

```rust
pub struct ScanReport { pub matched: Vec<ScanMatch>, pub unmatched: Vec<FolderInfo>, pub scanned_at: i64 }
pub fn scan_folders(addons_dir: &Path, flavor: Flavor) -> Result<Vec<FolderInfo>>
pub async fn scan_addons(ctx: &AppCtx, addons_dir: &Path, sel: &FileSelect, on_progress: &mut dyn FnMut(&str /*folder*/, usize /*done*/, usize /*total*/)) -> Result<ScanReport>
pub fn apply_scan_report(store: &mut StateStore, report: &ScanReport, cfg: &AppConfig)
```
- scan_folders: enumerate subdirs of AddOns (skip dot-dirs); for each: toc::find_toc
  (flavor), parse, fingerprint::folder_fingerprint. No TOC → FolderInfo with empty ParsedToc
  (fingerprint None) so it can become an "unmanaged local" addon.
- scan_addons: for each provider in `Registry::scan_order()` (curse first): provider.scan()
  on the remaining unclaimed folders. Enforce: a folder claimed by any provider is removed
  from the request for later providers.
- apply_scan_report: for each ScanMatch → AddonState (preserve existing auto_update/
  channel/ignored/user overrides from store; honor cfg.addons overrides:
  `[addon provider:id]` pins provider/folders/channel/auto_update/ignored). New addons get
  auto_update = cfg.updates.auto_update, channel = cfg.updates.channel.
  Unclaimed folders → AddonState{provider: Zip, external_id: "local:<folder>"} marked
  manual=true, auto_update=false, name from TOC title.
  Folders no longer present on disk (from managed states) → drop from store.

## install.rs — download / extract / apply (backup + rollback)

```rust
pub struct InstallOutcome { pub folders: Vec<String>, pub version: Option<String>, pub interfaces: Vec<u32> }
pub async fn install(ctx, addons_dir, meta: &RemoteMeta, file: &RemoteFile,
    on_progress: &mut dyn FnMut(InstallProgress)) -> Result<InstallOutcome>
pub enum InstallProgress { Downloading { bytes: u64, total: Option<u64> }, Extracting, BackingUp, Swapping, Done }
pub async fn uninstall(ctx, addons_dir, state: &AddonState) -> Result<()>
```
- Download bytes to memory (addon zips are small; cap 200 MiB, provider headers via
  file.headers) with net::download_to; verify md5/sha1 when present (md5: compute with a
  tiny impl — no external crate; sha1: implement ~40 lines in util or vendor a compact
  one, add unit test with known digest vectors).
- Extract with `zip` crate (version 9): `ZipArchive::new(Cursor)`, **enforce header
  consistency / sanitize entry names** (CVE-2026-115905: use `entry.sanitized()` /
  `enforce_header_consistency` APIs available in 9.x — read the local docs.rs source at
  ~/.cargo/registry/src/*/zip-9*/src to use the right API; reject `..` and absolute paths).
- Layout detection: collect top-level entries. If a single top-level dir and it contains
  a .toc → addon root = that dir. Every top-level dir that directly contains a .toc
  matching its name is an addon folder; loose top-level files → root named after the zip.
- Apply atomically: (1) copy each to-be-replaced existing folder to `<AddOns>/../.fa-tmp/backup/<ts>/<name>`;
  (2) remove existing folder dirs; (3) move extracted addon dirs into AddOns;
  (4) on any error restore backup copies and return Err.
- After install: re-parse each installed folder's TOC (installed_version, interfaces).
- uninstall(): remove each state.folders dir (safety: dir name must equal a folder name
  from state, direct child of AddOns, never a path).

## update.rs — check + auto-update queue

```rust
pub struct UpdateInfo { pub key: String, pub name: String, pub current: Option<String>, pub target: RemoteFile, pub version: String }
pub async fn check_updates(ctx, store: &mut StateStore, reg: &Registry, sel: &FileSelect,
    on_progress: &mut dyn FnMut(&str, usize, usize)) -> Result<Vec<UpdateInfo>>
pub struct ApplySummary { pub updated: usize, pub failed: Vec<(String, String)> }
pub async fn apply_updates(ctx, store: &mut StateStore, reg: &Registry, sel: &FileSelect,
    infos: &[UpdateInfo], addons_dir: &Path, on_progress: &mut dyn FnMut(&str, &InstallProgress)) -> Result<ApplySummary>
pub fn auto_update_targets(store: &StateStore, updates: &[UpdateInfo]) -> Vec<UpdateInfo>
```
- check_updates: group store addons by provider → reg.get(kind).meta(ids) (skip manual
  Zip "local:" pseudo addons, ignored=true, offline cfg). Cache RemoteMeta into
  state.cached_remote (+cached_at). Pick file via provider.pick(meta, sel_with_channel).
  Update when: no installed_file_id, OR target.file_id != installed_file_id AND
  version_is_newer(target.version, installed) (also flag when file_id differs and
  versions incomparable — date newer). Set state.pending_update.
- apply_updates: sequential (concurrency 1 is fine, avoids zip storms), calls
  install::install, on success updates state (installed_file_id, installed_version,
  folders from outcome), on failure records and continues. Prune backup dir keeping last
  3 snapshots when !cfg.updates.keep_backups → keep 1.
- auto_update_targets: updates for addons with auto_update && !ignored.
- Scheduler helper: `pub fn seconds_until_next_check(last: Option<i64>, interval_min: u32) -> Duration`.

## watcher (engine/scan.rs bottom)

```rust
pub fn spawn_watcher(addons_dir: PathBuf, on_change: impl Fn() + Send + 'static) -> Result<WatcherHandle>
pub struct WatcherHandle(...) // Drop = stop
```
- notify + notify-debouncer-full on the AddOns dir (Create/Modify/Delete of dirs,
  1.5 s timeout). Callback used by GUI to trigger rescan (debounced).

## Tests

- fixture AddOns dir in temp: 3 fake addons (single-folder, multi-folder bundle with
  Shared Lib, no-TOC junk) → scan_folders returns correct FolderInfos incl. fingerprints.
- install: build a zip in memory (zip writer) with a top-level addon dir + toc →
  install into temp AddOns → folders land, version parsed; second install replaces;
  corrupted zip (truncate) → rollback restores original.
- update: pure logic test of update-needed matrix using stub RemoteMeta.
- unit tests use no network (#[ignore] live tests only, FA_LIVE=1).
