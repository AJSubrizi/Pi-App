/**
 * Subagent spawning — pure flag / env helpers for Pi CLI agents.
 *
 * CLI surface (enabled by default):
 *   - `--no-subagents` (top-level `pi` flag, before `agent`)
 *   - `GROK_SUBAGENTS=0|1`
 *   - config.toml `[subagents] enabled = true|false`
 *
 * When disabled, App always forces `--no-subagents` + `GROK_SUBAGENTS=0` so a
 * user `~/.grok` config (shared mode) or leftover agent-home config cannot
 * re-enable subagents. When enabled, spawn leaves CLI defaults alone (on).
 */

/** Top-level CLI flags placed before `agent` (e.g. `pi --no-auto-update …`). */
export function subagentsSpawnFlags(enabled: boolean): string[] {
  return enabled ? [] : ["--no-subagents"];
}

/** Env overrides applied to the agent process when disabling. */
export function subagentsSpawnEnv(
  enabled: boolean,
): Record<"GROK_SUBAGENTS", string> | Record<string, never> {
  return enabled ? {} : { GROK_SUBAGENTS: "0" };
}

/**
 * Whether spawn must force-disable subagents.
 * Always true when the setting is off (independent or shared).
 */
export function shouldForceDisableSubagents(subagentsEnabled: boolean): boolean {
  return !subagentsEnabled;
}

/** `[subagents] enabled` value written into agent config.toml (independent mode). */
export function subagentsConfigEnabled(subagentsEnabled: boolean): boolean {
  return subagentsEnabled;
}

/**
 * Resolve whether subagents are on from settings + optional env override
 * (App settings win over process env for spawn decisions). Default true.
 */
export function resolveSubagentsEnabled(input: {
  settingsEnabled?: boolean | null;
  envValue?: string | null;
}): boolean {
  if (typeof input.settingsEnabled === "boolean") {
    return input.settingsEnabled;
  }
  const raw = (input.envValue ?? "").trim().toLowerCase();
  if (!raw) return true;
  if (raw === "0" || raw === "false" || raw === "no" || raw === "off") {
    return false;
  }
  if (raw === "1" || raw === "true" || raw === "yes" || raw === "on") {
    return true;
  }
  return true;
}
