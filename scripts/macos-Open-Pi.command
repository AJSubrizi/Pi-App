#!/bin/bash
# Double-click this from the Pi DMG (or after copy) to install & open on macOS.
# Needed because release builds are not Apple-notarized (no paid Developer ID).
set -euo pipefail

APP_SRC=""
# Prefer sibling Pi.app (when run from DMG volume)
DIR="$(cd "$(dirname "$0")" && pwd)"
if [[ -d "$DIR/Pi.app" ]]; then
  APP_SRC="$DIR/Pi.app"
elif [[ -d "/Volumes/Pi/Pi.app" ]]; then
  APP_SRC="/Volumes/Pi/Pi.app"
fi

DEST="/Applications/Pi.app"

echo ""
echo "  Pi — macOS install helper"
echo "  ─────────────────────────"
echo ""

if [[ -z "$APP_SRC" ]]; then
  echo "Could not find Pi.app next to this script."
  echo "Open the DMG, then double-click “Open Pi.command” inside it."
  echo ""
  read -r -p "Press Enter to close…"
  exit 1
fi

echo "Copying Pi to Applications…"
rm -rf "$DEST"
cp -R "$APP_SRC" "$DEST"

echo "Clearing download quarantine…"
xattr -cr "$DEST" 2>/dev/null || true

echo "Applying local ad-hoc signature…"
codesign --force --deep --sign - --timestamp=none --identifier "dev.pi.desktop" "$DEST" 2>/dev/null || true

echo "Opening Pi…"
if open "$DEST" 2>/dev/null; then
  echo ""
  echo "If macOS still blocks the app:"
  echo "  1. System Settings → Privacy & Security"
  echo "  2. Scroll to the message about Pi → Open Anyway"
  echo "  3. Confirm Open"
  echo ""
else
  echo ""
  echo "Gatekeeper blocked the open. Do this once:"
  echo "  System Settings → Privacy & Security → Open Anyway (Pi)"
  echo ""
  echo "Or run:"
  echo "  xattr -cr /Applications/Pi.app && open /Applications/Pi.app"
  echo ""
fi

read -r -p "Press Enter to close this window…"
