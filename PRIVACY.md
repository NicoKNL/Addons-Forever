# Privacy policy: Addons Forever

Addons Forever collects nothing. No telemetry, no analytics, no crash reporting, no
ads, no accounts, no background connections. This file is a contract: any change
that weakens it is a bug.

## Network traffic

The app contacts an addon provider's public API only for actions you asked for:
scanning (identifying your installed addons), search, update checks, downloads,
dependency lookups, and app self-update checks. Offline mode (`[network] offline`,
a Settings checkbox) is a hard boundary: with it on, no code path makes a request,
and the app works from its local cache. There is no first-party server for this
project, not even a website.

The HTTP layer enforces a domain allowlist. A request to any host outside this list
is refused before it leaves the process:

| Domain | Purpose |
|---|---|
| `api.curseforge.com`, `edge.forgecdn.net`, `mediafilez.forgecdn.net` | CurseForge search, download, and fingerprint matching (see below) |
| `api.github.com`, `github.com`, `objects.githubusercontent.com`, `release-assets.githubusercontent.com`, `raw.githubusercontent.com`, `media.githubusercontent.com`, `codeload.github.com` | GitHub-hosted addons, app self-update |
| `api.tukui.org`, `downloads.tukui.org` | Tukui addons |
| `api.mmoui.com`, `cdn.wowinterface.com` | MMOUI / WoWInterface addons |
| `hub.wowup.io` | WowUp Hub index (folder-name lookups, see below) |
| hosts of zip URLs you add yourself | `zip URL` installs: the URL you type authorizes exactly that host |

What the functional requests disclose:

- **Scanning** computes fingerprints of your installed addon folders. CurseForge
  matching sends a one-way hash per folder (its content fingerprint, not the
  files themselves) and receives the matching addon ids back. WowUp Hub matching
  sends folder names as search queries. Providers learn which addons (and which
  versions) you have, the same as any download would.
- **Searches** send your query text; **update checks** send addon ids.

These are normal web requests: the provider logs them like any request, including
your IP address. Beyond that, the app sends a static `User-Agent`
(`addons-forever/<version>`) with no machine identifier, path, locale, or anything
unique. The app has no client id.

The one automatic startup request is the check for a new version of the app itself
on GitHub Releases, sent to `api.github.com` for the configured `update_repo`
(default: this project's releases). It discovers a new version only: nothing
downloads and nothing is installed unless you press the update button, and
enabling automatic installs is your choice in Settings. Turn the check off by
clearing `update_repo` in the config, setting `[network] offline = true`, or any
ad block: it is one plain API call to GitHub, and GitHub logs it like any
request, including your IP address.

## What is stored, and where

The app is portable: everything lives next to the executable.

- `addons-forever.ini`: your settings and per-addon choices (auto-update, channel,
  ignored, source pins). This is the only permanent file outside the game folder.
- `.single-instance-*.lock` in the data folder: a small lock file that keeps two
  copies of the app from mutating the same installation at once. It is deleted on
  exit.
- `state.json.migrated`: kept only after a one-time migration from the pre-0.2.7
  on-disk layout, as a backup of your old state. Safe to delete.
- In the game folder, next to `AddOns`: `.fa-tmp/` is the app's scratch directory.
  Downloaded archives and extracted trees live there only during an install (they
  are deleted on completion), and `backup/` holds the snapshots used to roll back a
  failed update. With backup retention on (Settings), the newest few snapshots per
  addon are kept; with it off, they are deleted after a successful install.
  Deleting `.fa-tmp` at any time is safe.
- Addon state is kept in memory and rebuilt from a scan of your AddOns folder at
  startup. No log files, no caches, no hidden data. The Log tab can export its
  contents to a file when you ask it to.

## Credentials you provide

- A CurseForge API key (a public community key is the default; you can paste your
  own). Sent only to CurseForge domains.
- A GitHub token (optional, raises rate limits). Sent only to `api.github.com`.

Nothing is required to run the app, and every key stays in the local `.ini`.

## No-install by design

The program is a single executable. It writes no registry keys, registers no
services, and touches no files outside its own folder, the `.fa-tmp` scratch
directory beside your `AddOns` folder, and the `AddOns` folder itself, which is
its entire job.
