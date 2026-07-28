import { describe, expect, it } from "vitest";
import {
  agentEntriesFromFileNames,
  agentMetaLine,
  agentScopeRank,
  agentScopeTone,
  collectAgentDefs,
  collectPersonaDefs,
  definitionNameFromFileName,
  extractAgentDescription,
  grokHomeFromUserHome,
  isAgentDefinitionFileName,
  isPersonaDefinitionFileName,
  personaEntriesFromFileNames,
  resolveAgentsDirs,
  resolvePersonasDirs,
  sortAgentDefs,
} from "./agentsDiscovery";

describe("file name filters", () => {
  it("accepts agent markdown only", () => {
    expect(isAgentDefinitionFileName("explore.md")).toBe(true);
    expect(isAgentDefinitionFileName("plan.markdown")).toBe(true);
    expect(isAgentDefinitionFileName("explore.toml")).toBe(false);
    expect(isAgentDefinitionFileName(".hidden.md")).toBe(false);
    expect(isAgentDefinitionFileName("")).toBe(false);
    expect(isAgentDefinitionFileName(null)).toBe(false);
  });

  it("accepts persona toml and markdown", () => {
    expect(isPersonaDefinitionFileName("reviewer.toml")).toBe(true);
    expect(isPersonaDefinitionFileName("concise.md")).toBe(true);
    expect(isPersonaDefinitionFileName("notes.txt")).toBe(false);
  });

  it("stems names from file names", () => {
    expect(definitionNameFromFileName("explore.md")).toBe("explore");
    expect(definitionNameFromFileName("my-agent.markdown")).toBe("my-agent");
    expect(definitionNameFromFileName("path/to/plan.md")).toBe("plan");
  });
});

describe("path resolution", () => {
  it("builds ~/.grok from home", () => {
    expect(grokHomeFromUserHome("/Users/me")).toBe("/Users/me/.grok");
    expect(grokHomeFromUserHome("/Users/me/")).toBe("/Users/me/.grok");
  });

  it("resolves user / project / bundled agent dirs", () => {
    const dirs = resolveAgentsDirs("/home/dev", "/work/app");
    expect(dirs.user).toBe("/home/dev/.grok/agents");
    expect(dirs.project).toBe("/work/app/.grok/agents");
    expect(dirs.bundled).toBe("/home/dev/.grok/bundled/agents");
  });

  it("omits project dir when no project", () => {
    const dirs = resolveAgentsDirs("/home/dev", null);
    expect(dirs.project).toBeNull();
    expect(dirs.user).toBe("/home/dev/.grok/agents");
  });

  it("resolves persona dirs similarly", () => {
    const dirs = resolvePersonasDirs("/home/dev", "/work/app");
    expect(dirs.user).toBe("/home/dev/.grok/personas");
    expect(dirs.project).toBe("/work/app/.grok/personas");
    expect(dirs.bundled).toBe("/home/dev/.grok/bundled/personas");
  });
});

describe("entries from listings", () => {
  it("builds agent entries and skips junk", () => {
    const entries = agentEntriesFromFileNames(
      ["explore.md", "README.txt", ".DS_Store", "plan.md"],
      "/home/dev/.grok/agents",
      "user",
    );
    expect(entries).toEqual([
      {
        name: "explore",
        path: "/home/dev/.grok/agents/explore.md",
        scope: "user",
      },
      {
        name: "plan",
        path: "/home/dev/.grok/agents/plan.md",
        scope: "user",
      },
    ]);
  });

  it("builds persona entries", () => {
    const entries = personaEntriesFromFileNames(
      ["reviewer.toml", "notes.txt"],
      "/proj/.grok/personas",
      "project",
    );
    expect(entries).toEqual([
      {
        name: "reviewer",
        path: "/proj/.grok/personas/reviewer.toml",
        scope: "project",
      },
    ]);
  });
});

describe("sort / scope", () => {
  it("ranks project before user before bundled", () => {
    expect(agentScopeRank("project")).toBeLessThan(agentScopeRank("user"));
    expect(agentScopeRank("user")).toBeLessThan(agentScopeRank("bundled"));
  });

  it("sorts by scope then name", () => {
    const sorted = sortAgentDefs([
      { name: "zeta", scope: "user" },
      { name: "alpha", scope: "bundled" },
      { name: "beta", scope: "project" },
      { name: "alpha", scope: "user" },
    ]);
    expect(sorted.map((a) => `${a.scope}:${a.name}`)).toEqual([
      "project:beta",
      "user:alpha",
      "user:zeta",
      "bundled:alpha",
    ]);
  });

  it("maps scope to badge tone", () => {
    expect(agentScopeTone("user")).toBe("user");
    expect(agentScopeTone("project")).toBe("project");
    expect(agentScopeTone("bundled")).toBe("plugin");
    expect(agentScopeTone("other")).toBe("muted");
  });
});

describe("collect multi-scope", () => {
  it("merges scopes without dropping same-name defs", () => {
    const agents = collectAgentDefs({
      projectFiles: ["explore.md"],
      projectDir: "/p/.grok/agents",
      userFiles: ["explore.md", "custom.md"],
      userDir: "/h/.grok/agents",
      bundledFiles: ["plan.md"],
      bundledDir: "/h/.grok/bundled/agents",
    });
    expect(agents.map((a) => `${a.scope}:${a.name}`)).toEqual([
      "project:explore",
      "user:custom",
      "user:explore",
      "bundled:plan",
    ]);
  });

  it("collects personas across scopes", () => {
    const personas = collectPersonaDefs({
      userFiles: ["thorough.toml"],
      userDir: "/h/.grok/personas",
      bundledFiles: ["reviewer.toml"],
      bundledDir: "/h/.grok/bundled/personas",
    });
    expect(personas.map((p) => p.name)).toEqual(["thorough", "reviewer"]);
  });
});

describe("frontmatter description", () => {
  it("reads plain description", () => {
    const md = `---
name: explore
description: Fast explorer
---
body`;
    expect(extractAgentDescription(md)).toBe("Fast explorer");
  });

  it("reads folded description first line", () => {
    const md = `---
description: >
  Fast agent specialized for exploring.
  Second line ignored for short meta.
---
body`;
    expect(extractAgentDescription(md)).toBe(
      "Fast agent specialized for exploring.",
    );
  });

  it("returns null without frontmatter", () => {
    expect(extractAgentDescription("no frontmatter")).toBeNull();
    expect(extractAgentDescription(null)).toBeNull();
  });
});

describe("meta line", () => {
  it("includes scope and truncated description", () => {
    expect(
      agentMetaLine({
        scope: "user",
        description: "hello",
      }),
    ).toBe("user · hello");
    const long = "x".repeat(100);
    const line = agentMetaLine({
      scope: "project",
      description: long,
    });
    expect(line.startsWith("project · ")).toBe(true);
    expect(line.endsWith("…")).toBe(true);
  });
});
