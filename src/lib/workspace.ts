/**
 * Workspaces — the top-level context the shell is working in.
 *
 * A workspace decides what the sidebar lists and what "new chat" means. It is
 * orthogonal to `mainPane` (which pane of the current workspace is showing) and
 * to layout (how wide the panes are).
 *
 * `code` is the historical behaviour and must stay the default so an upgrade
 * lands users exactly where they were.
 */

export const WORKSPACE_STORAGE_KEY = "pi-app.workspace";

export const WORKSPACE_IDS = ["code", "pr", "design"] as const;
export type WorkspaceId = (typeof WORKSPACE_IDS)[number];

export const DEFAULT_WORKSPACE: WorkspaceId = "code";

export type WorkspaceMeta = {
  id: WorkspaceId;
  /** i18n key for the tooltip / aria-label. */
  labelKey: string;
  /** Not yet built: selectable but shows a placeholder instead of content. */
  comingSoon: boolean;
};

export const WORKSPACES: WorkspaceMeta[] = [
  { id: "code", labelKey: "workspace.code", comingSoon: false },
  { id: "pr", labelKey: "workspace.pr", comingSoon: false },
  { id: "design", labelKey: "workspace.design", comingSoon: true },
];

export function isWorkspaceId(value: unknown): value is WorkspaceId {
  return (
    typeof value === "string" &&
    (WORKSPACE_IDS as readonly string[]).includes(value)
  );
}

export function workspaceMeta(id: WorkspaceId): WorkspaceMeta {
  return WORKSPACES.find((w) => w.id === id) ?? WORKSPACES[0]!;
}

/** True when the workspace has no real content yet. */
export function isComingSoon(id: WorkspaceId): boolean {
  return workspaceMeta(id).comingSoon;
}

/** Storage-like object (localStorage or a test double), as in `theme.ts`. */
export interface WorkspaceStorage {
  getItem(key: string): string | null;
  setItem(key: string, value: string): void;
}

/**
 * Read the persisted workspace.
 *
 * Anything unrecognised — a hand-edited value, or an id from a newer build the
 * user rolled back from — falls back to `code` rather than leaving the shell in
 * a workspace it cannot render.
 */
export function loadWorkspace(storage: WorkspaceStorage): WorkspaceId {
  try {
    const raw = storage.getItem(WORKSPACE_STORAGE_KEY);
    return isWorkspaceId(raw) ? raw : DEFAULT_WORKSPACE;
  } catch {
    return DEFAULT_WORKSPACE;
  }
}

export function saveWorkspace(
  storage: WorkspaceStorage,
  id: WorkspaceId,
): void {
  try {
    storage.setItem(WORKSPACE_STORAGE_KEY, id);
  } catch {
    // Private mode / quota: the switcher still works for this session.
  }
}

/**
 * Resolve the workspace to activate when the user clicks an icon.
 *
 * Coming-soon workspaces are still selectable so the placeholder can explain
 * what is planned; the caller decides what to render.
 */
export function nextWorkspace(
  current: WorkspaceId,
  clicked: WorkspaceId,
): WorkspaceId {
  return clicked === current ? current : clicked;
}
