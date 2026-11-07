//! addons-forever core: providers, scanning, installation and update engine for
//! World of Warcraft addon management (Forever-first, flavor-aware).

pub mod config;
pub mod engine;
pub mod error;
pub mod fingerprint;
pub mod flavor;
pub mod net;
pub mod providers;
pub mod store;
pub mod toc;
pub mod types;
pub mod util;

pub use error::{Error, Result};

/// Re-exports that most consumers need.
pub mod prelude {
    pub use crate::config::AppConfig;
    pub use crate::error::{Error, Result};
    pub use crate::flavor::Flavor;
    pub use crate::types::{AppCtx, Channel, FileSelect, ProviderKind, RemoteFile, RemoteMeta};
}
