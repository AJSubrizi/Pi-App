# Context settings

Settings → **Context** edits the two Markdown context files in the active Pi
agent profile:

| File | Pi behavior |
|------|-------------|
| `SYSTEM.md` | Replaces or extends Pi's minimal system prompt |
| `AGENTS.md` | Standing workflow, tool, and convention instructions |

The target directory follows `sessionDataMode` through
`paths::resolve_agent_grok_home`:

- independent: the App-owned agent profile;
- shared: the same `PI_AGENT_HOME` used by Pi CLI sessions.

## Commands

| Command | Role |
|---------|------|
| `agent_context_get` | Read both files and their active profile path |
| `agent_context_set` | Atomically save one allowed file, then soft-respawn Pi |

Only `AGENTS.md` and `SYSTEM.md` are accepted. Content is UTF-8 and capped at
1 MB. Saving reconnects the live agent so the new context applies from the next
turn.
