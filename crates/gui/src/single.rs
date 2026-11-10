//! Single-instance guard, keyed to the AddOns installation being mutated.
//!
//! Every process that reads the ini (GUI and CLI alike) takes the lock, so a
//! CLI writer can never race a running GUI on the same installation.

use std::path::Path;

pub struct SingleInstanceGuard {
    _private: (),
    #[cfg(unix)]
    _lock: LockFile,
}

/// Stable identifier for a target: the canonical AddOns dir when set,
/// else the data dir.
pub fn lock_key(data_dir: &Path, addons_dir: Option<&str>) -> String {
    let base = addons_dir
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| data_dir.display().to_string());
    // FNV-1a over the lowercased path: stable, short, no external dep
    let key = base.to_lowercase();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in key.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

impl SingleInstanceGuard {
    /// Returns None when another instance holds the lock (caller should exit).
    pub fn acquire(data_dir: &Path, addons_dir: Option<&str>) -> Option<Self> {
        let key = lock_key(data_dir, addons_dir);
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
            use windows_sys::Win32::System::Threading::CreateMutexW;
            unsafe {
                // CreateMutexW returns NULL on failure (INVALID_HANDLE_VALUE is
                // not a CreateMutexW failure signal); ERROR_ALREADY_EXISTS is
                // checked after a successful open.
                let try_mutex = |scope: &str| -> Result<HANDLE, ()> {
                    let name: Vec<u16> =
                        format!("{scope}\\addons-forever-{key}\0").encode_utf16().collect();
                    let handle: HANDLE = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
                    if handle.is_null() {
                        return Err(());
                    }
                    Ok(handle)
                };
                // Global fails for restricted/service sessions; Local still
                // serializes the interactive user's processes.
                let handle = match try_mutex("Global").or_else(|()| try_mutex("Local")) {
                    Ok(h) => h,
                    // cannot create a mutex at all: refuse to run a second
                    // writer blind (better a clear error than a race)
                    Err(()) => return None,
                };
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    let _ = CloseHandle(handle);
                    return None;
                }
                std::mem::forget(handle); // held for the process lifetime
                Some(SingleInstanceGuard { _private: () })
            }
        }
        #[cfg(not(windows))]
        {
            let lock = LockFile::acquire(&data_dir.join(format!(".single-instance-{key}.lock"))).ok()?;
            Some(SingleInstanceGuard { _private: (), _lock: lock })
        }
    }
}

#[cfg(unix)]
struct LockFile {
    path: std::path::PathBuf,
}

#[cfg(unix)]
impl LockFile {
    fn acquire(path: &Path) -> std::io::Result<Self> {
        let pid = std::process::id().to_string();
        for _ in 0..2 {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(path) {
                Ok(mut f) => {
                    use std::io::Write;
                    let _ = f.write_all(pid.as_bytes());
                    return Ok(LockFile { path: path.to_path_buf() });
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let existing = std::fs::read_to_string(path).unwrap_or_default();
                    let alive = existing
                        .trim()
                        .parse::<i32>()
                        // signal 0 = liveness probe; portable across Linux and
                        // macOS (which has no /proc), EPERM counts as alive
                        .map(|p| unsafe { libc::kill(p, 0) == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM) })
                        .unwrap_or(false);
                    if !alive {
                        let _ = std::fs::remove_file(path);
                        continue;
                    }
                    return Err(std::io::Error::other("locked by running instance"));
                }
                Err(e) => return Err(e),
            }
        }
        Err(std::io::Error::other("lock failed"))
    }
}

#[cfg(unix)]
impl Drop for LockFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_key_is_stable_and_case_insensitive() {
        let a = lock_key(std::path::Path::new("/d"), Some("C:\\Games\\World of Warcraft\\Interface\\AddOns"));
        let b = lock_key(std::path::Path::new("/d"), Some("c:\\games\\world of warcraft\\interface\\addons"));
        assert_eq!(a, b);
        let c = lock_key(std::path::Path::new("/d"), Some("D:\\other"));
        assert_ne!(a, c);
        // no explicit dir falls back to the data dir
        let d = lock_key(std::path::Path::new("/d"), None);
        assert!(!d.is_empty());
    }
}
