//! Single-instance guard.

use std::path::Path;

pub struct SingleInstanceGuard {
    #[cfg(unix)]
    _lock: LockFile,
}

impl SingleInstanceGuard {
    /// Returns None when another instance holds the lock (caller should exit).
    pub fn acquire(data_dir: &Path) -> Option<Self> {
        #[cfg(windows)]
        {
            let _ = data_dir;
            use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
            use windows_sys::Win32::System::Threading::CreateMutexW;
            unsafe {
                let name: Vec<u16> = "Global\\addons-forever-instance\0".encode_utf16().collect();
                let handle: HANDLE = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
                if handle == -1_isize as _ && GetLastError() != ERROR_ALREADY_EXISTS {
                    // creation failed (e.g. no rights for Global\\); fall back to Local
                    let name: Vec<u16> = "Local\\addons-forever-instance\0".encode_utf16().collect();
                    let handle = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
                    if GetLastError() == ERROR_ALREADY_EXISTS {
                        return None;
                    }
                    if handle == -1_isize as _ {
                        return None;
                    }
                    std::mem::forget(handle);
                    return Some(SingleInstanceGuard {});
                }
                if GetLastError() == ERROR_ALREADY_EXISTS {
                    let _ = CloseHandle(handle);
                    return None;
                }
                std::mem::forget(handle); // keep for process lifetime
                Some(SingleInstanceGuard {})
            }
        }
        #[cfg(not(windows))]
        {
            let lock = LockFile::acquire(&data_dir.join(".single-instance.lock")).ok()?;
            Some(SingleInstanceGuard { _lock: lock })
        }
    }
}

#[cfg(unix)]
struct LockFile {
    path: std::path::PathBuf,
}

#[cfg(unix)]
impl LockFile {
    /// Best-effort lock for the Linux dev build: O_EXCL pid file with stale takeover.
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
                        .parse::<u32>()
                        .map(|p| std::path::Path::new(&format!("/proc/{p}")).exists())
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
