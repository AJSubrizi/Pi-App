#!/usr/bin/env python3
"""Merge one platform entry into a Tauri updater manifest (latest.json).

The macOS updater bundle is rebuilt after `tauri-action` runs (see
scripts/macos-fix-sign-dmg.sh), so its signature has to be patched into the
manifest afterwards. Merging — rather than rewriting — keeps the entries the
Linux/Windows jobs already published.

Usage:
  patch-updater-manifest.py <manifest> <platform-key> <sig-file> <url> <version>
"""

import json
import sys
from pathlib import Path


def main(argv: list[str]) -> int:
    if len(argv) != 6:
        print(__doc__, file=sys.stderr)
        return 2

    manifest, key, sig_file, url, version = argv[1:]
    path = Path(manifest)

    data = {}
    if path.is_file():
        try:
            data = json.loads(path.read_text() or "{}") or {}
        except json.JSONDecodeError:
            # A truncated download must not silently drop existing platforms.
            print(f"error: {manifest} is not valid JSON", file=sys.stderr)
            return 1

    data.setdefault("version", version)
    platforms = data.setdefault("platforms", {})
    platforms[key] = {
        "signature": Path(sig_file).read_text().strip(),
        "url": url,
    }

    path.write_text(json.dumps(data, indent=2) + "\n")
    print(f"{manifest}: set {key} ({len(platforms)} platforms total)")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
