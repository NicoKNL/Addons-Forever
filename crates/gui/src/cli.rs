//! Headless CLI: `addons-forever cli <command>`.
//!
//! Same engine as the GUI — used for automation, testing, and SSH sessions.
//! Run `addons-forever cli help` for the command list.

use std::sync::Arc;

use addons_forever_core::config::AppConfig;
use addons_forever_core::providers::Registry;
use addons_forever_core::store::{AddonState, StateStore};

/// Write the store's user-intent flags into the ini's `[addon ...]` sections.
fn persist_flags(data_dir: &std::path::Path, store: &StateStore) -> Result<(), String> {
    let path = AppConfig::config_path(data_dir);
    let mut cfg = AppConfig::load(&path).map_err(|e| e.to_string())?;
    if store.write_overrides(&mut cfg) {
        cfg.save(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
use addons_forever_core::types::{AppCtx, FileSelect, ProviderKind};

pub fn run(args: &[String]) -> i32 {
    attach_console();
    let data_dir = match addons_forever_core::util::data_dir() {
        Ok(d) => d,
        Err(e) => return fail(&format!("cannot resolve app data dir: {e}")),
    };
    let config_path = AppConfig::config_path(&data_dir);
    let mut cfg = match AppConfig::load(&config_path) {
        Ok(c) => c,
        Err(e) => return fail(&format!("config error: {e}")),
    };
    StateStore::migrate_into_cfg(&data_dir, &mut cfg);
    let cfg = Arc::new(cfg);

    let rt = match tokio::runtime::Builder::new_multi_thread().enable_all().build() {
        Ok(rt) => rt,
        Err(e) => return fail(&format!("runtime: {e}")),
    };
    let res = rt.block_on(dispatch(&data_dir, &config_path, cfg, args));
    match res {
        Ok(()) => 0,
        Err(e) => fail(&e),
    }
}

async fn dispatch(data_dir: &std::path::Path, config_path: &std::path::Path, cfg: Arc<AppConfig>, args: &[String]) -> Result<(), String> {
    let cmd = args.first().map(|s| s.as_str()).unwrap_or("help");
    let ctx = AppCtx::new(cfg.clone(), env!("CARGO_PKG_VERSION"));
    let reg = Registry::new(&ctx);
    let mut store = StateStore::default();
    let sel = FileSelect::from_cfg(&cfg);
    let needs_dir = matches!(cmd, "scan" | "check" | "update" | "install" | "uninstall" | "fp" | "import");
    let addons_dir = if needs_dir {
        cfg.addons_dir().ok_or("AddOns folder not configured. Run `addons-forever cli set-path <dir>`")?
    } else {
        Default::default()
    };
    use std::path::PathBuf as PB;
    let _: PB = Default::default();

    match cmd {
        "scan" => {
            let report = addons_forever_core::engine::scan::scan_addons(&ctx, &addons_dir, &sel, &mut |f, d, t| {
                if d % 25 == 0 || d == t {
                    eprintln!("scanning {d}/{t} ({f})");
                }
            })
            .await
            .map_err(|e| e.to_string())?;
            addons_forever_core::engine::scan::apply_scan_report(&mut store, &report, &cfg);
            persist_flags(data_dir, &store)?;
            for m in &report.matched {
                println!(
                    "MATCH  {:<28} {:<10} {:<14} {}",
                    m.meta.name,
                    m.provider.label(),
                    m.external_id,
                    m.folders.join(", ")
                );
            }
            for f in &report.unmatched {
                println!("LOCAL  {:<28} (no known source)", f.name);
            }
            println!("{} tracked addons", store.addons.len());
        }
        "list" => {
            for a in store.addons.values() {
                println!(
                    "{:<28} {:<10} {:<14} v{:<10} {:>8} auto:{} chan:{}",
                    truncate(&a.name, 28),
                    a.provider.label(),
                    a.external_id,
                    a.installed_version.clone().unwrap_or_else(|| "?".into()),
                    crate::tabs::fmt_size(a.size_bytes),
                    a.auto_update,
                    a.channel.as_str()
                );
            }
        }
        "check" => {
            let updates = addons_forever_core::engine::update::check_updates(&ctx, &mut store, &reg, &sel, &mut |_, _, _| {})
                .await
                .map_err(|e| e.to_string())?;
            persist_flags(data_dir, &store)?;
            for u in &updates {
                println!("UPDATE {:<28} {} -> {}", truncate(&u.name, 28), u.current.clone().unwrap_or_else(|| "-".into()), u.version);
            }
            println!("{} update(s)", updates.len());
        }
        "update" => {
            let mut updates = addons_forever_core::engine::update::check_updates(&ctx, &mut store, &reg, &sel, &mut |_, _, _| {})
                .await
                .map_err(|e| e.to_string())?;
            if args.len() > 1 {
                let f = args[1].to_lowercase();
                updates.retain(|u| u.name.to_lowercase().contains(&f) || u.key.to_lowercase().contains(&f));
            }
            if updates.is_empty() {
                println!("all addons up to date");
                return Ok(());
            }
            let summary = addons_forever_core::engine::update::apply_updates(
                &ctx,
                &mut store,
                &reg,
                &sel,
                &updates,
                &addons_dir,
                &mut |key, prog| {
                    if let addons_forever_core::engine::install::InstallProgress::Downloading { bytes, total } = prog {
                        eprint!("\rdownloading {key}: {bytes}/{}    ", total.map(|t| t.to_string()).unwrap_or("?".into()));
                    }
                },
            )
            .await
            .map_err(|e| e.to_string())?;
            eprintln!();
            persist_flags(data_dir, &store)?;
            println!("updated {}", summary.updated);
            for (k, msg) in summary.failed {
                println!("FAILED {k}: {msg}");
            }
        }
        "install" => {
            let url = args.get(1).ok_or("usage: cli install <url|owner/repo|id|slug>")?;
            let (provider, external_id, name) = parse_target(&ctx, url).await?;
            let prov = reg.get(provider);
            let metas = prov.meta(&ctx, std::slice::from_ref(&external_id)).await.map_err(|e| e.to_string())?;
            let meta = metas.into_iter().next().ok_or("source returned no data")?;
            let file = prov.pick(&meta, &sel).ok_or("no compatible file")?;
            let outcome = addons_forever_core::engine::install::install(
                &ctx,
                &addons_dir,
                &meta,
                &file,
                &mut |p| {
                    if let addons_forever_core::engine::install::InstallProgress::Downloading { bytes, total } = p {
                        eprint!("\r{bytes}/{} bytes", total.unwrap_or(0));
                    }
                },
            )
            .await
            .map_err(|e| e.to_string())?;
            eprintln!();
            let key = AddonState::key(provider, &external_id);
            let mut st = store.addons.remove(&key).unwrap_or_else(|| AddonState {
                key: key.clone(),
                provider,
                external_id: external_id.clone(),
                name: meta.name.clone(),
                site_url: meta.site_url.clone(),
                repo_url: meta.repo_url.clone(),
                icon_url: meta.icon_url.clone(),
                folders: vec![],
                fingerprints: vec![],
                installed_version: None,
                installed_file_id: None,
                size_bytes: 0,
                channel: cfg.updates.channel,
                auto_update: cfg.updates.auto_update,
                ignored: false,
                manual: provider == ProviderKind::Zip,
                cached_remote: None,
                cached_at: None,
                last_checked: None,
                provider_data: serde_json::Value::Null,
                pending_update: None,
            });
            st.installed_version = outcome.version.clone();
            st.installed_file_id = Some(file.file_id);
            st.folders = outcome.folders.clone();
            st.size_bytes = outcome.size_bytes;
            st.cached_remote = Some(meta);
            store.upsert(st);
            persist_flags(data_dir, &store)?;
            println!("installed {} v{} [{}]", name, outcome.version.unwrap_or_default(), outcome.folders.join(", "));
        }
        "uninstall" => {
            let needle = args.get(1).ok_or("usage: cli uninstall <name|key>")?.to_lowercase();
            let key = store
                .addons
                .iter()
                .find(|(k, a)| k.to_lowercase().contains(&needle) || a.name.to_lowercase().contains(&needle))
                .map(|(k, _)| k.clone())
                .ok_or("addon not found")?;
            let st = store.addons.get(&key).cloned().unwrap();
            addons_forever_core::engine::install::uninstall(&ctx, &addons_dir, &st)
                .await
                .map_err(|e| e.to_string())?;
            store.addons.remove(&key);
            persist_flags(data_dir, &store)?;
            println!("removed {}", st.name);
        }
        "set-path" => {
            let dir = args.get(1).ok_or("usage: cli set-path <AddOns dir>")?;
            let mut c = (*cfg).clone();
            c.app.addons_dir = Some(dir.clone());
            c.save(config_path).map_err(|e| e.to_string())?;
            println!("AddOns folder = {dir}");
        }
        "set-flavor" => {
            let f = args.get(1).and_then(|s| addons_forever_core::flavor::Flavor::parse(s)).ok_or("unknown flavor (forever|retail|classic|...)")?;
            let mut c = (*cfg).clone();
            c.app.flavor = f;
            c.save(config_path).map_err(|e| e.to_string())?;
            println!("flavor = {}", f.label());
        }
        "set-version" => {
            let mut c = (*cfg).clone();
            c.app.game_version = args.get(1).cloned();
            c.save(config_path).map_err(|e| e.to_string())?;
            println!("client version = {}", c.app.game_version.unwrap_or_default());
        }
        "fp" => {
            for e in std::fs::read_dir(&addons_dir).map_err(|e| e.to_string())? {
                let path = e.map_err(|e| e.to_string())?.path();
                if !path.is_dir() {
                    continue;
                }
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                match addons_forever_core::fingerprint::folder_fingerprint(&path) {
                    Ok(Some(fp)) => println!("{name}\t{fp}"),
                    Ok(None) => println!("{name}\t-"),
                    Err(e) => println!("{name}\terror: {e}"),
                }
            }
        }
        "offline" => {
            let mut c = (*cfg).clone();
            c.network.offline = args.get(1).map(|s| s != "off").unwrap_or(true);
            c.save(config_path).map_err(|e| e.to_string())?;
            println!("offline = {}", c.network.offline);
        }
        "selfupdate" => {
            let repo = cfg
                .app
                .update_repo
                .clone()
                .map(|r| r.trim().to_string())
                .filter(|r| !r.is_empty())
                .ok_or("set `update_repo = owner/repo` under [app] in addons-forever.ini (GitHub Releases)")?;
            let msg = crate::selfupdate::run(&repo).map_err(|e| e.to_string())?;
            println!("{msg}");
        }
        "help" | "--help" | "-h" => {
            println!(
                "addons-forever cli: headless addon management (same state as the GUI)\n\n\
                 addons-forever cli scan                       scan AddOns dir, match fingerprints\n\
                 addons-forever cli list                       tracked addons\n\
                 addons-forever cli check                      check for updates\n\
                 addons-forever cli update [name-filter]       install updates\n\
                 addons-forever cli install <target>           github owner/repo | curseforge slug/id |\n\
                                                               wowinterface url | tukui slug | .zip URL\n\
                 addons-forever cli uninstall <name>           remove an addon\n\
                 addons-forever cli set-path <dir>             AddOns folder\n\
                 addons-forever cli set-flavor <flavor>        forever (default), retail, ...\n\
                 addons-forever cli set-version <x.y.z>        client version for compatibility\n\
                 addons-forever cli offline on|off             privacy: no network at all\n\
                 addons-forever cli selfupdate                 check app releases (GitHub)\n"
            );
        }
        "export" => {
            let path = args.get(1).ok_or("usage: cli export <file.json>")?;
            let mut addons: Vec<crate::worker::ExportEntry> = store
                .addons
                .values()
                .map(|a| crate::worker::ExportEntry {
                    provider: a.provider.as_str().to_string(),
                    external_id: a.external_id.clone(),
                    name: a.name.clone(),
                    auto_update: a.auto_update,
                    channel: a.channel.as_str().to_string(),
                })
                .collect();
            addons.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            let manifest = crate::worker::ExportManifest {
                format: crate::worker::EXPORT_FORMAT.to_string(),
                version: 1,
                exported_at: chrono::Local::now().to_rfc3339(),
                addons,
            };
            let n = manifest.addons.len();
            std::fs::write(path, serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            println!("exported {n} addon(s) to {path}");
        }
        "import" => {
            let path = args.get(1).ok_or("usage: cli import <file.json>")?;
            let raw = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
            let manifest: crate::worker::ExportManifest =
                serde_json::from_str(&raw).map_err(|e| e.to_string())?;
            if manifest.format != crate::worker::EXPORT_FORMAT {
                return Err("not an addons-forever export file".into());
            }
            let n = manifest.addons.len();
            let mut ok = 0usize;
            for e in manifest.addons {
                let Some(provider) = ProviderKind::from_key(&e.provider) else {
                    eprintln!("skip `{}`: unknown provider `{}`", e.name, e.provider);
                    continue;
                };
                let key = AddonState::key(provider, &e.external_id);
                let r: Result<(), String> = async {
                    let prov = reg.get(provider);
                    let metas = prov
                        .meta(&ctx, std::slice::from_ref(&e.external_id))
                        .await
                        .map_err(|e| e.to_string())?;
                    let meta = metas.into_iter().next().ok_or("provider returned no data")?;
                    let file = prov.pick(&meta, &sel).ok_or("no compatible file for this client/channel")?;
                    let outcome = addons_forever_core::engine::install::install(
                        &ctx, &addons_dir, &meta, &file, &mut |_| {},
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                    let mut st = store.addons.get(&key).cloned().unwrap_or_else(|| AddonState {
                        key: key.clone(),
                        provider,
                        external_id: e.external_id.clone(),
                        name: meta.name.clone(),
                        site_url: meta.site_url.clone(),
                        repo_url: meta.repo_url.clone(),
                        icon_url: meta.icon_url.clone(),
                        folders: vec![],
                        fingerprints: vec![],
                        installed_version: None,
                        installed_file_id: None,
                        size_bytes: 0,
                        channel: cfg.updates.channel,
                        auto_update: false,
                        ignored: false,
                        manual: provider == ProviderKind::Zip,
                        cached_remote: None,
                        cached_at: None,
                        last_checked: None,
                        provider_data: serde_json::Value::Null,
                        pending_update: None,
                    });
                    st.installed_file_id = Some(file.file_id.clone());
                    st.installed_version = outcome.version;
                    st.folders = outcome.folders;
                    st.size_bytes = outcome.size_bytes;
                    st.cached_remote = Some(meta);
                    st.auto_update = e.auto_update;
                    if let Some(ch) = addons_forever_core::types::Channel::from_key(&e.channel) {
                        st.channel = ch;
                    }
                    store.upsert(st);
                    Ok(())
                }
                .await;
                match r {
                    Ok(()) => {
                        ok += 1;
                        println!("installed {} (from {})", e.name, e.provider);
                    }
                    Err(msg) => eprintln!("import `{}` failed: {msg}", e.name),
                }
            }
            persist_flags(data_dir, &store)?;
            println!("import done: {ok}/{n} addon(s) installed");
        }
        other => return Err(format!("unknown command `{other}`, try `addons-forever cli help`")),
    }
    Ok(())
}

/// Parse a CLI install target (shares URL shapes with the GUI box).
pub async fn parse_target(ctx: &AppCtx, raw: &str) -> Result<(ProviderKind, String, String), String> {
    crate::worker::parse_install_url_pub(ctx, raw).await
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let cut: String = s.chars().take(n - 1).collect();
        format!("{cut}…")
    }
}

fn fail(msg: &str) -> i32 {
    eprintln!("error: {msg}");
    1
}

/// Windows release builds are built with the windows subsystem (no console);
/// attach the parent console so CLI output is visible.
fn attach_console() {
    #[cfg(windows)]
    {
        use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
        unsafe {
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}
