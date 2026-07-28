/**
 * Slash command / skill palette — Codex-style single-row items.
 * Layout: [icon] Title ........................ description
 * Hover + keyboard share the same is-active surface.
 */

import {
  useEffect,
  useRef,
  type CSSProperties,
  type ReactNode,
  type Ref,
} from "react";
import type { Locale } from "@/i18n";
import { createT } from "@/i18n";
import type { SlashItem } from "@/lib/slashCatalog";
import {
  IconActivity,
  IconArrowsMinimize,
  IconAutomations,
  IconBox,
  IconCircleDashed,
  IconClipboardList,
  IconDoctor,
  IconNewChat,
  IconPlug,
  IconSettings,
  IconShieldCheck,
  IconPuzzle,
  IconSkills,
  IconTarget,
} from "@/components/icons";

const ICON_SIZE = 18;

function itemIcon(item: SlashItem): ReactNode {
  if (item.kind === "skill") {
    return <IconPuzzle size={ICON_SIZE} />;
  }
  const key = item.action ?? item.mode ?? item.name;
  switch (key) {
    case "goal":
      return <IconTarget size={ICON_SIZE} />;
    case "plan":
      return <IconClipboardList size={ICON_SIZE} />;
    case "compact":
      return <IconArrowsMinimize size={ICON_SIZE} />;
    case "status":
      return <IconActivity size={ICON_SIZE} />;
    case "mcp":
      return <IconPlug size={ICON_SIZE} />;
    case "doctor":
      return <IconDoctor size={ICON_SIZE} />;
    case "settings":
      return <IconSettings size={ICON_SIZE} />;
    case "automations":
      return <IconAutomations size={ICON_SIZE} />;
    case "newChat":
    case "new":
      return <IconNewChat size={ICON_SIZE} />;
    case "yolo":
    case "always-approve":
      return <IconShieldCheck size={ICON_SIZE} />;
    default:
      if (item.kind === "mode") return <IconCircleDashed size={ICON_SIZE} />;
      if (item.kind === "action") return <IconBox size={ICON_SIZE} />;
      return <IconSkills size={ICON_SIZE} />;
  }
}

export function SlashPalette({
  open,
  locale,
  commands,
  skills,
  activeIndex,
  onActiveIndexChange,
  onSelect,
  style,
  panelRef,
  resolveTitle,
  resolveDescription,
}: {
  open: boolean;
  locale: Locale;
  commands: SlashItem[];
  skills: SlashItem[];
  activeIndex: number;
  onActiveIndexChange: (i: number) => void;
  onSelect: (item: SlashItem) => void;
  style?: CSSProperties;
  /** For floating-menu outside-click + position clamp. */
  panelRef?: Ref<HTMLDivElement | null>;
  resolveTitle: (item: SlashItem) => string;
  resolveDescription: (item: SlashItem) => string;
}) {
  const tr = createT(locale);
  const listRef = useRef<HTMLDivElement | null>(null);
  const flat = [...commands, ...skills];

  const setRefs = (node: HTMLDivElement | null) => {
    listRef.current = node;
    if (typeof panelRef === "function") panelRef(node);
    else if (panelRef && "current" in panelRef) {
      (panelRef as { current: HTMLDivElement | null }).current = node;
    }
  };

  useEffect(() => {
    if (!open) return;
    const el = listRef.current?.querySelector<HTMLElement>(
      `[data-slash-idx="${activeIndex}"]`,
    );
    el?.scrollIntoView({ block: "nearest" });
  }, [activeIndex, open]);

  if (!open) return null;

  let idx = 0;
  const renderRow = (item: SlashItem) => {
    const i = idx++;
    const active = i === activeIndex;
    const title = resolveTitle(item);
    const desc = resolveDescription(item);
    // Codex: right side is the short description (not /command path).
    // Skills: prefer description; fall back to source label.
    const right =
      desc.trim() ||
      (item.kind === "skill" && item.source ? item.source : "") ||
      `/${item.name}`;

    return (
      <button
        key={item.id}
        type="button"
        role="option"
        aria-selected={active}
        data-slash-idx={i}
        className={
          "slash-palette__item" +
          (active ? " is-active" : "") +
          (item.kind === "skill" ? " slash-palette__item--skill" : "")
        }
        onMouseEnter={() => onActiveIndexChange(i)}
        onClick={() => onSelect(item)}
      >
        <span className="slash-palette__ico" aria-hidden>
          {itemIcon(item)}
        </span>
        <span className="slash-palette__title" title={title}>
          {title}
        </span>
        {right ? (
          <span className="slash-palette__desc" title={right}>
            {right}
          </span>
        ) : null}
      </button>
    );
  };

  return (
    <div
      className="menu-panel slash-palette"
      role="listbox"
      style={style}
      ref={setRefs}
    >
      {commands.length > 0 && (
        <>
          <div className="slash-palette__section">
            {tr("slash.section.commands")}
          </div>
          {commands.map(renderRow)}
        </>
      )}
      {skills.length > 0 && (
        <>
          <div className="slash-palette__section">
            {tr("slash.section.skills")}
          </div>
          {skills.map(renderRow)}
        </>
      )}
      {flat.length === 0 && (
        <div className="slash-palette__empty">{tr("slash.empty")}</div>
      )}
    </div>
  );
}
