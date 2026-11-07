#!/usr/bin/env bash
# Build portable release artifacts into dist/ (zips + SHA256SUMS).
#
# Requirements per target:
#   x86_64-pc-windows-gnu   : cargo-zigbuild + zig + a windres (x86_64-w64-mingw32-windres
#                             on PATH, or set WINDRES=...). The DPI manifest build script
#                             needs it; without it the exe builds but lacks PerMonitorV2.
#   *-apple-darwin          : cargo-zigbuild + zig + a macOS SDK (export SDKROOT=MacOSX.sdk)
#   native linux target     : plain cargo
#
# All targets build CONCURRENTLY (each with a slice of the CPU cores); the
# per-target dependency graphs are fully separate, so serial builds waste cores.
#
# Usage:
#   ./scripts/build-release.sh x86_64-pc-windows-gnu x86_64-unknown-linux-gnu \
#       aarch64-apple-darwin x86_64-apple-darwin
set -euo pipefail
cd "$(dirname "$0")/.."

NPROC=$(nproc)

VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)"/\1/')
mkdir -p dist
# Only clean artifacts for the targets we are (re)building, so a partial
# invocation does not wipe the other targets' archives.
for t in "$@"; do rm -f "dist/addons-forever-$VERSION-$t-macos.zip" "dist/addons-forever-$VERSION-$t-macos.tar.gz" "dist/addons-forever-$VERSION-$t.zip" "dist/addons-forever-$VERSION-$t.tar.gz"; done

build_target() {
  local t="$1" jobs="$2"
  echo "==> building $t (-j $jobs)"
  case "$t" in
    *"-windows-gnu"|*"-apple-"*)
      cargo zigbuild --release -j "$jobs" --target "$t" -p addons-forever-gui ;;
    *)
      cargo build --release -j "$jobs" --target "$t" -p addons-forever-gui ;;
  esac
  local dir="target/$t/release"
  case "$t" in
    *"-apple-"*)
      # Deliver a real .app bundle: Info.plist + icon (assets/app.icns) + binary.
      python3 - "$t" "$VERSION" <<'PY'
import sys, os, shutil, zipfile
t, v = sys.argv[1], sys.argv[2]
base = f"dist/.bundle-{t}"
app = f"{base}/AddonsForever.app"
shutil.rmtree(base, ignore_errors=True)
shutil.rmtree(app, ignore_errors=True)
os.makedirs(f"{app}/Contents/MacOS", exist_ok=True)
os.makedirs(f"{app}/Contents/Resources", exist_ok=True)
shutil.copy2(f"target/{t}/release/addons-forever", f"{app}/Contents/MacOS/addons-forever")
os.chmod(f"{app}/Contents/MacOS/addons-forever", 0o755)
shutil.copy2("assets/app.icns", f"{app}/Contents/Resources/app.icns")
plist = f"""<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Addons Forever</string>
  <key>CFBundleDisplayName</key><string>Addons Forever</string>
  <key>CFBundleExecutable</key><string>addons-forever</string>
  <key>CFBundleIdentifier</key><string>dev.local.addons-forever</string>
  <key>CFBundleIconFile</key><string>app.icns</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>{v}</string>
  <key>CFBundleVersion</key><string>{v}</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSHumanReadableCopyright</key><string>Private LAN build. No telemetry.</string>
</dict></plist>
"""
open(f"{app}/Contents/Info.plist", "w").write(plist)
open(f"{app}/Contents/PkgInfo", "w").write("APPL????")
# First-run helper: removes the quarantine xattr macOS sets on LAN downloads
# (Sequoia/Tahoe report quarantined ad-hoc signed apps as "damaged").
# Uses a glob so a renamed bundle (some archivers rename from the zip name)
# still works.
launch = """#!/bin/sh
# One-time setup for the private LAN build: clear the quarantine flag,
# re-sign locally with Apple's own tool, launch. Prints diagnostics on failure.
cd "$(dirname "$0")"
app=$(ls -d *.app 2>/dev/null | head -n 1)
if [ -z "$app" ]; then echo "No .app found next to this script."; read; exit 1; fi
if [ ! -f "$app/Contents/MacOS/addons-forever" ]; then
  echo "The bundle $app has no executable inside. macOS deleted it after a"
  echo "refused launch: re-extract a fresh copy of the archive, then run this"
  echo "script BEFORE opening the app."
  read; exit 1
fi
xattr -rc "$app"
echo "re-signing locally (ad-hoc)..."
codesign --force --deep -s - "$app" 2>&1 | tail -n 2
if open "$app"; then
  echo "launched. AddonsForever will start in a moment."
else
  echo "Finder refused to open the bundle; starting the binary directly..."
  "$app/Contents/MacOS/addons-forever"
fi
"""
open(f"{base}/Launch.command", "w").write(launch)
os.chmod(f"{base}/Launch.command", 0o755)
install = """Installing AddonsForever.app on this Mac
=========================================
This is a private LAN build: ad-hoc signed, not notarized. macOS marks
downloaded files "quarantined" and refuses to open such apps.

Easiest:
  Double-click Launch.command in this folder (first run only).
  If macOS blocks the script itself, use Terminal:

  cd <this folder>
  xattr -cr *.app
  open *.app

Then the app launches normally every time (the quarantine flag is gone).
If the app window flashes and quits, send us:
  sw_vers -productVersion ; uname -m
  spctl -a -vv *.app
"""
open(f"{base}/MACOS-INSTALL.txt", "w").write(install)
os.chmod(f"{base}/MACOS-INSTALL.txt", 0o644)
PY
      # Ad-hoc code signature: Apple Silicon refuses to launch unsigned Mach-O
      # binaries (generic "cannot be opened" error). rcodesign signs the bundle
      # recursively on Linux.
      if command -v rcodesign >/dev/null 2>&1; then
        echo "==> ad-hoc signing AddonsForever.app for $t"
        rcodesign sign --binary-identifier dev.local.addons-forever \
            "dist/.bundle-$t/AddonsForever.app" "dist/.bundle-$t/AddonsForever.signed.app"
        rm -rf "dist/.bundle-$t/AddonsForever.app"
        mv "dist/.bundle-$t/AddonsForever.signed.app" "dist/.bundle-$t/AddonsForever.app"
      else
        echo "==> WARNING: rcodesign not found: shipping UNSIGNED .app (will not launch on Apple Silicon)"
      fi
      python3 - "$t" "$VERSION" <<'PY2'
import sys, os, zipfile
t, v = sys.argv[1], sys.argv[2]
out = f"dist/addons-forever-{v}-{t}-macos.zip"

def add_tree(z: zipfile.ZipFile, path: str, root: str):
    # directory entries + real st_mode (incl. S_IFREG/S_IFDIR): macOS's expander
    # is picky, and Gatekeeper treats odd modes as "damaged".
    entries = []
    for r, dirs, files in os.walk(path):
        for d in dirs:
            entries.append(os.path.join(r, d))
        for f in files:
            entries.append(os.path.join(r, f))
    for pth in sorted(entries, key=lambda q: (os.path.isdir(q) is False, q)):
        rel = os.path.relpath(pth, root)
        mode = os.stat(pth).st_mode
        if os.path.isdir(pth):
            zi = zipfile.ZipInfo(rel + "/")
            zi.external_attr = (0o40755) << 16
            z.writestr(zi, b"")
        else:
            zi = zipfile.ZipInfo(rel)
            zi.external_attr = ((mode & 0o7777) | 0o100000) << 16
            zi.compress_type = zipfile.ZIP_DEFLATED
            with open(pth, "rb") as fh:
                z.writestr(zi, fh.read())

with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    add_tree(z, f"dist/.bundle-{t}/AddonsForever.app", f"dist/.bundle-{t}")
    for doc in (f"dist/.bundle-{t}/Launch.command", f"dist/.bundle-{t}/MACOS-INSTALL.txt"):
        if os.path.exists(doc):
            z.write(doc, os.path.basename(doc))
    for doc in ("README.md", "PRIVACY.md"):
        if os.path.exists(doc):
            z.write(doc, doc)
with zipfile.ZipFile(out) as z:
    bad = z.testzip()
    assert bad is None, f"corrupt zip entry: {bad}"
print("  +", out, f"{os.path.getsize(out)/1e6:.1f} MB")
# tar.gz variant: tar preserves modes exactly and Finder names the bundle
# AddonsForever.app on expand (no zip-name renaming).
targz = f"dist/addons-forever-{v}-{t}-macos.tar.gz"
PY2
      tar -C "dist/.bundle-$t" -czf "dist/addons-forever-$VERSION-$t-macos.tar.gz" AddonsForever.app Launch.command MACOS-INSTALL.txt
      echo "  + dist/addons-forever-$VERSION-$t-macos.tar.gz"
      rm -rf "dist/.bundle-$t"
      ;;
    *windows*)
      python3 - "$t" "$VERSION" <<'PY'
import sys, zipfile, os
t, v = sys.argv[1], sys.argv[2]
src = f"target/{t}/release/addons-forever"
if t.endswith("windows-gnu"):
    src += ".exe"
out = f"dist/addons-forever-{v}-{t}.zip"
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as z:
    z.write(src, os.path.basename(src))
    for doc in ("README.md", "PRIVACY.md"):
        if os.path.exists(doc):
            z.write(doc, doc)
print("  +", out, f"{os.path.getsize(out)/1e6:.1f} MB")
PY
      ;;
    *)
      local out="dist/addons-forever-$VERSION-$t.tar.gz"
      tar -czf "$out" -C "$dir" addons-forever -C "$PWD" README.md PRIVACY.md
      echo "  + $out"
      ;;
  esac
}

TARGETS=("$@")
NT=${#TARGETS[@]}
JOBS=$(( NPROC / (NT > 0 ? NT : 1) ))
JOBS=$(( JOBS > 0 ? JOBS : 1 ))
mkdir -p dist/logs
pids=()
for target in "${TARGETS[@]}"; do
  build_target "$target" "$JOBS" >"dist/logs/$target.log" 2>&1 &
  pids+=("$!")
done
fail=0
for i in "${!TARGETS[@]}"; do
  if ! wait "${pids[$i]}"; then
    echo "!! ${TARGETS[$i]} FAILED — tail of dist/logs/${TARGETS[$i]}.log:"
    tail -25 "dist/logs/${TARGETS[$i]}.log"
    fail=1
  else
    echo "==> ${TARGETS[$i]} ok (log: dist/logs/${TARGETS[$i]}.log)"
  fi
done
[ "$fail" -eq 0 ] || exit 1

( cd dist && sha256sum addons-forever-* > SHA256SUMS && cat SHA256SUMS )
echo "done."
