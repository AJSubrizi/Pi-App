import { ThinkingOrb } from "thinking-orbs";
import type { OrbState } from "thinking-orbs";
import { cn } from "@/lib/utils";

export type InlineOrbState = "thinking" | "working" | "searching" | "done";

const STATE_MAP: Record<InlineOrbState, OrbState> = {
  thinking: "composing",
  working: "working",
  searching: "searching",
  done: "solving",
};

export function ThinkingOrbInline({
  state = "thinking",
  paused = false,
  className,
  size = 20,
}: {
  state?: InlineOrbState;
  paused?: boolean;
  className?: string;
  size?: 20 | 64;
}) {
  return (
    <ThinkingOrb
      state={STATE_MAP[state]}
      size={size}
      theme="auto"
      paused={paused}
      className={cn("thinking-orb-inline", className)}
    />
  );
}
