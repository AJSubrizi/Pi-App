/**
 * Presenting the real token usage Pi reports.
 *
 * The host emits `session://usage` with the turn, a running session total and
 * a cache hit rate. This module decides what is worth showing — pure, so the
 * judgement calls are testable without a running agent.
 */

export type TokenUsage = {
  input: number;
  output: number;
  cacheRead: number;
  cacheWrite: number;
  totalTokens: number;
  costTotal?: number | null;
};

export type UsagePayload = {
  sessionId: string;
  turn: TokenUsage;
  total: TokenUsage;
  /** `0..1`, or null when no prompt tokens have been counted yet. */
  cacheHitRate: number | null;
};

/** How the chip should read the rate. */
export type CacheStanding = "cold" | "warming" | "good";

/**
 * Classify a hit rate.
 *
 * The bands are about cost, not neatness. Below a third, the cache is barely
 * paying for itself; past two thirds a long session is cheap. The middle is
 * genuinely "it depends", so it gets its own, non-alarming label.
 */
export function cacheStanding(rate: number): CacheStanding {
  if (rate < 0.34) return "cold";
  if (rate < 0.67) return "warming";
  return "good";
}

/** Whole-percent rate for display; never rounds a real hit down to 0%. */
export function formatCacheRate(rate: number): string {
  const pct = rate * 100;
  if (pct > 0 && pct < 1) return "<1%";
  if (pct < 100 && pct > 99) return ">99%";
  return `${Math.round(pct)}%`;
}

/**
 * Compact token count: 1_234 → "1.2k", 1_200_000 → "1.2M".
 *
 * The chip has one line; an exact count of 1,234,567 would push everything
 * else off it.
 */
export function formatTokens(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "0";
  if (n < 1000) return String(Math.round(n));
  // One decimal below ten, none above — and never a bare ".0", which reads
  // as spurious precision on a round number.
  const short = (v: number) =>
    v < 10 ? v.toFixed(1).replace(/\.0$/, "") : String(Math.round(v));
  if (n < 1_000_000) return `${short(n / 1000)}k`;
  return `${short(n / 1_000_000)}M`;
}

/** Provider cost, or null when they did not report one. */
export function formatCost(cost: number | null | undefined): string | null {
  if (cost === null || cost === undefined || !Number.isFinite(cost)) return null;
  if (cost === 0) return "$0.00";
  if (cost < 0.01) return "<$0.01";
  return `$${cost.toFixed(2)}`;
}

export type CacheChipView = {
  rateLabel: string;
  standing: CacheStanding;
  promptTokens: string;
  cachedTokens: string;
  outputTokens: string;
  costLabel: string | null;
};

/**
 * Everything the chip renders, or `null` when there is nothing honest to say.
 *
 * A session with no measured prompt tokens shows no chip at all: a "0%" badge
 * before the first billed turn would read as a cache that is failing, when in
 * fact nothing has been measured.
 */
export function cacheChipView(payload: UsagePayload | null): CacheChipView | null {
  if (!payload) return null;
  const { total, cacheHitRate } = payload;
  if (cacheHitRate === null) return null;

  return {
    rateLabel: formatCacheRate(cacheHitRate),
    standing: cacheStanding(cacheHitRate),
    promptTokens: formatTokens(total.input + total.cacheRead),
    cachedTokens: formatTokens(total.cacheRead),
    outputTokens: formatTokens(total.output),
    costLabel: formatCost(total.costTotal),
  };
}
