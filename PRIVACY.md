# Privacy policy: Addons Forever

Addons Forever collects nothing. No telemetry, no analytics, no crash reporting, no
ads, no accounts, no background connections. This file is a contract: any change
that weakens it is a bug.

## Network traffic

The app contacts the public API of an addon provider only when you asked it to
fetch something: a search, an update check, a download. There is no first-party
server for this project, not even a website.

The HTTP layer enforces a domain allowlist. A request to any host outside this list
is refused before it leaves the process:

| Domain | Purpose |
|---|---|
| `api.curseforge.com`, `edge.forgecdn.net`, `mediafilez.forgecdn.net` | CurseForge search and download |
| `api.github.com`, `github.com`, `objects.githubusercontent.com`, `release-assets.githubusercontent.com`, `raw.githubusercontent.com`, `media.githubusercontent.com`, `codeload.github.com` | GitHub-hosted addons, app self-update |
| `api.tukui.org`, `downloads.tukui.org` | Tukui addons |
| `api.mmoui.com`, `cdn.wowinterface.com` | MMOUI / WoWInterface addons |
| `hub.wowup.io` | WowUp Hub index |
| `addons.wago.io` | Wago Addons, only if you paste your own API key |
| any host | `zip URL` installs you add yourself |

These are normal web requests: the provider logs them like any download, including
your IP address. That is the full extent of what anyone learns. The app sends a
static `User-Agent` (`addons-forever/<version>`) with no machine identifier, path,
locale, or anything unique. The app has no client id.

The one background request is the optional, opt-in check for a new version of the
app itself on GitHub Releases. It is off until you set `update_repo` in the config.

## What is stored, and where

The app is portable: everything lives next to the executable.

- `addons-forever.ini`: your settings and per-addon choices (auto-update, channel,
  ignored, source pins). The only file the app writes.
- Addon state is kept in memory and rebuilt from a scan of your AddOns folder at
  startup. No log files, no caches, no hidden data. The Log tab can export its
  contents to a file when you ask it to.

## Credentials you provide

- A CurseForge API key (a public community key is the default; you can paste your
  own). Sent only to CurseForge domains.
- A GitHub token (optional, raises rate limits). Sent only to `api.github.com`.
- A Wago API key (optional). Sent only to `addons.wago.io`.

Nothing is required to run the app, and every key stays in the local `.ini`.

## No-install by design

The program is a single executable. It writes no registry keys, registers no
services, and touches no files outside its own folder and your `AddOns` directory,
which is its entire job.
