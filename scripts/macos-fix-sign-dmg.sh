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

rebuild_dmg() {
  local app="$1"
  local dmg_out="$2"
  local volname="${3:-Pi}"
  local tmp
  tmp="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$tmp'" RETURN
  mkdir -p "$tmp/stage"
  cp -R "$app" "$tmp/stage/Pi.app"
  ln -s /Applications "$tmp/stage/Applications"
  # Helper for Gatekeeper: double-click instead of dragging bare .app
  cp "$ROOT/scripts/macos-Open-Pi.command" "$tmp/stage/Open Pi.command"
  chmod +x "$tmp/stage/Open Pi.command"
  # Short readme on the volume
  cat > "$tmp/stage/README-macOS.txt" <<'EOF'
Pi for macOS
============

These builds are not Apple-notarized (community MIT app).

Recommended:
  1. Double-click "Open Pi.command"
  2. If macOS blocks Pi: System Settings → Privacy & Security → Open Anyway

Or drag Pi.app to Applications, then in Terminal:
  xattr -cr /Applications/Pi.app
  open /Applications/Pi.app
EOF
  rm -f "$dmg_out"
  hdiutil create \
    -volname "$volname" \
    -srcfolder "$tmp/stage" \
    -ov \
    -format UDZO \
    "$dmg_out"
  echo "==> wrote $dmg_out"
}

# Prefer release / target triple bundles from tauri build
APPS="$(find src-tauri/target -type d -name 'Pi.app' 2>/dev/null | sort -u)"
if [[ -z "$APPS" ]]; then
  echo "error: no Pi.app found under src-tauri/target" >&2
  find src-tauri/target -name '*.app' 2>/dev/null | head -20 || true
  exit 1
fi

ver="$(python3 -c 'import json; print(json.load(open("package.json"))["version"])')"
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
done

echo "OK — macOS ad-hoc sign + dmg rebuild complete"

