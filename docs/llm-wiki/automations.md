# Automations / scheduled tasks

**Status**: P1 UI + local storage + silent creation from chat + shell polling that fires while the app is open.
**Principle**: delegate to Build wherever Build can do it; the shell owns the list, the form and the orchestration. Never expose a JSON schema in the user's conversation.

## Entry points (modelled on Codex)

| Entry point | Behaviour |
|-------------|-----------|
| **New chat** under the sidebar logo | Draft chat with no project → the first send files it under "Other chats" |
| Sidebar **Scheduled** | Opens the task list in the main column (`#/automations`) |
| List **Create → with AI** | Switches to a project-less chat; the composer is pre-filled with **natural-language** guidance (no field schema shown) |
| List **Create → manually** | Right-hand form: title / prompt / project / model / effort / frequency / time / notifications |
| Composer "+" → create automation | Jumps to the Scheduled page |

## Chat creation protocol (silent)

1. The user describes **what to do and when to run it** in natural language.
2. On send, the host prepends a setup prefix to the agent text that is **never shown in the journal** (`wrapAutomationSetupAgentText`).
3. The agent confirms in natural language; once it has everything it appends a single fence at the **end** of the reply:

````text
```pi-automation
{"title":"...","prompt":"...","frequency":"daily|weekly|weekdays|once","time":"HH:MM","weekdays":[],"enabled":true}
```
````

4. On stream `done` the shell runs `extractAutomationPayload`: it **strips the fence** from the bubble, calls `automation_create`, and toasts "Scheduled: {title}".
5. Applied at most once per chat; the fence is stripped on reload too, so the user never sees raw JSON.

Implementation: `src/lib/automationSetup.ts` · intercepted in `App.tsx` `tryApplyAutomationFromSession`.

## Data

- File: `paths::automations_file()` (typically on macOS: `~/Library/Application Support/dev.pi.pi-app/automations.json`)
- Browser fallback: `localStorage["pi-app.automations"]`
- Fields: `title` `prompt` `enabled` `projectId` `modelId` `effort` `frequency` `time` `weekdays` `notify` `lastRunAt` `nextRunAt`

## Execution

1. Every 30s the shell checks for `enabled` tasks whose `nextRunAt` is due.
2. It never interrupts a `streaming` or connecting chat; **while busy it does not mark the task fired**, so it can still run once things go idle.
3. On trigger: `session_create` → write session prefs (model/effort) → `session_connect` → `session_send` the prompt.
4. **Connect failure**: delete the empty session so the sidebar does not collect ghost "empty Pi" chats; do not `mark_run`.
5. **Send failure**: leave the user bubble plus an error bubble in the chat; do not `mark_run`.
6. Success: update `lastRunAt` / `nextRunAt`; a `once` task sets `enabled=false` after it runs.

This coexists with Build's `/loop` and `scheduler_*`: the user can also ask the agent to schedule things directly inside a chat. The shell's list is an independent source of truth.

## UI rules

- **Welcome state**: only for a draft chat with no `sessionId`.
- **Has a `sessionId` but no messages**: show "No messages in this chat yet…", not the large new-chat splash.
- **Delete / destructive actions**: `window.confirm` is forbidden; use in-app dialogs (see [dialogs.md](./dialogs.md)). The `AutomationsPage` delete confirmation is the reference implementation.

## Tauri commands

- `automations_list`
- `automation_create` / `automation_update`
- `automation_set_enabled`
- `automation_mark_run`
- `automation_delete`

## Acceptance

- [x] Sidebar new-chat does not depend on the current project; the chat appears under "Other chats"
- [x] Scheduled list / filter / search / enable-disable / delete
- [x] Manual form create and edit
- [x] AI create entry point: natural-language seed, no JSON schema exposed
- [x] Assistant fence triggers `automation_create` automatically; the config block never renders in the bubble
- [x] Due tasks fire while the app is open (without blocking the main conversation architecture)
- [x] A failed connect leaves no empty session behind; an existing empty chat is not disguised as the new-chat page
- [ ] Background triggering with no window open (optional P2: system service / headless CLI)
- [ ] Two-way sync with the CLI scheduler (optional P2)
