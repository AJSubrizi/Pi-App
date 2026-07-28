# Releases

## Version files (keep in sync)

| File | Field |
|------|--------|
| `package.json` | `version` |
| `src-tauri/tauri.conf.json` | `version` |
| `src-tauri/Cargo.toml` | `[package].version` |
| `src/i18n/messages.ts` | `app.versionFooter` (`Pi vX.Y.Z`) |

## Steps

1. Add `## [X.Y.Z] - YYYY-MM-DD` to `CHANGELOG.md`
2. `pnpm typecheck && pnpm test` (and `cargo test` in `src-tauri` when possible)
3. `./scripts/release-tag.sh X.Y.Z --push`
4. GitHub Actions builds installers and attaches them to the Release

Update checks read: `https://api.github.com/repos/AJSubrizi/Pi-App/releases/latest`
