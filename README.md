<p align="center">
  <img src="assets/logo.png" alt="Pi" width="96" height="96" />
</p>

<h1 align="center">Pi</h1>

<p align="center"><strong>A desktop GUI for <a href="https://pi.dev">Pi</a></strong></p>
<p align="center">
  Projects, sessions, previews and packages — on top of the real Pi agent.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT" /></a>
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey" alt="Platforms" />
  <img src="https://img.shields.io/badge/Tauri-2-orange" alt="Tauri 2" />
  <img src="https://img.shields.io/badge/runtime-Pi%20RPC-informational" alt="Pi RPC" />
  <img src="https://img.shields.io/badge/note-unofficial-yellow" alt="Unofficial" />
</p>

---

## Philosophy

Pi keeps a **small core**. Workflows live in packages — extensions, skills, prompts, themes — not as fake built-ins inside the shell.

This app is the same idea on the desktop:

- The agent is **Pi**, talking over **JSONL RPC** (`pi --mode rpc`)
- Models, auth, sessions and tools stay **owned by Pi**
- The GUI is a workbench: chat, projects, file tree, git changes, browser, packages
- If something is missing, **install a package** or ask Pi to write an extension

It is a **sister project** to [pi.dev](https://pi.dev) — community-built, MIT, not an official Pi product.

```text
┌─────────────────────────┐
│  Pi desktop (this app)  │
│  Tauri · React · Rust   │
└───────────┬─────────────┘
            │  JSONL RPC
            ▼
┌─────────────────────────┐
│  pi --mode rpc          │
│  models · tools · pkgs  │
└─────────────────────────┘
```

---

## What you get

| Area | Details |
|------|---------|
| **Sessions** | Stream chat with the real Pi process — create, resume, abort, compact |
| **Models** | Pick model and thinking level from what Pi exposes |
| **Projects** | Trusted folders, multi-session sidebar, workbench around your repo |
| **Resources** | Files, previews, git changes (stage / discard / commit & push), embedded browser |
| **Packages** | Install Pi packages from npm, git or a local path (user or project scope) |
| **Extensions** | Native UI for Pi `select` / `confirm` / `input` / `editor` and status widgets |
| **Setup** | First-run detection and install path for the Pi CLI |

---

## Requirements

- **Pi CLI** — the agent runtime  
  ```bash
  npm install --global @earendil-works/pi-coding-agent@latest
  pi --version
  ```
- Credentials and providers are configured in Pi itself — see [Pi docs](https://pi.dev/docs/latest)
- To **build from source**: Node 22+, pnpm 9+, Rust stable, and [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS

---

## Install (macOS)

Download the `.dmg` from [Releases](https://github.com/AJSubrizi/Pi-App/releases) (`aarch64` = Apple Silicon, `x64` = Intel).

Builds are **not Apple-notarized** (no paid Developer ID). Gatekeeper may say the app is damaged or move it to Trash. That is expected for unsigned community builds.

**After dragging Pi to Applications**, run:

```bash
xattr -cr /Applications/Pi.app
codesign --force --deep --sign - /Applications/Pi.app
open /Applications/Pi.app
```

Or: Finder → right-click **Pi** → **Open** → confirm.

---

## Develop

```bash
pnpm install
pnpm dev          # full desktop app
pnpm dev:ui       # frontend only
```

Checks:

```bash
pnpm typecheck
pnpm test
pnpm build:ui
cd src-tauri && cargo test
```

Optional live RPC smoke test (starts `pi --mode rpc`, no paid prompt):

```bash
cd src-tauri
PI_APP_LIVE_RPC=1 cargo test live_pi_rpc_session_under_30s -- --nocapture
```

---

## Data layout

| Owner | Where |
|-------|--------|
| **This app** | Platform app-data for `dev.pi/pi-app` (override with `PI_APP_HOME`; falls back to `~/.pi-app`) |
| **Pi** | `~/.pi/agent` — config, auth, agent sessions |

The GUI does not copy provider secrets into its package center. Pi remains the source of truth for models and keys.

---

## Packages

**Settings → Packages** manages Pi packages the same way the CLI does:

- npm · git · local path  
- user-wide or current project  
- extensions, skills, prompts, themes  

Plan modes, gates, sub-agents and other workflows should come from packages — not from hard-coded shell features.

---

## License

[MIT](LICENSE)

Unofficial desktop shell for [Pi](https://pi.dev) · runtime by [earendil-works/pi](https://github.com/earendil-works/pi)
