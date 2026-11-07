//! CurseForge addon fingerprint: MurmurHash2-32 (seed 1) over whitespace-stripped
//! bytes; a folder fingerprint hashes the .toc + its recursive .lua/.xml include
//! closure + Bindings.xml, sorts the per-file hashes, concatenates their decimal
//! forms and hashes again. Verified against live CurseForge module hashes
//! (docs/fingerprint_verify.py).

use std::path::Path;

use crate::error::Result;
use crate::fingerprint::sealed::collect_hashed_files;

const M: u32 = 0x5BD1_E995; // 1_540_483_477

/// MurmurHash2 (32-bit) over `data` with `seed` — the raw primitive.
pub fn murmur2(data: &[u8], seed: u32) -> u32 {
    let mut h: u32 = seed ^ (data.len() as u32);
    let (mut i, mut shift): (u32, u32) = (0, 0);
    for &b in data {
        i |= (b as u32) << shift;
        shift += 8;
        if shift == 32 {
            i = i.wrapping_mul(M);
            i ^= i >> 24;
            i = i.wrapping_mul(M);
            h = h.wrapping_mul(M);
            h ^= i;
            i = 0;
            shift = 0;
        }
    }
    if shift > 0 {
        h ^= i;
        h = h.wrapping_mul(M);
    }
    h ^= h >> 13;
    h = h.wrapping_mul(M);
    h ^= h >> 15;
    h
}

/// Hash of a single file: space/tab/LF/CR bytes stripped, MurmurHash2 seed 1.
pub fn file_fingerprint(bytes: &[u8]) -> u32 {
    let stripped: Vec<u8> = bytes
        .iter()
        .copied()
        .filter(|b| !matches!(b, 0x20 | 0x09 | 0x0A | 0x0D))
        .collect();
    murmur2(&stripped, 1)
}

/// Fingerprint of one addon folder (the unit CurseForge stores in `modules[]`).
/// Returns None when the folder has no hashable TOC closure.
pub fn folder_fingerprint(dir: &Path) -> Result<Option<u32>> {
    let files = collect_hashed_files(dir)?;
    if files.is_empty() {
        return Ok(None);
    }
    let mut hashes: Vec<u32> = Vec::with_capacity(files.len());
    for path in files {
        let bytes = std::fs::read(&path)?;
        hashes.push(file_fingerprint(&bytes));
    }
    hashes.sort_unstable();
    let joined: String = hashes.iter().map(|h| h.to_string()).collect();
    Ok(Some(murmur2(joined.as_bytes(), 1)))
}

mod sealed {
    use std::path::{Path, PathBuf};

    use crate::error::Result;
    use crate::toc;
    use crate::util::lookup_ci;

    /// All files that take part in a folder fingerprint, deduplicated.
    pub fn collect_hashed_files(dir: &Path) -> Result<Vec<PathBuf>> {
        let mut out: Vec<PathBuf> = Vec::new();
        let mut seen: Vec<PathBuf> = Vec::new();

        let entries: Vec<PathBuf> = std::fs::read_dir(dir)?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .collect();

        // top-level TOCs named `<Folder>[-_][flavor].toc` (prefix up to the first
        // separator must match the folder's prefix — `BigWigs_Core/BigWigs_Core.toc` ✓)
        let folder = dir
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let folder_root = folder.split(['-', '_']).next().unwrap_or("");
        let tocs: Vec<PathBuf> = entries
            .iter()
            .filter(|p| {
                p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("toc")) == Some(true)
            })
            .filter(|p| {
                let stem = p.file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
                let root = stem.split(['-', '_']).next().unwrap_or("");
                root == folder_root
            })
            .cloned()
            .collect();

        // Bindings.xml is always part of the set
        if let Some(b) = entries.iter().find(|p| {
            p.file_name().map(|n| n.to_string_lossy().eq_ignore_ascii_case("Bindings.xml")).unwrap_or(false)
        }) {
            push(&mut seen, &mut out, b.clone());
        }

        for toc_path in &tocs {
            push(&mut seen, &mut out, toc_path.clone());
            let raw = std::fs::read(toc_path)?;
            let text = toc::decode_text(&raw);
            for inc in toc::toc_includes(&text) {
                if let Some(p) = lookup_ci(dir, &inc) {
                    visit_file(dir, &p, &mut seen, &mut out)?;
                }
            }
        }
        Ok(out)
    }

    fn visit_file(
        base: &Path,
        path: &Path,
        seen: &mut Vec<PathBuf>,
        out: &mut Vec<PathBuf>,
    ) -> Result<()> {
        if !push(seen, out, path.to_path_buf()) {
            return Ok(()); // already visited
        }
        let is_xml = path.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("xml")).unwrap_or(false);
        if is_xml {
            let raw = std::fs::read(path)?;
            let text = toc::decode_text(&raw);
            // includes are relative to the containing file's directory
            let own_dir = path.parent().unwrap_or(base);
            for inc in toc::xml_includes(&text) {
                if let Some(p) = lookup_ci(own_dir, &inc) {
                    visit_file(base, &p, seen, out)?;
                }
            }
        }
        Ok(())
    }

    fn push(seen: &mut Vec<PathBuf>, out: &mut Vec<PathBuf>, p: PathBuf) -> bool {
        if seen.contains(&p) {
            return false;
        }
        seen.push(p.clone());
        out.push(p);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fixtures generated by the independently implemented python version
    // (docs/fingerprint_verify.py) running the same published algorithm.
    fn py(data: &[u8]) -> u32 {
        file_fingerprint(data)
    }

    #[test]
    fn murmur2_fixtures() {
        // fixtures from the independent python implementation (docs/fingerprint_verify.py)
        assert_eq!(murmur2(b"", 1), 1_540_447_798);
        assert_eq!(murmur2(b"a", 1), 626_045_324);
        assert_eq!(murmur2(b"abc", 1), 1_621_425_345);
    }

    #[test]
    fn file_fixtures() {
        assert_eq!(py(b""), 1_540_447_798);
        assert_eq!(py(b"hello world"), py(b"helloworld"));
        assert_eq!(py(b"## Interface: 16001\r\n"), py(b"##Interface:16001"));
    }

    #[test]
    fn folder_fingerprint_fixture() {
        // value computed by the independent python implementation over the same content
        let dir = std::env::temp_dir().join(format!("fa-fp-{}", std::process::id())).join("Fixie");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Libs")).unwrap();
        std::fs::write(
            dir.join("Fixie.toc"),
            "## Interface: 16001\nCore.lua\nLibs/Embed.xml\n",
        )
        .unwrap();
        std::fs::write(dir.join("Core.lua"), b"-- hi\nprint(1)\n").unwrap();
        std::fs::write(
            dir.join("Libs").join("Embed.xml"),
            "<Ui><Script file=\"../Core.lua\"/>\n<Include file=\"Deep.xml\"/></Ui>\n",
        )
        .unwrap();
        std::fs::write(dir.join("Libs").join("Deep.xml"), b"<!-- no includes -->\n").unwrap();
        std::fs::write(dir.join("README.md"), b"not hashed").unwrap();
        let fp = folder_fingerprint(&dir).unwrap();
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
        let fp = fp.expect("fixture has a toc");
        assert_eq!(fp, 2_016_507_564, "folder fingerprint drift");
    }
}
