//! WoW TOC file parsing: metadata fields, include lists, encoding detection,
//! per-flavor TOC selection.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::flavor::Flavor;

/// Parsed, provider-relevant TOC content.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ParsedToc {
    pub title: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    /// All `## Interface:` numbers found (comma-separated lists supported).
    pub interface: Vec<u32>,
    pub load_on_demand: bool,
    pub optional: bool,
    pub dependencies: Vec<String>,
    pub saved_vars: Vec<String>,
    pub saved_chars: Vec<String>,
    pub ids: TocIds,
}

/// Provider identity lines authors embed in TOCs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct TocIds {
    pub curse: Option<String>,
    pub wowi: Option<String>,
    pub tukui: Option<String>,
    pub tukui_folders: Vec<String>,
    pub wago: Option<String>,
    pub github: Option<String>,
}

impl ParsedToc {
    pub fn max_interface(&self) -> Option<u32> {
        self.interface.iter().copied().max()
    }
}

/// Decode TOC bytes: UTF-8 (with/without BOM) and UTF-16 LE/BE.
pub fn decode_text(raw: &[u8]) -> String {
    if raw.starts_with(&[0xFF, 0xFE]) {
        return encoding_rs::UTF_16LE.decode(&raw[2..]).0.into_owned();
    }
    if raw.starts_with(&[0xFE, 0xFF]) {
        return encoding_rs::UTF_16BE.decode(&raw[2..]).0.into_owned();
    }
    let body = raw.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(raw);
    String::from_utf8_lossy(body).into_owned()
}

pub fn parse(content: &str) -> ParsedToc {
    let mut t = ParsedToc::default();
    for line in content.lines() {
        let line = line.trim_end_matches('\r');
        if !line.starts_with("##") {
            continue;
        }
        let rest = &line[2..];
        let (key, value) = match rest.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };
        let key = key.to_ascii_lowercase();
        match key.as_str() {
            "title" => t.title = Some(strip_color_codes(value)),
            "author" => t.author = Some(strip_color_codes(value)),
            "version" => t.version = Some(value.to_string()),
            "interface" => {
                for part in value.split(',') {
                    if let Ok(n) = part.trim().parse::<u32>() {
                        t.interface.push(n);
                    }
                }
            }
            "interface-mainline" | "interface-classic" => {
                for part in value.split(',') {
                    if let Ok(n) = part.trim().parse::<u32>() {
                        t.interface.push(n);
                    }
                }
            }
            "loadondemand" => t.load_on_demand = parse_bool(value),
            "optional" => t.optional = parse_bool(value),
            "dependencies" => {
                t.dependencies = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
            }
            "savedvariables" => push_csv(&mut t.saved_vars, value),
            "savedvariablespercharacter" => push_csv(&mut t.saved_chars, value),
            "x-curse-project-id" | "x-curse-embedded-mod-id" | "x-cursemprojectid" | "x-curse-projectid" => {
                t.ids.curse = nonempty(value)
            }
            "x-wowi-id" | "x-wowinterface-id" => t.ids.wowi = nonempty(value),
            "x-tukui-projectid" => t.ids.tukui = nonempty(value),
            "x-tukui-projectfolders" => {
                t.ids.tukui_folders = value.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
            }
            "x-wago-id" => t.ids.wago = nonempty(value),
            "x-github" | "x-repo" | "x-source" => t.ids.github = nonempty(value),
            _ => {}
        }
    }
    t
}

fn push_csv(v: &mut Vec<String>, csv: &str) {
    for part in csv.split(',') {
        let p = part.trim();
        if !p.is_empty() {
            v.push(p.to_string());
        }
    }
}

fn parse_bool(v: &str) -> bool {
    matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes")
}

fn nonempty(v: &str) -> Option<String> {
    let v = v.trim();
    (!v.is_empty()).then(|| v.to_string())
}

fn strip_color_codes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '|' {
            // consume |cffxxxxxx / |r sequences
            if chars.peek() == Some(&'c') || chars.peek() == Some(&'C') {
                // consume `c` + 6 hex + 2
                for _ in chars.by_ref().take(9) {}
                continue;
            }
            if chars.peek() == Some(&'r') || chars.peek() == Some(&'R') {
                chars.next();
                continue;
            }
        }
        out.push(c);
    }
    out.trim().to_string()
}

/// Load-order includes of a .toc — mirroring the CurseForge fingerprint rules
/// (verified against live module hashes): a line counts iff, after stripping
/// comments, it *ends* with `.lua`/`.xml` (a trailing `[condition]` token disqualifies
/// the line server-side), leading `[condition]` tokens are stripped, and `..` paths
/// are rejected.
pub fn toc_includes(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim_end_matches('\r');
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let path_part = trimmed.split('#').next().unwrap_or("").trim();
        let lower = path_part.to_ascii_lowercase();
        if !(lower.ends_with(".lua") || lower.ends_with(".xml")) {
            continue;
        }
        if path_part.contains("..") {
            continue;
        }
        // strip leading `[condition]` tokens
        let path = strip_bracket_tokens(path_part);
        if path.is_empty() {
            continue;
        }
        out.push(path.to_string());
    }
    out
}

/// Remove all balanced `[...]` groups and trim.
fn strip_bracket_tokens(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut depth = 0usize;
    for c in s.chars() {
        match c {
            '[' => depth += 1,
            ']' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

/// Includes of an .xml load file: `<Include file="...">` / `<Script file="..."/>`.
pub fn xml_includes(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    // strip <!-- --> comments
    let mut text = String::with_capacity(content.len());
    let mut rest = content;
    while let Some(start) = rest.find("<!--") {
        text.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = rest.find("-->").map(|e| e + 3).unwrap_or(rest.len());
        rest = &rest[end..];
    }
    text.push_str(rest);

    let lower = text.to_ascii_lowercase();
    let mut from = 0;
    loop {
        let inc = lower[from..].find("<include");
        let scr = lower[from..].find("<script");
        let rel = match (inc, scr) {
            (Some(a), Some(b)) => a.min(b),
            (Some(a), None) => a,
            (None, Some(b)) => b,
            (None, None) => break,
        };
        let tag_start = from + rel;
        let Some(tag_end) = lower[tag_start..].find('>') else { break };
        let tag = &lower[tag_start..tag_start + tag_end + 1];
        from = tag_start + tag_end + 1;
        if !tag.starts_with("<include") && !tag.starts_with("<script") {
            continue;
        }
        if let Some(rest) = after_file_attr(&text, tag_start, tag) {
            if !rest.contains("..") {
                out.push(rest);
            }
        }
    }
    out
}

fn after_file_attr(original: &str, tag_start: usize, tag_lower: &str) -> Option<String> {
    let at = tag_lower.find("file=")?;
    let abs = tag_start + at + "file=".len();
    let bytes = original.as_bytes();
    let quote = *bytes.get(abs)? as char;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = &original[abs + 1..];
    let end = inner.find(quote)?;
    let val = &inner[..end];
    let l = val.to_ascii_lowercase();
    (l.ends_with(".lua") || l.ends_with(".xml")).then(|| val.to_string())
}

/// All TOC file names known (flavor suffixes the client/fingerprint accept).
pub const TOC_SUFFIXES: &[&str] = &[
    "mainline", "standard", "forever", "camelot", "classic", "era", "vanilla", "tbc", "bcc", "wrath",
    "wotlkc", "cata", "mists", "titan", "ptr", "beta",
];

/// Find the TOC to trust for `flavor` inside an addon folder, by the client's
/// precedence: flavor-specific suffix > family suffix > plain.
pub fn find_toc(dir: &Path, folder_name: &str, flavor: Flavor) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = Vec::new();
    let rd = std::fs::read_dir(dir).ok()?;
    for e in rd.flatten() {
        let p = e.path();
        if p.extension().and_then(|x| x.to_str()).map(|x| x.eq_ignore_ascii_case("toc")) != Some(true) {
            continue;
        }
        candidates.push(p);
    }
    candidates.sort();
    let stem_root = folder_name.to_ascii_lowercase();
    let suffixes = flavor.toc_suffixes();
    let mut plain: Option<PathBuf> = None;
    let mut any: Option<PathBuf> = None;
    for p in &candidates {
        let stem = p.file_stem().map(|s| s.to_string_lossy().to_lowercase()).unwrap_or_default();
        let (root, suffix) = split_toc_stem(&stem);
        let root_ok = root == stem_root || stem_root.is_empty();
        any.get_or_insert_with(|| p.clone());
        if !root_ok {
            continue;
        }
        if let Some(suf) = suffix {
            if suffixes.contains(&suf.as_str()) {
                return Some(p.clone());
            }
        } else if plain.is_none() {
            plain = Some(p.clone());
        }
    }
    plain.or(any)
}

/// `"bagnon_forever" -> ("bagnon", Some("forever".to_string()))`, `"details" -> ("details", None)`.
pub fn split_toc_stem(stem: &str) -> (String, Option<String>) {
    let lower = stem.to_ascii_lowercase();
    if let Some(pos) = lower.rfind(['-', '_']) {
        let suf = lower[pos + 1..].to_string();
        if TOC_SUFFIXES.contains(&suf.as_str()) {
            return (stem[..pos].to_ascii_lowercase(), Some(suf));
        }
    }
    (lower, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_fields() {
        let toc = parse(
            "## Title: |cff00ff00My |rAddOn\n\
             ## Author: Me\n\
             ## Version: 1.2.3\n\
             ## Interface: 16001, 120105\n\
             ## LoadOnDemand: 1\n\
             ## SavedVariables: MyDB\n\
             ## X-Curse-Project-ID: 12345\n\
             ## X-WoWI-ID: 999\n\
             ## X-Tukui-ProjectID: -3\n\
             # comment\n\
             Core.lua\n",
        );
        assert_eq!(toc.title.as_deref(), Some("My AddOn"));
        assert_eq!(toc.version.as_deref(), Some("1.2.3"));
        assert_eq!(toc.interface, vec![16001, 120105]);
        assert!(toc.load_on_demand);
        assert_eq!(toc.saved_vars, vec!["MyDB"]);
        assert_eq!(toc.ids.curse.as_deref(), Some("12345"));
        assert_eq!(toc.ids.wowi.as_deref(), Some("999"));
        assert_eq!(toc.ids.tukui.as_deref(), Some("-3"));
    }

    #[test]
    fn includes_rules() {
        let inc = toc_includes(
            "## Interface: 1\nCore.lua\nSub\\Mod.xml   # trailing comment\n[Default@Player] Opt.lua\nBoss_Classic.lua [AllowLoadGameType classic]\n## LoadOnDemand: 1\nREADME.md\n../evil.lua\nTextures\\bg.png\n",
        );
        assert_eq!(inc, vec!["Core.lua", "Sub\\Mod.xml", "Opt.lua"]);
    }


    #[test]
    fn xml_includes_rules() {
        let x = xml_includes(
            "<Ui><!-- <Script file=\"skip.xml\"/> --><Include file=\"A.xml\"></Include>\n<Script file='b.lua'/></Ui>",
        );
        assert_eq!(x, vec!["A.xml", "b.lua"]);
    }

    #[test]
    fn utf16_decode() {
        let mut v = vec![0xFF, 0xFE];
        v.extend("## Title: Ok".encode_utf16().flat_map(|c| c.to_le_bytes()));
        assert_eq!(decode_text(&v), "## Title: Ok");
    }

    #[test]
    fn stem_split() {
        assert_eq!(split_toc_stem("BigWigs_Forever"), ("bigwigs".into(), Some("forever".into())));
        assert_eq!(split_toc_stem("Details"), ("details".into(), None::<String>));
        assert_eq!(split_toc_stem("DBM-Core"), ("dbm-core".into(), None::<String>));
    }
}
