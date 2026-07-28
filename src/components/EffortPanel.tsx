/**
 * 3-stage effort control: click chip to open panel, drag along a segmented track.
 * Particle burst on change (CSS). Panel portaled to body.
 */

import { useCallback, useEffect, useId, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { GROK_BUILD_EFFORTS } from "@/lib/agentCatalog";
import { IconChevronDown } from "@/components/icons";
import { Tip } from "@/components/ui/tooltip";
import { useFloatingMenu } from "@/lib/floatingMenu";

type EffortId = "low" | "medium" | "high";

const ORDER: EffortId[] = ["low", "medium", "high"];

function indexOf(id: string): number {
  const i = ORDER.indexOf(id as EffortId);
  return i >= 0 ? i : 2;
}

export interface EffortPanelProps {
  value: string;
  onChange: (v: EffortId) => void;
  labels: Record<EffortId, string>;
  ariaLabel: string;
  title?: string;
}

export function EffortPanel({
  value,
  onChange,
  labels,
  ariaLabel,
  title,
}: EffortPanelProps) {
  const [open, setOpen] = useState(false);
  const [particles, setParticles] = useState(0);
  const rootRef = useRef<HTMLDivElement>(null);
  const triggerRef = useRef<HTMLButtonElement>(null);
  const panelRef = useRef<HTMLDivElement>(null);
  const trackRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);
  const listId = useId();
  const idx = indexOf(value);

  const { pos, style } = useFloatingMenu({
    open,
    triggerRef,
    panelRef,
    roots: [rootRef],
    onClose: () => setOpen(false),
    placement: "auto",
    width: 240,
    estHeight: 140,
    gap: 8,
  });

  const pick = useCallback(
    (i: number) => {
      const clamped = Math.max(0, Math.min(2, i));
      const next = ORDER[clamped]!;
      if (next !== value) {
        onChange(next);
        setParticles((n) => n + 1);
      }
    },
    [onChange, value],
  );

  const fromClientX = useCallback(
    (clientX: number) => {
      const el = trackRef.current;
      if (!el) return;
      const r = el.getBoundingClientRect();
      const t = (clientX - r.left) / Math.max(r.width, 1);
      pick(Math.round(t * 2));
    },
    [pick],
  );

  useEffect(() => {
    const onMove = (e: PointerEvent) => {
      if (!dragging.current) return;
      fromClientX(e.clientX);
    };
    const onUp = () => {
      dragging.current = false;
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
    return () => {
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
  }, [fromClientX]);

  const label = labels[value as EffortId] ?? value;

  const panel =
    open && pos && typeof document !== "undefined"
      ? createPortal(
          <div
            ref={panelRef}
            className="effort-panel__pop effort-panel__pop--portal"
            id={listId}
            role="dialog"
            style={style}
          >
            <div className="effort-panel__labels">
              {ORDER.map((id) => (
                <button
                  key={id}
                  type="button"
                  className={
                    "effort-panel__label" + (id === value ? " is-active" : "")
                  }
                  onClick={() => pick(indexOf(id))}
                >
                  {labels[id]}
                </button>
              ))}
            </div>
            <div
              ref={trackRef}
              className="effort-panel__track"
              onPointerDown={(e) => {
                dragging.current = true;
                (e.target as HTMLElement).setPointerCapture?.(e.pointerId);
                fromClientX(e.clientX);
              }}
            >
              <div className="effort-panel__rail" />
              <div
                className="effort-panel__fill"
                style={{ width: `${(idx / 2) * 100}%` }}
              />
              {[0, 1, 2].map((i) => (
                <span
                  key={i}
                  className={
                    "effort-panel__tick" + (i <= idx ? " is-on" : "")
                  }
                  style={{ left: `${(i / 2) * 100}%` }}
                />
              ))}
              <span
                className="effort-panel__thumb"
                style={{ left: `${(idx / 2) * 100}%` }}
              >
                {particles > 0 && (
                  <span
                    key={particles}
                    className="effort-panel__burst"
                    aria-hidden
                  >
                    {Array.from({ length: 8 }).map((_, i) => (
                      <i key={i} style={{ ["--i" as string]: i }} />
                    ))}
                  </span>
                )}
              </span>
            </div>
            <div className="effort-panel__hint">
              {GROK_BUILD_EFFORTS.map((e) => e.id).join(" · ")}
            </div>
          </div>,
          document.body,
        )
      : null;

  const tipLabel = title ?? ariaLabel;
  const chip = (
    <button
      ref={triggerRef}
      type="button"
      className="effort-panel__chip"
      aria-haspopup="dialog"
      aria-expanded={open}
      aria-controls={listId}
      aria-label={ariaLabel}
      onClick={() => setOpen((v) => !v)}
    >
      <span className="effort-panel__chip-dots" aria-hidden>
        {[0, 1, 2].map((i) => (
          <i key={i} className={i <= idx ? "is-on" : undefined} />
        ))}
      </span>
      {label}
      <span className="effort-panel__chev" aria-hidden>
        <IconChevronDown size={12} />
      </span>
    </button>
  );

  return (
    <div
      ref={rootRef}
      className={`effort-panel ${open ? "is-open" : ""}`}
    >
      {tipLabel ? <Tip label={tipLabel}>{chip}</Tip> : chip}
      {panel}
    </div>
  );
}
