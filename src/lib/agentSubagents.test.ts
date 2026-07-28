import { describe, expect, it } from "vitest";
import {
  resolveSubagentsEnabled,
  shouldForceDisableSubagents,
  subagentsConfigEnabled,
  subagentsSpawnEnv,
  subagentsSpawnFlags,
} from "./agentSubagents";

describe("subagentsSpawnFlags", () => {
  it("adds no flags when subagents are enabled (CLI default)", () => {
    expect(subagentsSpawnFlags(true)).toEqual([]);
  });

  it("disables with --no-subagents", () => {
    expect(subagentsSpawnFlags(false)).toEqual(["--no-subagents"]);
  });
});

describe("subagentsSpawnEnv", () => {
  it("sets no env when enabled", () => {
    expect(subagentsSpawnEnv(true)).toEqual({});
  });

  it("sets GROK_SUBAGENTS=0 when disabled", () => {
    expect(subagentsSpawnEnv(false)).toEqual({ GROK_SUBAGENTS: "0" });
  });
});

describe("shouldForceDisableSubagents", () => {
  it("forces disable only when the setting is off", () => {
    expect(shouldForceDisableSubagents(false)).toBe(true);
    expect(shouldForceDisableSubagents(true)).toBe(false);
  });
});

describe("subagentsConfigEnabled", () => {
  it("mirrors the settings boolean for [subagents] enabled", () => {
    expect(subagentsConfigEnabled(true)).toBe(true);
    expect(subagentsConfigEnabled(false)).toBe(false);
  });
});

describe("resolveSubagentsEnabled", () => {
  it("prefers settings boolean over env", () => {
    expect(
      resolveSubagentsEnabled({ settingsEnabled: true, envValue: "0" }),
    ).toBe(true);
    expect(
      resolveSubagentsEnabled({ settingsEnabled: false, envValue: "1" }),
    ).toBe(false);
  });

  it("defaults to true when settings and env are unset", () => {
    expect(resolveSubagentsEnabled({})).toBe(true);
    expect(resolveSubagentsEnabled({ envValue: "  " })).toBe(true);
  });

  it("parses common env truthy/falsey forms when settings unset", () => {
    expect(resolveSubagentsEnabled({ envValue: "1" })).toBe(true);
    expect(resolveSubagentsEnabled({ envValue: "true" })).toBe(true);
    expect(resolveSubagentsEnabled({ envValue: "YES" })).toBe(true);
    expect(resolveSubagentsEnabled({ envValue: "0" })).toBe(false);
    expect(resolveSubagentsEnabled({ envValue: "false" })).toBe(false);
    expect(resolveSubagentsEnabled({ envValue: "off" })).toBe(false);
    expect(resolveSubagentsEnabled({ envValue: "maybe" })).toBe(true);
  });
});
