import { describe, expect, it } from "vitest";
import {
  AGENTS_MD_TEMPLATE_PATH,
  agentsMdTemplateBody,
  classifyProjectRulePath,
  hasRootAgentsMd,
  isGrokRulesPath,
  isNestedAgentsPath,
  normalizeRuleRelativePath,
  preferredAgentsMdPath,
  selectExistingProjectRules,
} from "./projectRules";

describe("normalizeRuleRelativePath", () => {
  it("strips ./ and backslashes", () => {
    expect(normalizeRuleRelativePath("./AGENTS.md")).toBe("AGENTS.md");
    expect(normalizeRuleRelativePath(".\\CLAUDE.md")).toBe("CLAUDE.md");
    expect(normalizeRuleRelativePath("/.grok/rules/foo.md")).toBe(
      ".grok/rules/foo.md",
    );
    expect(normalizeRuleRelativePath("  a/b/  ")).toBe("a/b");
  });
});

describe("classifyProjectRulePath", () => {
  it("classifies root AGENTS / AGENT variants", () => {
    expect(classifyProjectRulePath("AGENTS.md")?.kind).toBe("agents_md");
    expect(classifyProjectRulePath("Agents.md")?.kind).toBe("agents_md");
    expect(classifyProjectRulePath("agents.md")?.kind).toBe("agents_md");
    expect(classifyProjectRulePath("AGENT.md")?.kind).toBe("agents_md");
  });

  it("classifies root CLAUDE.md", () => {
    expect(classifyProjectRulePath("CLAUDE.md")?.kind).toBe("claude_md");
    expect(classifyProjectRulePath("claude.md")?.kind).toBe("claude_md");
  });

  it("classifies .grok/rules* paths", () => {
    expect(classifyProjectRulePath(".grok/rules")?.kind).toBe("grok_rules");
    expect(classifyProjectRulePath(".grok/rules.md")?.kind).toBe("grok_rules");
    expect(classifyProjectRulePath(".grok/rules.txt")?.kind).toBe("grok_rules");
    expect(classifyProjectRulePath(".grok/rules/base.md")?.kind).toBe(
      "grok_rules",
    );
    expect(classifyProjectRulePath(".grok/rules/team/coding.md")?.kind).toBe(
      "grok_rules",
    );
  });

  it("classifies nested AGENTS.md under .grok", () => {
    expect(classifyProjectRulePath(".grok/AGENTS.md")?.kind).toBe(
      "nested_agents",
    );
    expect(classifyProjectRulePath(".grok/subdir/AGENTS.md")?.kind).toBe(
      "nested_agents",
    );
    expect(classifyProjectRulePath(".grok/a/b/Agents.md")?.kind).toBe(
      "nested_agents",
    );
  });

  it("rejects unrelated paths", () => {
    expect(classifyProjectRulePath("README.md")).toBeNull();
    expect(classifyProjectRulePath("docs/AGENTS.md")).toBeNull();
    expect(classifyProjectRulePath("src/lib/foo.ts")).toBeNull();
    expect(classifyProjectRulePath(".grok/config.toml")).toBeNull();
    expect(classifyProjectRulePath(".grok/hooks/x.json")).toBeNull();
    expect(classifyProjectRulePath("")).toBeNull();
  });
});

describe("isGrokRulesPath / isNestedAgentsPath", () => {
  it("does not treat nested agents as grok_rules", () => {
    expect(isGrokRulesPath(".grok/AGENTS.md")).toBe(false);
    expect(isNestedAgentsPath(".grok/AGENTS.md")).toBe(true);
    expect(isNestedAgentsPath(".grok/rules/AGENTS.md")).toBe(false);
    expect(isGrokRulesPath(".grok/rules/AGENTS.md")).toBe(true);
  });
});

describe("selectExistingProjectRules", () => {
  it("filters, dedupes, and orders by kind then path", () => {
    const list = selectExistingProjectRules([
      "README.md",
      ".grok/rules/z.md",
      "CLAUDE.md",
      "AGENTS.md",
      "./AGENTS.md",
      ".grok/rules/a.md",
      ".grok/nested/AGENTS.md",
      "src/x.ts",
    ]);
    expect(list.map((r) => r.relativePath)).toEqual([
      "AGENTS.md",
      "CLAUDE.md",
      ".grok/rules/a.md",
      ".grok/rules/z.md",
      ".grok/nested/AGENTS.md",
    ]);
    expect(list.map((r) => r.kind)).toEqual([
      "agents_md",
      "claude_md",
      "grok_rules",
      "grok_rules",
      "nested_agents",
    ]);
  });

  it("returns empty for no matches", () => {
    expect(selectExistingProjectRules(["a.ts", "b.md"])).toEqual([]);
  });
});

describe("hasRootAgentsMd / preferredAgentsMdPath", () => {
  it("detects root agents presence", () => {
    expect(hasRootAgentsMd(["CLAUDE.md"])).toBe(false);
    expect(hasRootAgentsMd(["Agents.md"])).toBe(true);
  });

  it("prefers existing agents path else template path", () => {
    expect(preferredAgentsMdPath(["CLAUDE.md"])).toBe(AGENTS_MD_TEMPLATE_PATH);
    expect(preferredAgentsMdPath(["Agents.md"])).toBe("Agents.md");
    expect(preferredAgentsMdPath(["AGENTS.md", "Agents.md"])).toBe("AGENTS.md");
  });
});

describe("agentsMdTemplateBody", () => {
  it("is a short non-empty markdown stub with useful headings", () => {
    const body = agentsMdTemplateBody();
    expect(body.startsWith("# Project rules")).toBe(true);
    expect(body).toContain("## Commands");
    expect(body).toContain("## Conventions");
    expect(body.length).toBeLessThan(800);
    // No marketing fluff
    expect(body.toLowerCase()).not.toContain("leverage");
    expect(body.toLowerCase()).not.toContain("robust");
  });
});
