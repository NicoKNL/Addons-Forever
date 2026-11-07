//! Provider registry: builds all providers, routes by kind.
//! Individual providers live in sibling modules.

pub mod curse;
pub mod github;
pub mod hub;
pub mod mmoui;
pub mod tukui;
pub mod zip;

use crate::types::{AppCtx, ProviderKind, Provider};

/// All providers, constructed once per app context.
pub struct Registry {
    pub curse: curse::CurseProvider,
    pub github: github::GitHubProvider,
    pub tukui: tukui::TukuiProvider,
    pub mmoui: mmoui::MmouiProvider,
    pub hub: hub::HubProvider,
    pub zip: zip::ZipProvider,
}

impl Registry {
    pub fn new(_ctx: &AppCtx) -> Self {
        Registry {
            curse: curse::CurseProvider::new(),
            github: github::GitHubProvider::new(),
            tukui: tukui::TukuiProvider::new(),
            mmoui: mmoui::MmouiProvider::new(),
            hub: hub::HubProvider::new(),
            zip: zip::ZipProvider::new(),
        }
    }

    pub fn get(&self, kind: ProviderKind) -> &dyn Provider {
        match kind {
            ProviderKind::Curse => &self.curse,
            ProviderKind::GitHub => &self.github,
            ProviderKind::Tukui => &self.tukui,
            ProviderKind::WowInterface => &self.mmoui,
            ProviderKind::WowUpHub => &self.hub,
            ProviderKind::Zip => &self.zip,
        }
    }

    /// Providers in scan/claim order.
    pub fn scan_order() -> [ProviderKind; 5] {
        [
            ProviderKind::Curse,
            ProviderKind::Tukui,
            ProviderKind::WowInterface,
            ProviderKind::GitHub,
            ProviderKind::WowUpHub,
        ]
    }
}
