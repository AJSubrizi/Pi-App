/**
 * Cache hit rate for the current session, next to the context chip.
 *
 * Fed by `session://usage`, which carries the figures Pi already reports —
 * these are measured, not the chars/4 estimate the shell shows elsewhere.
 */

import { Tip } from "@/components/ui/tooltip";
import { IconActivity } from "@/components/icons";
import { cacheChipView, type UsagePayload } from "@/lib/cacheUsage";

export type CacheChipLabels = {
  /** e.g. "Cache" */
  title: string;
  tipCold: string;
  tipWarming: string;
  tipGood: string;
  prompt: string;
  cached: string;
  output: string;
  cost: string;
};

export function CacheChip({
  usage,
  labels,
}: {
  usage: UsagePayload | null;
  labels: CacheChipLabels;
}) {
  const view = cacheChipView(usage);
  // Nothing measured yet: no chip, rather than a 0% that reads as failure.
  if (!view) return null;

  const tip =
    view.standing === "good"
      ? labels.tipGood
      : view.standing === "warming"
        ? labels.tipWarming
        : labels.tipCold;

  const detail = [
    `${labels.prompt}: ${view.promptTokens}`,
    `${labels.cached}: ${view.cachedTokens}`,
    `${labels.output}: ${view.outputTokens}`,
    view.costLabel ? `${labels.cost}: ${view.costLabel}` : null,
  ]
    .filter(Boolean)
    .join(" · ");

  return (
    <Tip label={`${tip}\n${detail}`}>
      <span
        className={`cache-chip cache-chip--${view.standing}`}
        aria-label={`${labels.title} ${view.rateLabel} — ${detail}`}
      >
        <IconActivity size={13} />
        <span className="cache-chip__rate">{view.rateLabel}</span>
      </span>
    </Tip>
  );
}
