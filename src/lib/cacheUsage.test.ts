import { describe, expect, it } from "vitest";
import {
  cacheChipView,
  cacheStanding,
  formatCacheRate,
  formatCost,
  formatTokens,
  type UsagePayload,
} from "./cacheUsage";

describe("cacheStanding", () => {
  it("splits the range into cost-meaningful bands", () => {
    expect(cacheStanding(0)).toBe("cold");
    expect(cacheStanding(0.33)).toBe("cold");
    expect(cacheStanding(0.34)).toBe("warming");
    expect(cacheStanding(0.66)).toBe("warming");
    expect(cacheStanding(0.67)).toBe("good");
    expect(cacheStanding(1)).toBe("good");
  });
});

describe("formatCacheRate", () => {
  it("rounds to whole percent", () => {
    expect(formatCacheRate(0.923)).toBe("92%");
    expect(formatCacheRate(0.5)).toBe("50%");
  });

  it("never reports a real hit as zero, or a partial miss as perfect", () => {
    expect(formatCacheRate(0.004)).toBe("<1%");
    expect(formatCacheRate(0.999)).toBe(">99%");
    expect(formatCacheRate(0)).toBe("0%");
    expect(formatCacheRate(1)).toBe("100%");
  });
});

describe("formatTokens", () => {
  it("keeps small counts exact", () => {
    expect(formatTokens(0)).toBe("0");
    expect(formatTokens(999)).toBe("999");
  });

  it("compacts thousands and millions", () => {
    expect(formatTokens(1234)).toBe("1.2k");
    expect(formatTokens(45_000)).toBe("45k");
    expect(formatTokens(1_200_000)).toBe("1.2M");
    expect(formatTokens(8000)).toBe("8k");
    expect(formatTokens(2_000_000)).toBe("2M");
    expect(formatTokens(12_000_000)).toBe("12M");
  });

  it("does not produce nonsense from bad input", () => {
    expect(formatTokens(-5)).toBe("0");
    expect(formatTokens(NaN)).toBe("0");
  });
});

describe("formatCost", () => {
  it("distinguishes free, negligible and real", () => {
    expect(formatCost(0)).toBe("$0.00");
    expect(formatCost(0.004)).toBe("<$0.01");
    expect(formatCost(1.239)).toBe("$1.24");
  });

  it("returns null when the provider reported nothing", () => {
    expect(formatCost(null)).toBeNull();
    expect(formatCost(undefined)).toBeNull();
    expect(formatCost(NaN)).toBeNull();
  });
});

function payload(over: Partial<UsagePayload> = {}): UsagePayload {
  return {
    sessionId: "s1",
    turn: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 0,
    },
    total: {
      input: 2000,
      output: 500,
      cacheRead: 8000,
      cacheWrite: 100,
      totalTokens: 10_500,
      costTotal: 0.42,
    },
    cacheHitRate: 0.8,
    ...over,
  };
}

describe("cacheChipView", () => {
  it("summarises a measured session", () => {
    const v = cacheChipView(payload())!;
    expect(v.rateLabel).toBe("80%");
    expect(v.standing).toBe("good");
    expect(v.promptTokens).toBe("10k"); // input + cacheRead
    expect(v.cachedTokens).toBe("8k");
    expect(v.outputTokens).toBe("500");
    expect(v.costLabel).toBe("$0.42");
  });

  /**
   * Before the first billed turn there is nothing to report, and a "0%" badge
   * would read as a cache that is failing rather than one not yet exercised.
   */
  it("shows nothing until prompt tokens have been measured", () => {
    expect(cacheChipView(null)).toBeNull();
    expect(cacheChipView(payload({ cacheHitRate: null }))).toBeNull();
  });

  it("reports a genuinely cold cache rather than hiding it", () => {
    const v = cacheChipView(
      payload({
        cacheHitRate: 0,
        total: {
          input: 5000,
          output: 100,
          cacheRead: 0,
          cacheWrite: 0,
          totalTokens: 5100,
          costTotal: null,
        },
      }),
    )!;
    expect(v.rateLabel).toBe("0%");
    expect(v.standing).toBe("cold");
    expect(v.costLabel).toBeNull();
  });
});
