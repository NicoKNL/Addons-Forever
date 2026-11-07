//! Small shared helpers: paths, version comparison, atomic file writes.

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

/// Directory the app keeps its data in: next to the executable (portable app).
/// Overridable with `ADDONS_FOREVER_DATA_DIR` (used by tests and CLI `--data-dir`).
pub fn data_dir() -> Result<PathBuf> {
    if let Ok(dir) = std::env::var("ADDONS_FOREVER_DATA_DIR") {
        let p = PathBuf::from(dir);
        std::fs::create_dir_all(&p)?;
        return Ok(p);
    }
    let exe = std::env::current_exe().map_err(|e| Error::Config(format!("current_exe: {e}")))?;
    let dir = exe
        .parent()
        .ok_or_else(|| Error::Config("exe has no parent dir".into()))?
        .to_path_buf();
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Write `bytes` to `path` atomically (tmp file + rename).
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Loose, provider-agnostic version comparison key: leading numbers.
/// `"1.60.1-beta.2" -> [1, 60, 1, 2]` (non-numeric segments dropped).
pub fn version_key(v: &str) -> Vec<u64> {
    let mut out = Vec::new();
    for token in v.split(|c: char| !c.is_ascii_digit()) {
        if let Ok(n) = token.parse::<u64>() {
            out.push(n);
        }
    }
    out
}

/// `Some(Ordering)`-style compare: returns true if `a` is newer than `b`.
pub fn version_is_newer(a: &str, b: &str) -> bool {
    version_key(a).as_slice() > version_key(b).as_slice()
}

/// Case-insensitive path lookup: resolve `rel` (with `/` separators) under `base`
/// matching file/dir names ignoring case (TOC include paths are case-insensitive on Windows).
pub fn lookup_ci(base: &Path, rel: &str) -> Option<PathBuf> {
    let mut cur = base.to_path_buf();
    for seg in rel.split(['/', '\\']) {
        match seg {
            "" | "." => continue,
            _ => {}
        }
        let mut found = None;
        if cur.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&cur) {
                for e in entries.flatten() {
                    if e.file_name().to_string_lossy().eq_ignore_ascii_case(seg) {
                        found = Some(e.path());
                        break;
                    }
                }
            }
        } else {
            // single file?
            if cur.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.eq_ignore_ascii_case(seg)) {
                found = Some(cur.clone());
            }
        }
        cur = found?;
    }
    Some(cur)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_keys() {
        assert!(version_is_newer("1.60.2", "1.60.1"));
        assert!(version_is_newer("1.61", "1.60.9"));
        assert!(!version_is_newer("1.6.0", "1.60.1"));
        assert!(version_is_newer("v1.5.1-forever", "1.5.0"));
        assert_eq!(version_key("v427.8"), vec![427, 8]);
    }

    #[test]
    fn lookup_case_insensitive() {
        let dir = std::env::temp_dir().join("fa-lookup-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Locales")).unwrap();
        std::fs::write(dir.join("Locales").join("enUS.lua"), b"x").unwrap();
        std::fs::write(dir.join("Core.lua"), b"x").unwrap();
        assert!(lookup_ci(&dir, "core.lua").is_some());
        assert!(lookup_ci(&dir, "Locales/enus.lua").is_some());
        assert!(lookup_ci(&dir, "Locales\\enUS.lua").is_some());
        assert!(lookup_ci(&dir, "missing.lua").is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
