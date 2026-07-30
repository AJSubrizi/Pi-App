#!/usr/bin/env bash
# Re-sign Pi.app with a proper ad-hoc signature (includes Resources) and rebuild .dmg.
# Unsigned/broken linker-signed binaries are treated as "damaged" on modern macOS
# and moved to Trash after download.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

sign_app() {
  local app="$1"
  echo "==> codesign ad-hoc: $app"
  # Remove any broken signature first
  codesign --remove-signature "$app" 2>/dev/null || true
  # Deep ad-hoc sign with explicit entitlements-free bundle seal
  codesign \
    --force \
    --deep \
    --sign - \
    --timestamp=none \
    --identifier "dev.pi.desktop" \
    "$app"
  codesign --verify --deep --strict --verbose=2 "$app"
  codesign -dv --verbose=2 "$app" 2>&1 | head -20 || true
}

# Rebuild the updater bundle from the *re-signed* app.
#
# tauri-action produces Pi.app.tar.gz + .sig from the app as the linker left it,
# i.e. before sign_app() seals Resources. Shipping that tarball would make the
# updater install exactly the "damaged" bundle this script exists to repair, so
# the tarball and its signature are regenerated here and re-uploaded.
rebuild_updater_bundle() {
  local app="$1"
  local outdir="$2"
  if [[ -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]]; then
    echo "==> skip updater bundle: TAURI_SIGNING_PRIVATE_KEY unset"
    return 0
  fi
  mkdir -p "$outdir"
  local tgz="$outdir/Pi.app.tar.gz"
  rm -f "$tgz" "$tgz.sig"
  echo "==> updater bundle: $tgz"
  # Tauri expects the .app at the tarball root.
  tar czf "$tgz" -C "$(dirname "$app")" "$(basename "$app")"
  # `-p` must be passed even for an unencrypted key: without it the signer opens
  # an interactive password prompt, which on a CI runner fails immediately with
  # "Device not configured (os error 6)".
  #
  # A signing failure must not abort the release: the DMG is already built and
  # is uploaded by the caller after this returns. Losing auto-update for one
  # release is recoverable; losing the download is not.
  if ! pnpm --silent tauri signer sign \
    --password "${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}" \
    "$tgz"; then
    echo "warning: updater signing failed; DMG still ships, auto-update skipped" >&2
    rm -f "$tgz" "$tgz.sig"
    return 0
  fi
  if [[ ! -f "$tgz.sig" ]]; then
    echo "warning: signer wrote no $tgz.sig; auto-update skipped" >&2
    rm -f "$tgz"
    return 0
  fi
}

rebuild_dmg() {
  local app="$1"
  local dmg_out="$2"
  local volname="${3:-Pi}"
  local tmp
  local verify_mount=""
  tmp="$(mktemp -d)"
  cleanup_rebuild() {
    if [[ -n "$verify_mount" ]]; then
      hdiutil detach "$verify_mount" -quiet || true
    fi
    rm -rf "$tmp"
  }
  trap cleanup_rebuild RETURN
  mkdir -p "$tmp/stage"
  cp -R "$app" "$tmp/stage/Pi.app"
  ln -s /Applications "$tmp/stage/Applications"
  cat > "$tmp/stage/START HERE.txt" <<'EOF'
Pi for macOS
============

These builds are not Apple-notarized (community MIT app), so macOS blocks
them on first launch. Clearing the download quarantine flag is the one
method that still works on current macOS.

1. Drag Pi.app onto the Applications shortcut.
2. Open Terminal (Applications > Utilities).
3. Paste this line and press Return:

     xattr -dr com.apple.quarantine /Applications/Pi.app

4. Open Pi normally from Applications.

You only do this once. After the first launch Pi opens like any other app.

Note: right-clicking and choosing Open no longer works. Apple removed that
override in macOS 15 Sequoia, and macOS 26 Tahoe tightened it further, so
older instructions that mention it will fail. System Settings >
Privacy & Security > "Open Anyway" works only sometimes; the command above
is reliable.

Prefer not to run a command? Build from source instead - locally built
apps are never quarantined:

     git clone https://github.com/AJSubrizi/Pi-App && cd Pi-App
     pnpm install && pnpm build
EOF
  rm -f "$dmg_out"
  hdiutil create \
    -volname "$volname" \
    -srcfolder "$tmp/stage" \
    -ov \
    -format UDZO \
    "$dmg_out"
  verify_mount="$(
    hdiutil attach -nobrowse -readonly "$dmg_out" |
      sed -nE 's|^.*(/Volumes/.*)$|\1|p' |
      tail -1
  )"
  [[ -d "$verify_mount/Pi.app" ]] || {
    echo "error: rebuilt DMG does not expose Pi.app" >&2
    hdiutil detach "$verify_mount" -quiet || true
    return 1
  }
  [[ -L "$verify_mount/Applications" ]] || {
    echo "error: rebuilt DMG is missing Applications shortcut" >&2
    hdiutil detach "$verify_mount" -quiet || true
    return 1
  }
  codesign --verify --deep --strict "$verify_mount/Pi.app"
  hdiutil detach "$verify_mount" -quiet
  verify_mount=""
  echo "==> wrote $dmg_out"
}

ver="$(python3 -c 'import json; print(json.load(open("package.json"))["version"])')"

# tauri-action removes the .app after producing the DMG. When the bundle still
# exists, sign it directly. Otherwise, extract it from the generated DMG first.
APPS="$(find src-tauri/target -type d -name 'Pi.app' 2>/dev/null | sort -u)"
if [[ -n "$APPS" ]]; then
  echo "$APPS" | while IFS= read -r app; do
    [[ -n "$app" ]] || continue
    sign_app "$app"
    # Sibling dmg dir used by tauri: .../bundle/dmg and .../bundle/macos
    bundle_dir="$(cd "$(dirname "$app")/.." && pwd)"
    dmg_dir="$bundle_dir/dmg"
    mkdir -p "$dmg_dir"
    arch="universal"
    case "$app" in
      *aarch64-apple-darwin*) arch="aarch64" ;;
      *x86_64-apple-darwin*) arch="x64" ;;
    esac
    out="$dmg_dir/Pi_${ver}_${arch}.dmg"
    rebuild_dmg "$app" "$out" "Pi"
    rebuild_updater_bundle "$app" "$bundle_dir/macos"
  done
else
  DMGS="$(find src-tauri/target -type f -name 'Pi_*.dmg' 2>/dev/null | sort -u)"
  if [[ -z "$DMGS" ]]; then
    echo "error: no Pi.app or Pi_*.dmg found under src-tauri/target" >&2
    exit 1
  fi

  echo "$DMGS" | while IFS= read -r dmg; do
    [[ -n "$dmg" ]] || continue
    tmp="$(mktemp -d)"
    mount_point=""
    cleanup_extract() {
      if [[ -n "$mount_point" ]]; then
        hdiutil detach "$mount_point" -quiet || true
      fi
      rm -rf "$tmp"
    }
    trap cleanup_extract EXIT

    mount_point="$(
      hdiutil attach -nobrowse -readonly "$dmg" |
        sed -nE 's|^.*(/Volumes/.*)$|\1|p' |
        tail -1
    )"
    mounted_app="$mount_point/Pi.app"
    if [[ ! -d "$mounted_app" ]]; then
      mounted_app="$mount_point/.payload/Pi.app"
    fi
    if [[ -z "$mount_point" || ! -d "$mounted_app" ]]; then
      echo "error: Pi.app not found inside $dmg" >&2
      exit 1
    fi
    cp -R "$mounted_app" "$tmp/Pi.app"
    hdiutil detach "$mount_point" -quiet
    mount_point=""

    sign_app "$tmp/Pi.app"
    rebuild_dmg "$tmp/Pi.app" "$dmg" "Pi"
    rebuild_updater_bundle "$tmp/Pi.app" "$(dirname "$dmg")/../macos"
    cleanup_extract
    trap - EXIT
  done
fi

echo "OK — macOS ad-hoc sign + dmg rebuild complete"
