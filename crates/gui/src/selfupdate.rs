//! App self-update from the project's GitHub Releases (zip asset
//! `addons-forever-<target>.zip`). Windows-only swap (rename self + replace);
//! other platforms report and exit. No telemetry: one release-API call.

use addons_forever_core::net;
use addons_forever_core::types::AppCtx;
use addons_forever_core::util::version_cmp;

/// Query the release feed; `Some(version)` when it is newer than this build.
/// One API call, no download.
pub async fn check(ctx: &AppCtx, repo: &str) -> Result<Option<String>, String> {
    let (owner, name) = repo.split_once('/').ok_or("update repo must be `owner/name`")?;
    let url = format!("https://api.github.com/repos/{owner}/{name}/releases/latest");
    let v: serde_json::Value = net::get_json(&ctx.http, &url, &[]).await.map_err(|e| e.to_string())?;
    let Some(tag) = v.get("tag_name").and_then(|t| t.as_str()) else {
        return Err("release feed returned no tag_name".into());
    };
    let latest = tag.trim().trim_start_matches('v').to_string();
    match version_cmp(&latest, env!("CARGO_PKG_VERSION")) {
        std::cmp::Ordering::Greater => Ok(Some(latest)),
        _ => Ok(None),
    }
}

pub fn run(repo: &str) -> Result<String, String> {
    let (owner, name) = repo.split_once('/').ok_or("update repo must be `owner/name`")?;

    #[cfg(windows)]
    {
        use self_update::backends::github::Update;
        let target = self_update::get_target();
        let status = Update::configure()
            .repo_owner(owner)
            .repo_name(name)
            .target(target)
            .bin_name("addons-forever")
            .current_version(env!("CARGO_PKG_VERSION"))
            .no_confirm(true)
            .show_output(false)
            .show_download_progress(false)
            .build()
            .map_err(|e| e.to_string())?
            .update()
            .map_err(|e| e.to_string())?;
        if status.is_updated() {
            Ok(format!(
                "Updated to {}. The new version starts on the next launch.",
                status.version()
            ))
        } else {
            Ok(format!("Already up to date ({}).", env!("CARGO_PKG_VERSION")))
        }
    }
    #[cfg(not(windows))]
    {
        let _ = (owner, name);
        Ok("App self-update ships with the Windows build; on other platforms rebuild from source.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live release-feed check (FA_LIVE=1 cargo test -- --include-ignored).
    #[test]
    #[ignore]
    fn check_live_feed() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cfg = std::sync::Arc::new(addons_forever_core::config::AppConfig::default());
            let ctx = addons_forever_core::types::AppCtx::new(cfg, env!("CARGO_PKG_VERSION"));
            let r = check(&ctx, addons_forever_core::config::DEFAULT_UPDATE_REPO).await;
            assert!(r.is_ok(), "release feed: {:?}", r.err());
            // a repo without releases must fail, not hang
            let r = check(&ctx, "Yoremto/Addons-Forever-nonexistent-repo").await;
            assert!(r.is_err());
        });
    }
}
