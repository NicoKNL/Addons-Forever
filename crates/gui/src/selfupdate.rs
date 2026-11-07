//! App self-update from the project's GitHub Releases (zip asset
//! `addons-forever-<target>.zip`). Windows-only swap (rename self + replace);
//! other platforms report and exit. No telemetry: one release-API call.

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
