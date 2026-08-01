# Pi desktop - end-to-end QA report

Date: 2026-08-01
Branch: `main`
Target: macOS Apple Silicon, installed `/Applications/Pi.app`
Scope: build, automated suites, packaging, install, launch smoke test, runtime diagnostics

## Health score

**78/100 - healthy alpha build with release and environment blockers**

The application bundle launches and the distributable mounts correctly. Full click-through of webview controls could not be completed because macOS denied Accessibility control to `osascript`; the limitation is recorded rather than treated as an app failure.

## Checks passed

| Area | Result | Evidence |
| --- | --- | --- |
| TypeScript | PASS | `pnpm typecheck` |
| Frontend tests | PASS | 72 files, 705 tests |
| Rust tests | PASS | 290 passed, 1 ignored |
| Rust formatting | PASS | `cargo fmt --check` |
| UI production build | PASS | Vite production build completed |
| Tauri bundle | PASS with signing caveat | `Pi.app`, DMG and updater archive generated |
| DMG mount | PASS | `/Volumes/Pi/Pi.app`, identifier `dev.pi.desktop`, version `0.2.12` |
| Architecture | PASS | DMG app executable is `arm64` |
| Installed launch | PASS | `/Applications/Pi.app` process remained alive after restart |
| Code signature integrity | PASS | `codesign --verify --deep --strict` valid on disk |
| Product reference scan | PASS | no `grok-app` references found outside build/dependency output |

Screenshot evidence: `screenshots/initial.png`.

## Findings

### QA-001 - updater signing key missing (High for release, not runtime)

`pnpm tauri build` creates all bundles but exits with code 1 because `TAURI_SIGNING_PRIVATE_KEY` is not set. Automatic update artifacts cannot be signed in this environment. The DMG and `.app` are still usable.

### QA-002 - Clippy strict gate fails on existing code (Medium)

`cargo clippy --all-targets -- -D warnings` reports five errors: complex return type, too many arguments in three commands, and an owned comparison in `process_util.rs`. Unit tests and compilation pass, but a strict lint CI gate would fail.

### QA-003 - app is ad-hoc signed / not notarized (Medium for downloads)

The bundle passes local `codesign --verify`, but macOS logs identify it as ad-hoc signed and the build skips notarization because Apple credentials are absent. This is expected for the alpha workflow, but external users may see Gatekeeper warnings.

### QA-004 - model round-trip blocked by provider entitlement in this environment (Medium, external)

The installed Pi CLI reports `AccessDenied.Unpurchased` for the configured model. This prevents validating a real assistant response end-to-end here; it indicates an account/model entitlement issue rather than a packaging crash.

### QA-005 - UI click-through requires Accessibility permission (Test limitation)

The app window renders correctly, but macOS returned `-25211` when `osascript` attempted to click webview controls. Settings, Skills, Packages and composer interactions therefore remain covered by automated tests and visual smoke capture, not by automated pointer clicks in this run.

### QA-006 - large frontend chunks (Low performance risk)

Vite reports post-minification chunks above 500 kB, including the main bundle and OfficeDocumentPreview. The build is valid, but code splitting would improve first-load performance.

### QA-007 - Full access still shows a red warning (High if confirmed during execution)

The live composer screenshot shows `Full access` with a red warning icon. Because pointer automation was blocked by macOS Accessibility permissions, this run could not submit a command and verify whether the warning is only visual or whether an actual permission prompt still interrupts execution. This should be the first manual regression check.

## Runtime observations

- Pi opens with the expected landscape background and an existing workspace/session.
- The process survives restart and remains resident.
- macOS logs contain WebKit/AppKit sandbox and App Intents noise typical of an ad-hoc desktop build; no Pi panic or process crash was observed.

## Recommended next checks

1. Configure `TAURI_SIGNING_PRIVATE_KEY` and notarization credentials in CI, then rerun the release build.
2. Grant Accessibility permission to the test runner and repeat pointer-level checks for Settings, Skills, Packages, composer, terminal, onboarding and updater flows.
3. Repeat the model send/receive test with a provider/model enabled for the test account.
4. Address Clippy findings and split the largest Vite chunks before beta distribution.
