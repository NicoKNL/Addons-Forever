#!/usr/bin/env bash
# Build the macOS .app bundle + first-run helper from a compiled binary, and
# ad-hoc sign it. Run on macOS (uses the system codesign) or with rcodesign
# installed (Linux cross builds).
#
# Usage: scripts/macos-bundle.sh <target-triple>   (e.g. aarch64-apple-darwin)
set -euo pipefail
cd "$(dirname "$0")/.."

T="${1:?usage: macos-bundle.sh <target-triple>}"
VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')
BIN="target/$T/release/addons-forever"
[ -x "$BIN" ] || { echo "missing $BIN (build it first)"; exit 1; }
mkdir -p dist
base="dist/.bundle-$T"
app="$base/AddonsForever.app"
rm -rf "$base"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$BIN" "$app/Contents/MacOS/addons-forever"
chmod 755 "$app/Contents/MacOS/addons-forever"
cp assets/app.icns "$app/Contents/Resources/app.icns"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Addons Forever</string>
  <key>CFBundleDisplayName</key><string>Addons Forever</string>
  <key>CFBundleExecutable</key><string>addons-forever</string>
  <key>CFBundleIdentifier</key><string>dev.local.addons-forever</string>
  <key>CFBundleIconFile</key><string>app.icns</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSHumanReadableCopyright</key><string>No telemetry. Independent project, not affiliated with Blizzard Entertainment.</string>
</dict></plist>
PLIST
printf 'APPL????' > "$app/Contents/PkgInfo"
cat > "$base/Launch.command" <<'CMD'
#!/bin/sh
# One-time setup: clear the quarantine flag macOS sets on downloads, re-sign
# locally, launch.
cd "$(dirname "$0")"
app=$(ls -d *.app 2>/dev/null | head -n 1)
if [ -z "$app" ]; then echo "No .app found next to this script."; read; exit 1; fi
if [ ! -f "$app/Contents/MacOS/addons-forever" ]; then
  echo "The bundle $app has no executable inside: re-extract the archive and"
  echo "run this script before opening the app."
  read; exit 1
fi
xattr -rc "$app"
codesign --force --deep -s - "$app" >/dev/null 2>&1
if ! open "$app"; then
  echo "Finder refused; starting the binary directly..."
  "$app/Contents/MacOS/addons-forever"
fi
CMD
chmod 755 "$base/Launch.command"
cat > "$base/MACOS-INSTALL.txt" <<TXT
Installing AddonsForever.app
============================
Private build: ad-hoc signed, not notarized. macOS marks downloaded apps as
quarantined. Run Launch.command once (double-click), or in Terminal:

  xattr -cr *.app
  open *.app
TXT

# Sign: Apple's codesign on macOS runners, rcodesign on Linux cross builds.
if [ "$(uname -s)" = "Darwin" ]; then
  codesign --force --deep -s - "$app"
elif command -v rcodesign >/dev/null 2>&1; then
  rcodesign sign --binary-identifier dev.local.addons-forever "$app" "$app"
  echo "signed with rcodesign"
else
  echo "WARNING: no signing tool; the .app will not launch on Apple Silicon"
fi

out="dist/addons-forever-$VERSION-$T-macos.zip"
rm -f "$out"
( cd "$base" && zip -qryX "$OLDPWD/$out" AddonsForever.app Launch.command MACOS-INSTALL.txt )
[ -f README.md ] && ( cd dist && zip -q "$out" ../README.md ) 2>/dev/null || true
tar -C "$base" -czf "dist/addons-forever-$VERSION-$T-macos.tar.gz" AddonsForever.app Launch.command MACOS-INSTALL.txt
rm -rf "$base"
ls -la "$out"
