/**
 * Cross-session memory (Pi CLI experimental) — pure flag / env helpers.
 *
 * CLI surface:
 *   - `--experimental-memory` / `--no-memory` (top-level `pi` flags)
 *   - `GROK_MEMORY=1|0`
 *   - config.toml `[memory] enabled = true|false`
 *   - `pi memory clear [--workspace|--global|--all] -y`
 *
 * When disabled, App always forces `--no-memory` + `GROK_MEMORY=0` so a user
 * `~/.grok` config (shared mode) or leftover agent-home config cannot enable
 * memory accidentally — especially important for independent-mode isolation.
 */

export type MemoryClearScope = "workspace" | "global" | "all";

/** Top-level CLI flags placed before `agent` (e.g. `pi --no-auto-update …`). */
export function memorySpawnFlags(enabled: boolean): string[] {
  return enabled ? ["--experimental-memory"] : ["--no-memory"];
}

/** Env overrides applied to the agent process. */
export function memorySpawnEnv(enabled: boolean): Record<"GROK_MEMORY", string> {
  return { GROK_MEMORY: enabled ? "1" : "0" };
}

/**
 * Whether spawn must force-disable memory for isolation.
 * Always true when the setting is off (independent or shared).
 */
export function shouldForceDisableMemory(experimentalMemory: boolean): boolean {
  return !experimentalMemory;
}

/** `[memory] enabled` value written into agent config.toml (independent mode). */
export function memoryConfigEnabled(experimentalMemory: boolean): boolean {
  return experimentalMemory;
}

/**
 * Args for `pi memory clear` (after the binary path).
 * Workspace is the default product scope for “Clear workspace memory”.
 */
export function memoryClearArgs(
  scope: MemoryClearScope = "workspace",
): string[] {
  const args = ["memory", "clear", "-y"];
  if (scope === "global") args.push("--global");
  else if (scope === "all") args.push("--all");
  else args.push("--workspace");
  return args;
}

/**
 * Resolve whether experimental memory is on from settings + optional env
 * override (App settings win over process env for spawn decisions).
 */
export function resolveExperimentalMemory(input: {
  settingsEnabled?: boolean | null;
  envValue?: string | null;
}): boolean {
  if (typeof input.settingsEnabled === "boolean") {
    return input.settingsEnabled;
  }
  const raw = (input.envValue ?? "").trim().toLowerCase();
  if (!raw) return false;
  if (raw === "0" || raw === "false" || raw === "no" || raw === "off") {
    return false;
  }
  if (raw === "1" || raw === "true" || raw === "yes" || raw === "on") {
    return true;
  }
  return false;
}
