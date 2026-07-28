import type { MessageKey } from "@/i18n";

export type PiPackageAccess =
  | "conversation"
  | "workspace"
  | "system"
  | "provider"
  | "network"
  | "browser";

export type PiPackageCatalogEntry = {
  id: string;
  source: `npm:${string}@${string}`;
  packageName: string;
  version: string;
  titleKey: MessageKey;
  descriptionKey: MessageKey;
  access: PiPackageAccess[];
  group: "foundation" | "optional";
};

/**
 * Curated sources are intentionally pinned. Pi packages execute in the local
 * agent process, so a moving `latest` tag is not appropriate for one-click
 * installation from the GUI.
 */
export const PI_PACKAGE_CATALOG: readonly PiPackageCatalogEntry[] = [
  {
    id: "goal",
    source: "npm:@narumitw/pi-goal@0.31.0",
    packageName: "@narumitw/pi-goal",
    version: "0.31.0",
    titleKey: "piExt.package.goal.title",
    descriptionKey: "piExt.package.goal.description",
    access: ["conversation", "workspace", "system"],
    group: "foundation",
  },
  {
    id: "subagents",
    source: "npm:@tintinweb/pi-subagents@0.14.3",
    packageName: "@tintinweb/pi-subagents",
    version: "0.14.3",
    titleKey: "piExt.package.subagents.title",
    descriptionKey: "piExt.package.subagents.description",
    access: ["conversation", "workspace", "system", "provider"],
    group: "foundation",
  },
  {
    id: "permissions",
    source: "npm:@gotgenes/pi-permission-system@23.0.2",
    packageName: "@gotgenes/pi-permission-system",
    version: "23.0.2",
    titleKey: "piExt.package.permissions.title",
    descriptionKey: "piExt.package.permissions.description",
    access: ["conversation", "workspace", "system"],
    group: "foundation",
  },
  {
    id: "codex-image",
    source: "npm:@capyup/pi-codex-image@0.2.2",
    packageName: "@capyup/pi-codex-image",
    version: "0.2.2",
    titleKey: "piExt.package.codexImage.title",
    descriptionKey: "piExt.package.codexImage.description",
    access: ["workspace", "provider", "network"],
    group: "foundation",
  },
  {
    id: "vcc",
    source: "npm:@sting8k/pi-vcc@0.4.0",
    packageName: "@sting8k/pi-vcc",
    version: "0.4.0",
    titleKey: "piExt.package.vcc.title",
    descriptionKey: "piExt.package.vcc.description",
    access: ["conversation"],
    group: "foundation",
  },
  {
    id: "web-access",
    source: "npm:pi-web-access@0.14.0",
    packageName: "pi-web-access",
    version: "0.14.0",
    titleKey: "piExt.package.web.title",
    descriptionKey: "piExt.package.web.description",
    access: ["workspace", "network", "provider"],
    group: "optional",
  },
  {
    id: "browser",
    source: "npm:pi-agent-browser-native@0.2.72",
    packageName: "pi-agent-browser-native",
    version: "0.2.72",
    titleKey: "piExt.package.browser.title",
    descriptionKey: "piExt.package.browser.description",
    access: ["workspace", "system", "network", "browser"],
    group: "optional",
  },
  {
    id: "local-rag",
    source: "npm:pi-local-rag@0.4.1",
    packageName: "pi-local-rag",
    version: "0.4.1",
    titleKey: "piExt.package.localRag.title",
    descriptionKey: "piExt.package.localRag.description",
    access: ["workspace", "system"],
    group: "optional",
  },
  {
    id: "hashline-edit",
    source: "npm:pi-hashline-edit@0.8.3",
    packageName: "pi-hashline-edit",
    version: "0.8.3",
    titleKey: "piExt.package.hashline.title",
    descriptionKey: "piExt.package.hashline.description",
    access: ["workspace"],
    group: "optional",
  },
] as const;

export const PI_FOUNDATION_PACKAGES = PI_PACKAGE_CATALOG.filter(
  (entry) => entry.group === "foundation",
);

/**
 * Return a stable identity for comparison with `pi list`, which may omit a
 * version even when the original install source was pinned.
 */
export function piPackageIdentity(source: string): string {
  const value = source.trim();
  if (!value.startsWith("npm:")) return value.replace(/#.*$/, "");

  const spec = value.slice(4);
  const versionAt = spec.lastIndexOf("@");
  if (versionAt <= 0) return `npm:${spec}`;

  // A scoped package begins with @. Its version separator is the final @ after
  // the slash; an unscoped package uses the only @ as the separator.
  const slashAt = spec.indexOf("/");
  const hasVersion =
    spec.startsWith("@") ? versionAt > slashAt : versionAt > 0;
  return `npm:${hasVersion ? spec.slice(0, versionAt) : spec}`;
}

export function isPinnedPiPackageSource(source: string): boolean {
  const value = source.trim();
  if (value.startsWith("npm:")) {
    return piPackageIdentity(value) !== value;
  }
  if (value.startsWith("git:") || value.startsWith("http")) {
    return /#[a-f0-9]{7,40}$/i.test(value);
  }
  // Local paths are already exact locations selected by the user.
  return (
    value.startsWith("./") ||
    value.startsWith("../") ||
    value.startsWith("/") ||
    /^[A-Za-z]:[\\/]/.test(value)
  );
}

export function installedPiPackageIds(
  installedSources: readonly string[],
): Set<string> {
  const installed = new Set(installedSources.map(piPackageIdentity));
  return new Set(
    PI_PACKAGE_CATALOG.filter((entry) =>
      installed.has(piPackageIdentity(entry.source)),
    ).map((entry) => entry.id),
  );
}
