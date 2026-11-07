//! Core shared types: flavors, providers, remote addon metadata, provider trait.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::error::Result;
use crate::flavor::Flavor;
use crate::toc::ParsedToc;

/// Which addon provider an addon comes from.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProviderKind {
    Curse,
    GitHub,
    Tukui,
    WowInterface,
    WowUpHub,
    Zip,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ProviderKind::Curse => "curse",
            ProviderKind::GitHub => "github",
            ProviderKind::Tukui => "tukui",
            ProviderKind::WowInterface => "mmoui",
            ProviderKind::WowUpHub => "hub",
            ProviderKind::Zip => "zip",
        }
    }

    /// Inverse of [`Self::as_str`] (used by the portable export format).
    pub fn from_key(s: &str) -> Option<Self> {
        Some(match s {
            "curse" => ProviderKind::Curse,
            "github" => ProviderKind::GitHub,
            "tukui" => ProviderKind::Tukui,
            "mmoui" => ProviderKind::WowInterface,
            "hub" => ProviderKind::WowUpHub,
            "zip" => ProviderKind::Zip,
            _ => return None,
        })
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "curse" | "curseforge" => Some(ProviderKind::Curse),
            "github" | "gh" => Some(ProviderKind::GitHub),
            "tukui" => Some(ProviderKind::Tukui),
            "mmoui" | "wowi" | "wowinterface" => Some(ProviderKind::WowInterface),
            "hub" | "wowup" | "wowuphub" => Some(ProviderKind::WowUpHub),
            "zip" | "url" => Some(ProviderKind::Zip),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ProviderKind::Curse => "CurseForge",
            ProviderKind::GitHub => "GitHub",
            ProviderKind::Tukui => "Tukui",
            ProviderKind::WowInterface => "MMOUI",
            ProviderKind::WowUpHub => "WowUp Hub",
            ProviderKind::Zip => "Zip URL",
        }
    }
}

/// Release channel of a file; ordered so `file.channel <= user_channel` filters.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Channel {
    #[default]
    Stable,
    Beta,
    Alpha,
}

impl Channel {
    /// Inverse of [`Self::as_str`] (used by the portable export format).
    pub fn from_key(s: &str) -> Option<Self> {
        Some(match s {
            "stable" => Channel::Stable,
            "beta" => Channel::Beta,
            "alpha" => Channel::Alpha,
            _ => return None,
        })
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Stable => "stable",
            Channel::Beta => "beta",
            Channel::Alpha => "alpha",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "stable" | "release" => Some(Channel::Stable),
            "beta" => Some(Channel::Beta),
            "alpha" => Some(Channel::Alpha),
            _ => None,
        }
    }
}

impl std::fmt::Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One distributable file (release asset) of a remote addon.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RemoteFile {
    /// Provider-internal file/release id (stringified).
    pub file_id: String,
    /// Version string as published by the author.
    pub version: String,
    pub channel: Channel,
    pub file_name: String,
    /// Direct download URL (already authenticated, provider-resolved).
    pub url: String,
    /// Extra headers required for the download (e.g. CurseForge api key).
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    /// Flavors this file declares compatibility with. Empty = unknown/any.
    #[serde(default)]
    pub flavors: Vec<Flavor>,
    /// Publish date, unix seconds.
    pub date: Option<i64>,
    pub size: Option<u64>,
    /// md5 of the zip, if the provider publishes it (MMOUI, curse hashes).
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub changelog_url: Option<String>,
}

/// Remote metadata for one addon (project) from one provider.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RemoteMeta {
    pub provider: ProviderKind,
    /// Curse: mod id. GitHub: `owner/repo`. Tukui: slug. MMOUI: file id. Hub: id. Zip: url.
    pub external_id: String,
    pub name: String,
    pub author: Option<String>,
    pub site_url: Option<String>,
    pub repo_url: Option<String>,
    pub description: Option<String>,
    pub icon_url: Option<String>,
    pub downloads: Option<i64>,
    pub updated_at: Option<i64>,
    /// Newest first.
    pub files: Vec<RemoteFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchItem {
    pub provider: ProviderKind,
    pub external_id: String,
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub downloads: Option<i64>,
    pub icon_url: Option<String>,
    pub site_url: Option<String>,
    pub updated_at: Option<i64>,
    pub latest_version: Option<String>,
    /// Size of the latest file in bytes, when the provider reports it.
    #[serde(default)]
    pub size: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct SearchPage {
    pub items: Vec<SearchItem>,
    pub total: u32,
    pub page: u32,
    pub page_size: u32,
}

/// Runtime context shared by all providers.
#[derive(Clone)]
pub struct AppCtx {
    pub http: reqwest::Client,
    pub cfg: Arc<AppConfig>,
    pub app_version: &'static str,
}

impl AppCtx {
    pub fn new(cfg: Arc<AppConfig>, app_version: &'static str) -> Self {
        let ua = cfg.user_agent(app_version);
        AppCtx { http: crate::net::build_client(&ua), cfg, app_version }
    }
}

/// What "compatible with my client" means when picking files.
#[derive(Clone, Debug)]
pub struct FileSelect {
    pub flavor: Flavor,
    /// e.g. 16001
    pub interface: u32,
    /// e.g. "1.60.1"
    pub version: String,
    pub channel: Channel,
}

impl FileSelect {
    /// Selection derived from config: flavor, client version (explicit or the
    /// known-current default), interface number, global channel.
    pub fn from_cfg(cfg: &AppConfig) -> Self {
        Self::from_cfg_channel(cfg, cfg.updates.channel)
    }

    pub fn from_cfg_channel(cfg: &AppConfig, channel: Channel) -> Self {
        let version = cfg
            .app
            .game_version
            .clone()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| crate::flavor::default_client_version(cfg.app.flavor));
        let interface = crate::flavor::version_to_interface(&version).unwrap_or(0);
        FileSelect { flavor: cfg.app.flavor, interface, version, channel }
    }
}

/// A scanned addon folder on disk.
#[derive(Clone, Debug)]
pub struct FolderInfo {
    /// Folder name inside AddOns.
    pub name: String,
    pub path: PathBuf,
    /// Best-matching TOC for the current flavor.
    pub toc: ParsedToc,
    /// CurseForge folder fingerprint (MurmurHash2), None if no TOC closure.
    pub curse_fingerprint: Option<u32>,
    /// Total size of the folder on disk, bytes.
    pub size_bytes: u64,
}

/// One provider-matched addon: the folders on disk that belong to it.
#[derive(Clone, Debug)]
pub struct ScanMatch {
    pub provider: ProviderKind,
    pub external_id: String,
    /// Folder names (in AddOns) claimed by this addon.
    pub folders: Vec<String>,
    pub meta: RemoteMeta,
    /// Version parsed from the installed TOC, if any.
    pub installed_version: Option<String>,
}

#[derive(Debug)]
pub struct ScanRequest<'a> {
    pub folders: &'a [FolderInfo],
    pub sel: &'a FileSelect,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn kind(&self) -> ProviderKind;

    /// Batch-fetch metadata for known external ids. Missing/unknown ids are skipped.
    async fn meta(&self, ctx: &AppCtx, ids: &[String]) -> Result<Vec<RemoteMeta>>;

    /// Claim scanned folders that this provider recognizes.
    async fn scan(&self, ctx: &AppCtx, req: &ScanRequest<'_>) -> Result<Vec<ScanMatch>>;

    /// Search the catalog for installation. Providers without search return
    /// `Error::Unsupported`.
    async fn search(
        &self,
        ctx: &AppCtx,
        query: &str,
        page: u32,
        per_page: u32,
        sel: &FileSelect,
    ) -> Result<SearchPage>;

    /// Changelog text for a file (markdown/html as provided).
    async fn changelog(
        &self,
        _ctx: &AppCtx,
        _meta: &RemoteMeta,
        file: &RemoteFile,
    ) -> Result<Option<String>> {
        Ok(file.changelog_url.clone().map(|u| format!("Changelog: {u}")))
    }

    /// Pick the best file for a selection. Overridable for providers with rich
    /// asset conventions (GitHub); default filters by flavor + channel, newest first.
    fn pick(&self, meta: &RemoteMeta, sel: &FileSelect) -> Option<RemoteFile> {
        pick_generic(meta, sel)
    }
}

/// Generic file selection: compatible flavor, channel, newest date then highest version.
pub fn pick_generic(meta: &RemoteMeta, sel: &FileSelect) -> Option<RemoteFile> {
    meta.files
        .iter()
        .filter(|f| f.channel <= sel.channel)
        .filter(|f| f.flavors.is_empty() || f.flavors.contains(&sel.flavor))
        .max_by_key(|f| (f.date.unwrap_or(0), crate::util::version_key(&f.version)))
        .cloned()
}

/// Parse a loose version string into a comparable numeric key.
pub use crate::util::version_key;
