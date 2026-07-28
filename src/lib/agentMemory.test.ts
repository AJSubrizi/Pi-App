import { describe, expect, it } from "vitest";
import {
  memoryClearArgs,
  memoryConfigEnabled,
  memorySpawnEnv,
  memorySpawnFlags,
  resolveExperimentalMemory,
  shouldForceDisableMemory,
} from "./agentMemory";

describe("memorySpawnFlags", () => {
  it("enables with --experimental-memory", () => {
    expect(memorySpawnFlags(true)).toEqual(["--experimental-memory"]);
  });

  it("disables with --no-memory", () => {
    expect(memorySpawnFlags(false)).toEqual(["--no-memory"]);
  });
});

describe("memorySpawnEnv", () => {
  it("sets GROK_MEMORY=1 when enabled", () => {
    expect(memorySpawnEnv(true)).toEqual({ GROK_MEMORY: "1" });
  });

  it("sets GROK_MEMORY=0 when disabled", () => {
    expect(memorySpawnEnv(false)).toEqual({ GROK_MEMORY: "0" });
  });
});

describe("shouldForceDisableMemory", () => {
  it("forces disable only when experimental memory is off", () => {
    expect(shouldForceDisableMemory(false)).toBe(true);
    expect(shouldForceDisableMemory(true)).toBe(false);
  });
});

describe("memoryConfigEnabled", () => {
  it("mirrors the settings boolean for [memory] enabled", () => {
    expect(memoryConfigEnabled(true)).toBe(true);
    expect(memoryConfigEnabled(false)).toBe(false);
  });
});

describe("memoryClearArgs", () => {
  it("defaults to workspace clear with -y", () => {
    expect(memoryClearArgs()).toEqual([
      "memory",
      "clear",
      "-y",
      "--workspace",
    ]);
    expect(memoryClearArgs("workspace")).toEqual([
      "memory",
      "clear",
      "-y",
      "--workspace",
    ]);
  });

  it("supports global and all scopes", () => {
    expect(memoryClearArgs("global")).toEqual([
      "memory",
      "clear",
      "-y",
      "--global",
    ]);
    expect(memoryClearArgs("all")).toEqual(["memory", "clear", "-y", "--all"]);
  });
});

describe("resolveExperimentalMemory", () => {
  it("prefers settings boolean over env", () => {
    expect(
      resolveExperimentalMemory({ settingsEnabled: true, envValue: "0" }),
    ).toBe(true);
    expect(
      resolveExperimentalMemory({ settingsEnabled: false, envValue: "1" }),
    ).toBe(false);
  });

  it("parses common env truthy/falsey forms when settings unset", () => {
    expect(resolveExperimentalMemory({ envValue: "1" })).toBe(true);
    expect(resolveExperimentalMemory({ envValue: "true" })).toBe(true);
    expect(resolveExperimentalMemory({ envValue: "YES" })).toBe(true);
    expect(resolveExperimentalMemory({ envValue: "0" })).toBe(false);
    expect(resolveExperimentalMemory({ envValue: "false" })).toBe(false);
    expect(resolveExperimentalMemory({ envValue: "off" })).toBe(false);
    expect(resolveExperimentalMemory({})).toBe(false);
    expect(resolveExperimentalMemory({ envValue: "  " })).toBe(false);
    expect(resolveExperimentalMemory({ envValue: "maybe" })).toBe(false);
  });
});
