# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2] - 2026-07-28

> **Highlight:** macOS DMG opens without “damaged / move to Trash” (fixed ad-hoc codesign).

### Fixed

- **macOS packaging** — re-sign `Pi.app` with a full ad-hoc signature (sealed Resources) and rebuild the DMG in CI.
- README install steps for Gatekeeper / quarantine.

## [0.2.1] - 2026-07-28

> **Highlight:** update links always open **AJSubrizi/Pi-App**.

### Fixed

- **App update open URL** — only Pi-App release pages; prefer direct `Pi_*.dmg` / setup for this OS/arch.
- Startup banner and Settings → About use the same open helper.

## [0.2.0] - 2026-07-28

> **Highlight:** first public **Pi** desktop release — GUI for [Pi](https://pi.dev) over JSONL RPC.

### Added

- **Pi RPC runtime** — local sessions via `pi --mode rpc`.
- **Pi packages center** — install / update / remove packages (npm, git, local path).
- **Workbench panels as tabs** — Changes, Browser, Files.
- **Git Changes panel** — stage/unstage, discard, Commit & Push, numstat badges.
- **Startup update check** — soft banner when a newer GitHub release exists.
- **Pi branding** — product name Pi, dock icon, MIT README.

### Notes

- Unofficial sister project to pi.dev. Requires a working Pi CLI.
- Installers do not auto-install updates — open the release page to download.
