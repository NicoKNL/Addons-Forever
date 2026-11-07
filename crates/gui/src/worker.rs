//! GUI ↔ worker bridge: commands, events, and the tokio runtime loop.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::sync::mpsc;

use addons_forever_core::config::AppConfig;
use addons_forever_core::engine::install::{self, InstallProgress};
use addons_forever_core::engine::scan;
use addons_forever_core::engine::update::{self, UpdateInfo};
use addons_forever_core::providers::Registry;
use addons_forever_core::store::{AddonState, StateStore};
use addons_forever_core::types::{AppCtx, Channel, FileSelect, ProviderKind, SearchItem};

/// Self-update source: a GitHub `owner/repo` (optional, set via `[app] update_repo`).

#[derive(Debug, Clone)]
pub enum Cmd {
    Rescan,
    Check,
    ApplyAll,
    /// Install updates for a single addon.
    ApplyKey(String),
    /// Re-install/update a known addon.
    InstallExisting(String),
    /// Fresh install of a searched addon.
    InstallNew { provider: ProviderKind, external_id: String, name: String },
    Uninstall(String),
    Search { provider: ProviderKind, query: String, page: u32 },
    Changelog { key: String },
    SetFlags { key: String, auto: Option<bool>, ignored: Option<bool>, channel: Option<Channel> },
    /// Turn auto-update on/off for every tracked addon at once.
    SetAutoAll(bool),
    /// Write a portable JSON manifest of the tracked addons.
    ExportAddons(std::path::PathBuf),
    /// Install every addon listed in a portable JSON manifest.
    ImportAddons(std::path::PathBuf),
    /// Manual provider binding ("set source").
    Override { key: String, provider: ProviderKind, external_id: String },
    /// Install by pasting a URL (github repo, curse slug page, mmoui file, zip URL).
    InstallUrl(String),
    SaveConfig(Box<AppConfig>),
    SelfUpdate,
}

#[derive(Debug, Clone)]
pub enum Ev {
    /// Full store snapshot after every mutation (GUI renders from this).
    State { addons: Vec<AddonState> },
    ScanProgress { folder: String, done: usize, total: usize },
    ScanDone { matched: usize, unmatched: usize },
    CheckProgress { name: String, done: usize, total: usize },
    CheckDone { updates: Vec<UpdateInfo> },
    InstallProgress { key: String, name: String, phase: String, bytes: u64, total: Option<u64> },
    InstallDone { key: String, name: String, version: Option<String> },
    Failed { key: String, msg: String },
    SearchDone {
        #[allow(dead_code)] // reserved for UI filtering
        provider: ProviderKind,
        query: String,
        page: u32,
        items: Vec<SearchItem>,
        total: u32,
    },
    ChangelogDone { key: String, title: String, text: String },
    Log { line: String },
    SelfUpdateDone { info: String },
}

pub struct WorkerHandle {
    pub tx: mpsc::UnboundedSender<Cmd>,
    pub rx: mpsc::UnboundedReceiver<Ev>,
}

/// Spawn the worker thread with its own tokio runtime; it owns the StateStore.
pub const EXPORT_FORMAT: &str = "addons-forever-export";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExportEntry {
    pub provider: String,
    pub external_id: String,
    pub name: String,
    pub auto_update: bool,
    pub channel: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ExportManifest {
    pub format: String,
    pub version: u32,
    pub exported_at: String,
    pub addons: Vec<ExportEntry>,
}

pub fn spawn(cfg: Arc<AppConfig>, data_dir: PathBuf, app_version: &'static str) -> WorkerHandle {
    let (cmd_tx, mut cmd_rx) = mpsc::unbounded_channel::<Cmd>();
    let (ev_tx, ev_rx) = mpsc::unbounded_channel::<Ev>();

    std::thread::Builder::new()
        .name("fa-worker".into())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("tokio runtime");
            rt.block_on(async move {
                run(&mut cmd_rx, &ev_tx, cfg, data_dir, app_version).await;
            });
        })
        .expect("spawn worker thread");

    WorkerHandle { tx: cmd_tx, rx: ev_rx }
}

struct Worker {
    ev: mpsc::UnboundedSender<Ev>,
    cfg: Arc<AppConfig>,
    ctx: AppCtx,
    data_dir: PathBuf,
    store: StateStore,
    _watcher: Option<scan::WatcherHandle>,
    watcher_tx: mpsc::UnboundedSender<()>,
    watcher_rx: mpsc::UnboundedReceiver<()>,
}

impl Worker {
    fn log(&self, line: impl AsRef<str>) {
        let ts = chrono_now();
        let _ = self.ev.send(Ev::Log { line: format!("[{ts}] {}", line.as_ref()) });
    }

    fn addons_dir(&self) -> Option<PathBuf> {
        self.cfg.addons_dir()
    }

    fn sel(&self, channel: Channel) -> FileSelect {
        FileSelect::from_cfg_channel(&self.cfg, channel)
    }

    /// Store is in-memory; user intent (auto/channel/ignored pins) goes to the
    /// ini's `[addon ...]` sections. Nothing else is written to disk.
    fn save_state(&mut self) {
        let mut c = (*self.cfg).clone();
        if self.store.write_overrides(&mut c) {
            let path = addons_forever_core::config::AppConfig::config_path(&self.data_dir);
            if let Err(e) = c.save(&path) {
                self.log(format!("config save failed: {e}"));
            } else {
                self.cfg = Arc::new(c);
            }
        }
        let addons: Vec<AddonState> = self.store.addons.values().cloned().collect();
        let _ = self.ev.send(Ev::State { addons });
    }

    fn emit_state(&self) {
        let addons: Vec<AddonState> = self.store.addons.values().cloned().collect();
        let _ = self.ev.send(Ev::State { addons });
    }

    fn setup_watcher(&mut self) {
        self._watcher = None;
        let Some(dir) = self.addons_dir() else { return };
        match scan::spawn_watcher(dir.clone(), {
            let tx = self.watcher_tx.clone();
            move || {
                let _ = tx.send(());
            }
        }) {
            Ok(h) => {
                self._watcher = Some(h);
                self.log(format!("watching {}", dir.display()));
            }
            Err(e) => self.log(format!("watcher unavailable: {e}")),
        }
    }

    async fn do_scan(&mut self) {
        let Some(dir) = self.addons_dir() else {
            self.log("scan: no AddOns directory configured");
            return;
        };
        let ev = self.ev.clone();
        let sel = self.sel(self.cfg.updates.channel);
        self.log(format!("scanning {}", dir.display()));
        match scan::scan_addons(&self.ctx, &dir, &sel, &mut |folder, done, total| {
            let _ = ev.send(Ev::ScanProgress { folder: folder.to_string(), done, total });
        })
        .await
        {
            Ok(report) => {
                let matched = report.matched.len();
                let unmatched = report.unmatched.len();
                scan::apply_scan_report(&mut self.store, &report, &self.cfg);
                self.save_state();
                self.log(format!("scan done: {matched} matched, {unmatched} unmanaged"));
                let _ = self.ev.send(Ev::ScanDone { matched, unmatched });
            }
            Err(e) => {
                self.log(format!("scan failed: {e}"));
                let _ = self.ev.send(Ev::Failed { key: String::new(), msg: format!("Scan: {e}") });
            }
        }
    }

    async fn do_check(&mut self) -> Vec<UpdateInfo> {
        let reg = Registry::new(&self.ctx);
        let sel = self.sel(self.cfg.updates.channel);
        let ev = self.ev.clone();
        let updates = match update::check_updates(
            &self.ctx,
            &mut self.store,
            &reg,
            &sel,
            &mut |name, done, total| {
                let _ = ev.send(Ev::CheckProgress { name: name.to_string(), done, total });
            },
        )
        .await
        {
            Ok(u) => u,
            Err(e) => {
                self.log(format!("check failed: {e}"));
                let _ = self.ev.send(Ev::Failed { key: String::new(), msg: format!("Check: {e}") });
                return vec![];
            }
        };
        self.save_state();
        if !updates.is_empty() {
            self.log(format!("{} update(s) available", updates.len()));
        }
        let _ = self.ev.send(Ev::CheckDone { updates: updates.clone() });
        updates
    }

    async fn do_apply(&mut self, infos: Vec<UpdateInfo>) {
        if infos.is_empty() {
            self.log("nothing to update");
            return;
        }
        let Some(dir) = self.addons_dir() else {
            self.log("apply: no AddOns directory configured");
            return;
        };
        let reg = Registry::new(&self.ctx);
        let sel = self.sel(self.cfg.updates.channel);
        let ev = self.ev.clone();
        let names: std::collections::HashMap<String, String> =
            self.store.addons.iter().map(|(k, a)| (k.clone(), a.name.clone())).collect();
        self.log(format!("updating {} addon(s)", infos.len()));
        let summary = update::apply_updates(
            &self.ctx,
            &mut self.store,
            &reg,
            &sel,
            &infos,
            &dir,
            &mut |key, prog| {
                let name = names.get(key).cloned().unwrap_or_default();
                let (phase, bytes, total) = match prog {
                    InstallProgress::Downloading { bytes, total } => ("download", *bytes, *total),
                    other => (phase_str(other), 0, None),
                };
                let _ = ev.send(Ev::InstallProgress {
                    key: key.to_string(),
                    name,
                    phase: phase.into(),
                    bytes,
                    total,
                });
            },
        )
        .await;
        match summary {
            Ok(s) => {
                for (key, version) in &s.updated_keys {
                    let name = names.get(key).cloned().unwrap_or_default();
                    let _ = ev.send(Ev::InstallDone {
                        key: key.clone(),
                        name,
                        version: version.clone(),
                    });
                }
                for (key, msg) in &s.failed {
                    let _ = ev.send(Ev::Failed { key: key.clone(), msg: msg.clone() });
                }
                self.save_state();
                self.log(format!("apply done: {} updated, {} failed", s.updated, s.failed.len()));
            }
            Err(e) => {
                self.log(format!("apply failed: {e}"));
                let _ = ev.send(Ev::Failed { key: String::new(), msg: format!("Apply: {e}") });
            }
        }
    }

    async fn do_install_existing(&mut self, key: String) {
        let Some(st) = self.store.addons.get(&key).cloned() else { return };
        let reg = Registry::new(&self.ctx);
        let Some(dir) = self.addons_dir() else { return };
        let ev = self.ev.clone();
        let name = st.name.clone();
        let sel = self.sel(st.channel);
        let provider = reg.get(st.provider);
        let res: Result<(), String> = async {
            let metas = provider.meta(&self.ctx, std::slice::from_ref(&st.external_id)).await.map_err(|e| e.to_string())?;
            let meta = metas.into_iter().next().ok_or("provider returned no data")?;
            let file = provider
                .pick(&meta, &sel)
                .ok_or("no compatible file for this client/channel")?;
            let mut prog = {
                let ev = ev.clone();
                let key = key.clone();
                let name = name.clone();
                move |p: InstallProgress| {
                    let (phase, bytes, total) = match p {
                        InstallProgress::Downloading { bytes, total } => ("download", bytes, total),
                        other => (phase_str(&other), 0, None),
                    };
                    let _ = ev.send(Ev::InstallProgress {
                        key: key.clone(),
                        name: name.clone(),
                        phase: phase.into(),
                        bytes,
                        total,
                    });
                }
            };
            let outcome = install::install(&self.ctx, &dir, &meta, &file, &mut prog)
                .await
                .map_err(|e| e.to_string())?;
            if let Some(st) = self.store.addons.get_mut(&key) {
                st.installed_file_id = Some(file.file_id.clone());
                st.installed_version = outcome.version.clone();
                st.folders = outcome.folders.clone();
                st.size_bytes = outcome.size_bytes;
                st.pending_update = None;
                st.cached_remote = Some(meta.clone());
            }
            let _ = ev.send(Ev::InstallDone { key: key.clone(), name: name.clone(), version: outcome.version });
            Ok(())
        }
        .await;
        if let Err(msg) = res {
            self.log(format!("install {name} failed: {msg}"));
            let _ = ev.send(Ev::Failed { key, msg });
        }
        self.save_state();
    }

    async fn do_install_new(&mut self, provider: ProviderKind, external_id: String, name: String) {
        let reg = Registry::new(&self.ctx);
        let Some(dir) = self.addons_dir() else {
            self.log("install: no AddOns directory configured");
            return;
        };
        let ev = self.ev.clone();
        let key = AddonState::key(provider, &external_id);
        let sel = self.sel(self.cfg.updates.channel);
        let prov = reg.get(provider);
        let res: Result<(), String> = async {
            let metas = prov.meta(&self.ctx, std::slice::from_ref(&external_id)).await.map_err(|e| e.to_string())?;
            let meta = metas.into_iter().next().ok_or("provider returned no data")?;
            let file = prov.pick(&meta, &sel).ok_or("no compatible file for this client/channel")?;
            let mut prog = {
                let ev = ev.clone();
                let key = key.clone();
                let name = name.clone();
                move |p: InstallProgress| {
                    let (phase, bytes, total) = match p {
                        InstallProgress::Downloading { bytes, total } => ("download", bytes, total),
                        other => (phase_str(&other), 0, None),
                    };
                    let _ = ev.send(Ev::InstallProgress {
                        key: key.clone(),
                        name: name.clone(),
                        phase: phase.into(),
                        bytes,
                        total,
                    });
                }
            };
            let outcome = install::install(&self.ctx, &dir, &meta, &file, &mut prog)
                .await
                .map_err(|e| e.to_string())?;
            let mut st = AddonState {
                key: key.clone(),
                provider,
                external_id: external_id.clone(),
                name: meta.name.clone(),
                site_url: meta.site_url.clone(),
                repo_url: meta.repo_url.clone(),
                icon_url: meta.icon_url.clone(),
                folders: outcome.folders.clone(),
                fingerprints: vec![],
                installed_version: outcome.version.clone(),
                installed_file_id: Some(file.file_id.clone()),
                size_bytes: outcome.size_bytes,
                channel: self.cfg.updates.channel,
                auto_update: self.cfg.updates.auto_update,
                ignored: false,
                manual: provider == ProviderKind::Zip,
                cached_remote: Some(meta.clone()),
                cached_at: Some(now_unix()),
                last_checked: Some(now_unix()),
                provider_data: serde_json::Value::Null,
                pending_update: None,
            };
            if st.name.is_empty() {
                st.name = name.clone();
            }
            self.store.upsert(st);
            let _ = ev.send(Ev::InstallDone { key: key.clone(), name: name.clone(), version: outcome.version });
            Ok(())
        }
        .await;
        if let Err(msg) = res {
            self.log(format!("install {name} failed: {msg}"));
            let _ = ev.send(Ev::Failed { key, msg });
        }
        self.save_state();
    }

    async fn do_uninstall(&mut self, key: String) {
        let Some(st) = self.store.addons.get(&key).cloned() else { return };
        let Some(dir) = self.addons_dir() else { return };
        match install::uninstall(&self.ctx, &dir, &st).await {
            Ok(()) => {
                self.log(format!("removed {}", st.name));
                self.store.addons.remove(&key);
                self.save_state();
            }
            Err(e) => {
                let _ = self.ev.send(Ev::Failed { key, msg: format!("Uninstall: {e}") });
            }
        }
    }

    fn do_export(&mut self, path: std::path::PathBuf) {
        let mut addons: Vec<ExportEntry> = self
            .store
            .addons
            .values()
            .map(|a| ExportEntry {
                provider: a.provider.as_str().to_string(),
                external_id: a.external_id.clone(),
                name: a.name.clone(),
                auto_update: a.auto_update,
                channel: a.channel.as_str().to_string(),
            })
            .collect();
        addons.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        let manifest = ExportManifest {
            format: EXPORT_FORMAT.to_string(),
            version: 1,
            exported_at: chrono::Local::now().to_rfc3339(),
            addons,
        };
        match serde_json::to_string_pretty(&manifest) {
            Ok(json) => {
                let n = manifest.addons.len();
                match std::fs::write(&path, json) {
                    Ok(()) => self.log(format!("exported {n} addon(s) to {}", path.display())),
                    Err(e) => self.log(format!("export failed: {e}")),
                }
            }
            Err(e) => self.log(format!("export failed: {e}")),
        }
    }

    async fn do_import(&mut self, path: std::path::PathBuf) {
        let raw = match std::fs::read_to_string(&path) {
            Ok(r) => r,
            Err(e) => {
                self.log(format!("import failed: {e}"));
                return;
            }
        };
        let manifest: ExportManifest = match serde_json::from_str::<ExportManifest>(&raw) {
            Ok(m) if m.format == EXPORT_FORMAT => m,
            Ok(_) => {
                self.log("import failed: not an addons-forever export file");
                return;
            }
            Err(e) => {
                self.log(format!("import failed: {e}"));
                return;
            }
        };
        let n = manifest.addons.len();
        self.log(format!("importing {n} addon(s) from {}", path.display()));
        let mut ok = 0usize;
        for e in manifest.addons {
            let Some(provider) = ProviderKind::from_key(&e.provider) else {
                self.log(format!("import skip `{}`: unknown provider `{}`", e.name, e.provider));
                continue;
            };
            self.do_install_new(provider, e.external_id.clone(), e.name.clone()).await;
            let key = addons_forever_core::store::AddonState::key(provider, &e.external_id);
            if let Some(st) = self.store.addons.get_mut(&key) {
                st.auto_update = e.auto_update;
                if let Some(ch) = Channel::from_key(&e.channel) {
                    st.channel = ch;
                }
                ok += 1;
            }
        }
        self.save_state();
        self.log(format!("import done: {ok}/{n} addon(s) in your set"));
    }

    /// Search fetches ALL result pages (bounded) so the UI can sort the complete
    /// result list, not just one server page.
    async fn do_search(&mut self, provider: ProviderKind, query: String, _page: u32) {
        const PER: usize = 50;
        const CAP: usize = 500;
        let reg = Registry::new(&self.ctx);
        let sel = self.sel(self.cfg.updates.channel);
        let mut all: Vec<SearchItem> = Vec::new();
        let mut total = 0usize;
        let mut page = 0u32;
        let mut err = None;
        loop {
            match reg.get(provider).search(&self.ctx, &query, page, PER as u32, &sel).await {
                Ok(p) => {
                    total = p.total as usize;
                    if p.items.is_empty() {
                        break;
                    }
                    let n = p.items.len();
                    all.extend(p.items);
                    if all.len() >= total.min(CAP) || n < PER {
                        break;
                    }
                    page += 1;
                }
                Err(e) => {
                    err = Some(e.to_string());
                    break;
                }
            }
        }
        if let Some(e) = err {
            if all.is_empty() {
                let _ = self
                    .ev
                    .send(Ev::Failed { key: "search".into(), msg: format!("{} search: {e}", provider.label()) });
                return;
            }
        }
        let _ = self.ev.send(Ev::SearchDone { provider, query, page: 0, items: all, total: total as u32 });
    }

    async fn do_changelog(&mut self, key: String) {
        let Some(st) = self.store.addons.get(&key).cloned() else { return };
        let reg = Registry::new(&self.ctx);
        let prov = reg.get(st.provider);
        let meta = match st.cached_remote.clone() {
            Some(m) => m,
            None => {
                let ev = self.ev.clone();
                let _ = ev.send(Ev::ChangelogDone { key, title: st.name, text: "No cached metadata. Update first.".into() });
                return;
            }
        };
        let sel = self.sel(st.channel);
        let title = st.name.clone();
        let file = prov.pick(&meta, &sel).or_else(|| meta.files.first().cloned());
        let text = match file {
            Some(f) => match prov.changelog(&self.ctx, &meta, &f).await {
                Ok(Some(t)) => t,
                Ok(None) => "No changelog provided by the source.".into(),
                Err(e) => format!("Failed: {e}"),
            },
            None => "No file available.".into(),
        };
        let _ = self.ev.send(Ev::ChangelogDone { key, title, text });
    }

    fn do_save_config(&mut self, new_cfg: AppConfig, path: &Path) {
        if let Err(e) = new_cfg.save(path) {
            let _ = self.ev.send(Ev::Failed { key: "config".into(), msg: format!("Save: {e}") });
            return;
        }
        self.cfg = Arc::new(new_cfg);
        self.ctx = AppCtx::new(self.cfg.clone(), self.ctx.app_version);
        self.setup_watcher();
        self.log("settings saved");
        self.emit_state();
    }

    fn do_self_update(&mut self) {
        let ev = self.ev.clone();
        let Some(repo) = self
            .cfg
            .app
            .update_repo
            .clone()
            .map(|r| r.trim().to_string())
            .filter(|r| !r.is_empty())
        else {
            let _ = ev.send(Ev::SelfUpdateDone {
                info: "app self-update is not configured. Set `update_repo = owner/repo` under [app] in the .ini (GitHub Releases)".into(),
            });
            return;
        };
        tokio::spawn(async move {
            let out = tokio::task::spawn_blocking(move || crate::selfupdate::run(&repo)).await;
            let info = match out {
                Ok(Ok(msg)) => msg,
                Ok(Err(e)) => format!("update failed: {e}"),
                Err(e) => format!("update job panicked: {e}"),
            };
            let _ = ev.send(Ev::SelfUpdateDone { info });
        });
    }
}

fn phase_str(p: &InstallProgress) -> &'static str {
    match p {
        InstallProgress::Extracting => "extract",
        InstallProgress::BackingUp => "backup",
        InstallProgress::Swapping => "install",
        InstallProgress::Done => "done",
        InstallProgress::Downloading { .. } => "download",
    }
}

async fn run(
    cmd_rx: &mut mpsc::UnboundedReceiver<Cmd>,
    ev: &mpsc::UnboundedSender<Ev>,
    cfg: Arc<AppConfig>,
    data_dir: PathBuf,
    app_version: &'static str,
) {
    // Fold legacy state.json/egui-ui.toml/logs into the ini, then delete them.
    let mut cfg = (*cfg).clone();
    if StateStore::migrate_into_cfg(&data_dir, &mut cfg) {
        let _ = cfg.save(&addons_forever_core::config::AppConfig::config_path(&data_dir));
    }
    let cfg = Arc::new(cfg);
    let store = StateStore::default();
    let (watcher_tx, watcher_rx) = mpsc::unbounded_channel();
    let mut w = Worker {
        ev: ev.clone(),
        cfg: cfg.clone(),
        ctx: AppCtx::new(cfg.clone(), app_version),
        data_dir: data_dir.clone(),
        store,
        _watcher: None,
        watcher_tx,
        watcher_rx,
    };
    w.setup_watcher();
    w.emit_state();

    // startup jobs
    let cfg0 = w.cfg.clone();
    if cfg0.app.addons_dir.is_some() || cfg0.app.auto_detect {
        w.do_scan().await;
        if cfg0.updates.check_on_startup {
            let updates = w.do_check().await;
            if cfg0.updates.auto_update {
                let targets = update::auto_update_targets(&w.store, &updates);
                if !targets.is_empty() {
                    w.log(format!("auto-update: {} addon(s)", targets.len()));
                    w.do_apply(targets).await;
                }
            }
        }
    } else {
        w.log("AddOns folder not configured. Open Settings.");
    }

    let mut last_check: i64 = 0;
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            cmd = cmd_rx.recv() => {
                let Some(cmd) = cmd else { break };
                match cmd {
                    Cmd::Rescan => w.do_scan().await,
                    Cmd::Check => { w.do_check().await; last_check = now_unix(); }
                    Cmd::ApplyAll => {
                        let updates = w.do_check().await;
                        w.do_apply(updates).await;
                    }
                    Cmd::ApplyKey(key) => {
                        let info = w.store.addons.get(&key).and_then(|st| {
                            let p = st.pending_update.clone()?;
                            let target = st.cached_remote.as_ref()?.files.iter().find(|f| f.file_id == p.file_id)?.clone();
                            Some(UpdateInfo {
                                key: key.clone(),
                                name: st.name.clone(),
                                current: st.installed_version.clone(),
                                target,
                                version: p.version,
                            })
                        });
                        match info {
                            Some(i) => w.do_apply(vec![i]).await,
                            None => w.do_install_existing(key).await,
                        }
                    }
                    Cmd::InstallExisting(key) => w.do_install_existing(key).await,
                    Cmd::InstallNew { provider, external_id, name } => {
                        w.do_install_new(provider, external_id, name).await
                    }
                    Cmd::InstallUrl(url) => match parse_install_url(&w.ctx, &url).await {
                        Ok((provider, external_id, name)) => {
                            w.do_install_new(provider, external_id, name).await
                        }
                        Err(msg) => {
                            let _ = w.ev.send(Ev::Failed { key: "install-url".into(), msg: msg.clone() });
                            w.log(format!("bad install URL {url}: {msg}"));
                        }
                    },
                    Cmd::Uninstall(key) => w.do_uninstall(key).await,
                    Cmd::Search { provider, query, page } => w.do_search(provider, query, page).await,
                    Cmd::Changelog { key } => w.do_changelog(key).await,
                    Cmd::ExportAddons(p) => w.do_export(p),
                    Cmd::ImportAddons(p) => w.do_import(p).await,
                    Cmd::SetAutoAll(on) => {
                        for a in w.store.addons.values_mut() {
                            a.auto_update = on;
                        }
                        w.save_state();
                        w.log(format!("auto-update {} for all addons", if on { "enabled" } else { "disabled" }));
                    }
                    Cmd::SetFlags { key, auto, ignored, channel } => {
                        if let Some(st) = w.store.addons.get_mut(&key) {
                            if let Some(a) = auto { st.auto_update = a; }
                            if let Some(i) = ignored { st.ignored = i; }
                            if let Some(c) = channel { st.channel = c; }
                        }
                        w.save_state();
                    }
                    Cmd::Override { key, provider, external_id } => {
                        if let Some(mut st) = w.store.addons.remove(&key) {
                            st.provider = provider;
                            st.external_id = external_id.clone();
                            st.key = AddonState::key(provider, &external_id);
                            st.manual = true;
                            st.cached_remote = None;
                            w.store.upsert(st);
                            w.save_state();
                            w.log(format!("source set to {}/{}", provider.label(), external_id));
                        }
                    }
                    Cmd::SaveConfig(new_cfg) => {
                        let path = AppConfig::config_path(&w.data_dir);
                        w.do_save_config(*new_cfg, &path);
                    }
                    Cmd::SelfUpdate => w.do_self_update(),
                }
            }
            _ = w.watcher_rx.recv() => {
                // debounce bursts (the debouncer already does 1.5s)
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                while w.watcher_rx.try_recv().is_ok() {}
                w.log("AddOns folder changed, rescanning");
                w.do_scan().await;
            }
            _ = tick.tick() => {
                let c = w.cfg.clone();
                if c.updates.auto_update
                    && !c.network.offline
                    && last_check > 0
                    && now_unix() - last_check >= (c.updates.interval_minutes as i64) * 60
                {
                    w.log("scheduled update check");
                    let updates = w.do_check().await;
                    let targets = update::auto_update_targets(&w.store, &updates);
                    w.do_apply(targets).await;
                    last_check = now_unix();
                }
            }
        }
    }
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Parse a user-pasted install target into (provider, external_id, name).
pub(crate) async fn parse_install_url(ctx: &AppCtx, raw: &str) -> Result<(ProviderKind, String, String), String> {
    let s = raw
        .trim()
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .trim_start_matches("www.");

    // github.com/owner/repo[/...]
    if let Some(rest) = s.strip_prefix("github.com/") {
        let segs: Vec<&str> = rest.split('/').collect();
        if segs.len() >= 2 && !segs[0].is_empty() && !segs[1].is_empty() {
            let owner = segs[0].trim_end_matches(".git");
            let repo = segs[1];
            return Ok((ProviderKind::GitHub, format!("{owner}/{repo}"), repo.to_string()));
        }
        return Err("expected github.com/owner/repo".into());
    }

    // curseforge.com/wow/addons/<slug>
    if let Some(rest) = s
        .strip_prefix("curseforge.com/wow/addons/")
        .or(s.strip_prefix("www.curseforge.com/wow/addons/"))
    {
        let slug = rest.split(['/', '?', '#']).next().unwrap_or("");
        if slug.is_empty() {
            return Err("no addon slug in CurseForge URL".into());
        }
        let id = addons_forever_core::providers::curse::resolve_slug(ctx, slug)
            .await
            .map_err(|e| e.to_string())?
            .ok_or("mod slug not found on CurseForge")?;
        return Ok((ProviderKind::Curse, id, slug.to_string()));
    }

    // wowinterface.com/downloads/info<id>-Name.html
    if let Some(pos) = s.find("/downloads/info") {
        let digits: String = s[pos + "/downloads/info".len()..]
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() {
            return Ok((ProviderKind::WowInterface, digits.clone(), digits));
        }
    }

    // plain https zip URL
    let lower = raw.to_ascii_lowercase();
    if (raw.starts_with("https://") || raw.starts_with("http://"))
        && lower.split('?').next().unwrap_or("").ends_with(".zip")
    {
        let file = raw.split('?').next().unwrap_or("addon.zip");
        let name = file.rsplit('/').next().unwrap_or("addon").trim_end_matches(".zip").to_string();
        return Ok((ProviderKind::Zip, raw.trim().to_string(), name));
    }

    // bare owner/repo
    if let Some((o, r)) = s.split_once('/') {
        let ok = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.');
        if !o.is_empty() && !r.is_empty() && o.chars().all(ok) && r.chars().all(ok) {
            return Ok((ProviderKind::GitHub, format!("{o}/{r}"), r.to_string()));
        }
    }

    // bare numeric CurseForge mod id
    if !s.is_empty() && s.chars().all(|c| c.is_ascii_digit()) {
        return Ok((ProviderKind::Curse, s.to_string(), s.to_string()));
    }

    // bare Tukui slug (elvui, tukui, ...)
    if s.len() >= 3
        && s.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        return Ok((ProviderKind::Tukui, s.to_string(), s.to_string()));
    }

    Err("paste a github.com/owner/repo, CurseForge addon page, WoWInterface file URL, Tukui slug, numeric CurseForge id, or a direct .zip URL".into())
}

fn chrono_now() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

/// CLI entry point into the GUI's URL parser.
pub async fn parse_install_url_pub(ctx: &AppCtx, raw: &str) -> Result<(ProviderKind, String, String), String> {
    parse_install_url(ctx, raw).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parses_install_targets_offline() {
        // github repo URLs & bare owner/repo
        let ctx = test_ctx();
        let (p, id, _) = parse_install_url(&ctx, "https://github.com/BigWigsMods/BigWigs/releases").await.unwrap();
        assert_eq!((p, id.as_str()), (ProviderKind::GitHub, "BigWigsMods/BigWigs"));
        let (p, id, _) = parse_install_url(&ctx, "johnnystarr/some-addon").await.unwrap();
        assert_eq!((p, id.as_str()), (ProviderKind::GitHub, "johnnystarr/some-addon"));
        // direct zip
        let (p, id, _) =
            parse_install_url(&ctx, "https://cdn.example.org/files/my-addon-1.2.3.zip?v=2").await.unwrap();
        assert_eq!((p, id.as_str()), (ProviderKind::Zip, "https://cdn.example.org/files/my-addon-1.2.3.zip?v=2"));
        // numeric curse id, bare tukui slug, wowinterface URL
        let (p, id, _) = parse_install_url(&ctx, "2382").await.unwrap();
        assert_eq!((p, id.as_str()), (ProviderKind::Curse, "2382"));
        let (p, id, _) = parse_install_url(&ctx, "elvui").await.unwrap();
        assert_eq!((p, id.as_str()), (ProviderKind::Tukui, "elvui"));
        let (p, id, _) =
            parse_install_url(&ctx, "https://www.wowinterface.com/downloads/info19508-Foo.html").await.unwrap();
        assert_eq!((p, id.as_str()), (ProviderKind::WowInterface, "19508"));
        // nonsense rejected
        assert!(parse_install_url(&ctx, "?!??").await.is_err());
    }

    fn test_ctx() -> AppCtx {
        AppCtx::new(std::sync::Arc::new(AppConfig::default()), "test")
    }
}
