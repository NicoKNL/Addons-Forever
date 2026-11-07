# Provider implementation spec (M2)

All providers implement `crate::types::Provider` (see `crates/core/src/types.rs`).
Read `types.rs`, `net.rs`, `config.rs`, `store.rs`, `toc.rs`, `fingerprint.rs` FIRST —
they are the contract; do not change them (note deviations in your report).
All HTTP goes through `net::get_json` / `net::download_to` (allowlist enforced).
Live facts below were verified against the real APIs on 2026-11-06.

Common rules:
- `external_id` formats — Curse: mod id string ("2382"); GitHub: "owner/repo";
  Tukui: slug ("elvui"); MMOUI: file id ("8814"); Hub: hub id; Zip: the download URL.
- RemoteMeta.files: newest first. RemoteFile.flavors = compatible flavors, empty = any.
- Dates: unix seconds (convert from ISO8601, chrono).
- On non-2xx from APIs return `Error::provider(kind.label(), "...")` with status+snippet.
- Every provider gets unit tests: JSON fixtures in `#[cfg(test)]` strings, parse fn tested.
  Live network tests: `#[ignore]`-ed `#[tokio::test]` behind `FA_LIVE=1` env check.

## 1. CurseForge (`providers/curse.rs`)

- Base `https://api.curseforge.com`. Header `x-api-key: {ctx.cfg.network.curse_api_key}`
  (default value is the community key; `cfg.curse_key_header()`).
- Search: `GET /v1/mods/search?gameId=1&searchFilter={q}&sortField=2&gameVersionTypeId={flavor.curse_type_id()}&index={page*per}&pageSize={per}`
  → `{data:[Mod], pagination:{index,pageSize,resultCount,totalCount}}`.
  Mod → SearchItem{external_id:id, name, author:authors[0].name, summary→description,
  downloadCount, logo→icon (thumbnail URLs are plain fields), links.websiteUrl→site_url}.
- Meta: `GET /v1/mods/{id}` (name, slug, authors, latestFiles, downloadCount,
  latestFilesIndexes, links). Files list: `GET /v1/mods/{id}/files?gameVersionTypeId={}&pageSize=50&index=0`
  (paged; fetch pages until you have >=1 file compatible with the flavor, cap 4 pages).
- File → RemoteFile{file_id:id, displayName→version (fallback fileName),
  releaseType 1→Stable 2→Beta 3→Alpha, fileName, downloadUrl (direct CDN url),
  headers: x-api-key (required on edge.forgecdn.net since 2026-07!),
  flavors: from `sortableGameVersions[].gameVersionTypeId == flavor typeIds` (map typeId
  back to Flavor; if file has NO sortableGameVersions, treat as compatible-any),
  fileDate→date, fileLength→size, hashes[] algo 2=md5 1=sha1,
  changelog: `GET /v1/mods/{m}/files/{f}/changelog` returns `{data:{body:html}}`.
  If no downloadUrl field, `GET /v1/mods/{m}/files/{f}/download-url` → `{data:url}`.
- Scan: collect `req.folders[].curse_fingerprint` (u32) →
  `POST /v1/fingerprints` body `{"fingerprints":[u32...]}` (Content-Type json) →
  `{data:{exactMatches:[{id,file,latestFiles}], partialMatches:[{id,file,latestFiles}],
  partialMatchFingerprints:{modId:[fps]}, unmatchedFingerprints:[]}}`.
  exact/partial → ScanMatch{provider:Curse, external_id:id, folders = all scanned folder
  names whose fingerprint ∈ that mod's module fps (match `file.modules[].fingerprint`
  against folder fingerprints, fall back: mod of every fingerprint of the match),
  meta: build RemoteMeta from `file`+`latestFiles` (fields as above; name from a
  follow-up `GET /v1/mods/{id}` if needed — one batch call `POST /v1/mods {"modIds":[..]}`
  is allowed). Prefer exactMatches; a folder claimed once is never re-claimed.
  Also claim folders whose TOC has `## X-Curse-Project-ID:` via meta().
- `isAvailable:false` files must be filtered out.
- Live smoke (#[ignore]): search "details", meta of 2382 (BigWigs), fingerprint POST with
  [3793833344] → expect mod 2382.

## 2. GitHub (`providers/github.rs`)

- Base `https://api.github.com`. Optional `Authorization: Bearer {github_token}` when set;
  `User-Agent` from cfg. Handle `x-ratelimit-remaining: 0` → Error::provider with
  "GitHub rate limit — set [network] github_token".
- external_id "owner/repo". Meta: `GET /repos/{o}/{r}` + `GET /repos/{o}/{r}/releases?per_page=10`.
- File selection (`pick` override, BigWigs-packager convention):
  1. Asset named `release.json` → `{releases:[{name,version,filename,nolib,
     metadata:[{flavor,interface}]}]}` (flavor strings: forever, mainline, classic, bcc,
     wrath, cata, mists, titan; camelot→forever). Pick entries: !nolib &&
     metadata.flavor == sel.flavor.packager_flavor(); map to asset by `filename`.
  2. Fallback: content_type=="application/zip" (or name ends .zip), exclude names
     containing "-nolib"; flavor token rules: for Forever prefer `-forever.zip`; for
     Retail/mainline prefer NO client suffix; suffixes to exclude for retail:
     `-classic|-vanilla|-bcc|-tbc|-wrath|-wotlkc|-cata|-mists|-titan|-forever.zip`.
  3. Skip draft; prereleases only when channel >= Beta. Sort: published_at desc,
     stable-before-prerelease; scan up to 5 releases.
- Download: use asset `browser_download_url` (public, no octet-stream dance needed).
  If absent use `.../releases/assets/{id}` + Accept: application/octet-stream.
- RemoteFile: file_id = release id ("{release_id}:{asset_id}"), version = release tag_name
  (strip leading v), date published_at, size asset size, flavors from release.json metadata.
- Search: `GET /search/repositories?q={q}+topic:wow+topic:world-of-warcraft&per_page=20`
  is noisy → NOT required; `search()` returns Unsupported with a message telling the user
  to paste a repo URL (the GUI has a dedicated "install from GitHub URL" box).
- Scan: folders with TOC `## X-Source:`/`## X-Website:` matching github.com/{o}/{r} → claim.

## 3. Tukui (`providers/tukui.rs`)

- `GET https://api.tukui.org/v1/addons` (array, cache in a std::sync::Mutex<Option<(Instant, Vec)>>
  for 10 min). Filter by slug for meta().
- Fields: {id, slug, name, author, version, url, patch[], directories[], small_desc, desc,
  web_url, changelog_url, last_update "YYYY-MM-DD"}.
- RemoteFile: file_id = format!("{slug}@{version}"), url (tokenized, FRESH — never cache
  the token; re-GET the list in meta()), version, flavors: derive from patch[] entries
  (parse version: 1.6x→Forever, 12.x/11.x→Retail, 5.5.x→Mists? use ranges: 1.15→ClassicEra,
  2.5→TBC, 3.4→Wotlk, 4.4→Cata, 5.5→Mists, 1.60→Forever, 11/12→Retail; if any patch matches
  sel.flavor → include flavor). date = last_update.
- Scan: TOC `## X-Tukui-ProjectID:` (note: Tukui/ElvUI have negative ids) → claim, folders
  from `## X-Tukui-ProjectFolders:` (csv) else the folder itself.
- Search: client-side over the full list, simple case-insensitive substring in
  name/author/slug; no pagination (return all).

## 4. MMOUI / WoWInterface (`providers/mmoui.rs`)

- `GET https://api.mmoui.com/v4/game/WOW/filedetails/{ids}.json` (comma-joined batch);
  v4 record: {id,title,author,version,downloadUri,changeLog,downloads,lastUpdate,checksum(md5),fileName}.
  v4 has holes → per-id fallback `GET .../v3/game/WOW/filedetails/{id}.json`
  (UID/UIVersion/UIDownload/UIFileName/UIHitCount/UIDate ms).
- RemoteFile: file_id=id, version, url=downloadUri, md5=checksum, date lastUpdate
  (unix ms → s), flavors: [] (provider gives none; filelist has none for Forever).
- Scan: TOC `## X-WoWI-ID:` only. No fingerprint, no search endpoint → search() Unsupported
  (explain: install by pasting a wowinterface file id/url; GUI handles).

## 5. WowUp Hub (`providers/hub.rs`)

- No auth. `GET https://hub.wowup.io/addons/search/{flavor.hub_game_type()}?query={q}&limit=20`
  → `{addons:[{id,repository,repository_name,source,description(html strip),homepage,
  image_url,owner_name,total_download_count,topics,releases[]}]}` (query REQUIRED, min 2 chars).
- Item external_id = id as string. Releases: download_url, tag_name, game_version,
  game_type, prerelease, published_at, addonFolders[] (folder names!), toc_version.
- RemoteFile: file_id tag_name, version toc_version fallback tag, flavors [flavor] from
  game_type map, date published_at, url download_url.
- meta(): `GET /addons/{id}` (single).
- Scan: folders with TOC `## X-Wago-ID:` — NO (that's wago). Scan: name-similarity over a
  search of the folder name (best-effort, threshold >= 0.85 using a simple normalized
  containment) → claim, folders = release.addonFolders ∩ scanned folder names.
- `search()` supported (this is our GitHub-addon discovery source).

## 6. Zip (`providers/zip.rs`)

- external_id = https URL (allowlist: user-added host — engine passes extra_hosts).
- meta(): HEAD/GET the url, RemoteFile{file_id: etag/date/url, version: query-less
  filename, flavors: []}; no API. scan: none. search: none.

## Definition of done (each provider file)

- compiles (`cargo check -p addons-forever-core`), stub body replaced,
- unit fixture tests pass (`cargo test -p addons-forever-core`),
- live smoke tests added behind `#[ignore]`,
- no panics on malformed JSON (log + provider error).
