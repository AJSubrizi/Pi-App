import { describe, expect, it } from "vitest";
import {
  DEFAULT_WORKSPACE,
  WORKSPACES,
  WORKSPACE_IDS,
  WORKSPACE_STORAGE_KEY,
  isComingSoon,
  isWorkspaceId,
  loadWorkspace,
  nextWorkspace,
  saveWorkspace,
  workspaceMeta,
  type WorkspaceStorage,
} from "./workspace";

function memoryStorage(
  initial: Record<string, string> = {},
): WorkspaceStorage & { map: Record<string, string> } {
  const map = { ...initial };
  return {
    map,
    getItem: (k) => (k in map ? map[k]! : null),
    setItem: (k, v) => {
      map[k] = v;
    },
  };
}

function throwingStorage(): WorkspaceStorage {
  return {
    getItem() {
      throw new Error("denied");
    },
    setItem() {
      throw new Error("quota");
    },
  };
}

describe("workspace ids", () => {
  it("accepts only known ids", () => {
    expect(isWorkspaceId("code")).toBe(true);
    expect(isWorkspaceId("pr")).toBe(true);
    expect(isWorkspaceId("design")).toBe(true);
    expect(isWorkspaceId("nope")).toBe(false);
    expect(isWorkspaceId("")).toBe(false);
    expect(isWorkspaceId(null)).toBe(false);
    expect(isWorkspaceId(3)).toBe(false);
  });

  it("defaults to code so an upgrade lands where the user was", () => {
    expect(DEFAULT_WORKSPACE).toBe("code");
    expect(WORKSPACES[0]!.id).toBe("code");
  });

  it("describes every declared id exactly once", () => {
    expect(WORKSPACES.map((w) => w.id)).toEqual([...WORKSPACE_IDS]);
    expect(new Set(WORKSPACES.map((w) => w.labelKey)).size).toBe(
      WORKSPACES.length,
    );
  });

  it("marks only design as coming soon", () => {
    expect(isComingSoon("design")).toBe(true);
    expect(isComingSoon("code")).toBe(false);
    expect(isComingSoon("pr")).toBe(false);
  });

  it("falls back to code metadata for an unknown id", () => {
    expect(workspaceMeta("bogus" as never).id).toBe("code");
  });
});

describe("persistence", () => {
  it("round-trips a saved workspace", () => {
    const s = memoryStorage();
    saveWorkspace(s, "pr");
    expect(s.map[WORKSPACE_STORAGE_KEY]).toBe("pr");
    expect(loadWorkspace(s)).toBe("pr");
  });

  it("returns the default when nothing is stored", () => {
    expect(loadWorkspace(memoryStorage())).toBe("code");
  });

  it("ignores a stored value it cannot render", () => {
    const s = memoryStorage({
      [WORKSPACE_STORAGE_KEY]: "from-a-newer-build",
    });
    expect(loadWorkspace(s)).toBe("code");
  });

  it("survives storage throwing", () => {
    const s = throwingStorage();
    expect(loadWorkspace(s)).toBe("code");
    expect(() => saveWorkspace(s, "pr")).not.toThrow();
  });
});

describe("nextWorkspace", () => {
  it("switches to the clicked workspace", () => {
    expect(nextWorkspace("code", "pr")).toBe("pr");
  });

  it("clicking the active one is a no-op rather than a toggle-off", () => {
    expect(nextWorkspace("pr", "pr")).toBe("pr");
  });

  it("allows selecting a coming-soon workspace so it can explain itself", () => {
    expect(nextWorkspace("code", "design")).toBe("design");
  });
});
