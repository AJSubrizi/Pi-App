// @vitest-environment happy-dom

import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { CacheChip } from "./CacheChip";
import type { UsagePayload } from "@/lib/cacheUsage";

afterEach(cleanup);

const labels = {
  title: "Cache",
  tipCold: "Cache barely paying off",
  tipWarming: "Cache warming up",
  tipGood: "Cache working well",
  prompt: "Prompt",
  cached: "Cached",
  output: "Output",
  cost: "Cost",
};

function payload(over: Partial<UsagePayload> = {}): UsagePayload {
  const zero = {
    input: 0,
    output: 0,
    cacheRead: 0,
    cacheWrite: 0,
    totalTokens: 0,
  };
  return {
    sessionId: "s1",
    turn: zero,
    total: {
      input: 2000,
      output: 500,
      cacheRead: 8000,
      cacheWrite: 0,
      totalTokens: 10_500,
      costTotal: 0.42,
    },
    cacheHitRate: 0.8,
    ...over,
  };
}

describe("CacheChip", () => {
  it("shows the measured rate", () => {
    render(<CacheChip usage={payload()} labels={labels} />);
    expect(screen.getByText("80%")).toBeDefined();
  });

  it("renders nothing before anything has been measured", () => {
    const { container } = render(<CacheChip usage={null} labels={labels} />);
    expect(container.textContent).toBe("");
  });

  it("renders nothing when no prompt tokens were counted", () => {
    const { container } = render(
      <CacheChip usage={payload({ cacheHitRate: null })} labels={labels} />,
    );
    expect(container.textContent).toBe("");
  });

  it("carries the breakdown for screen readers, not just the tooltip", () => {
    render(<CacheChip usage={payload()} labels={labels} />);
    const label = screen.getByLabelText(/Cache 80%/);
    expect(label.getAttribute("aria-label")).toContain("Cached: 8k");
    expect(label.getAttribute("aria-label")).toContain("Cost: $0.42");
  });

  it("marks a cold cache differently from a good one", () => {
    const { container, rerender } = render(
      <CacheChip
        usage={payload({
          cacheHitRate: 0,
          total: {
            input: 5000,
            output: 0,
            cacheRead: 0,
            cacheWrite: 0,
            totalTokens: 5000,
            costTotal: null,
          },
        })}
        labels={labels}
      />,
    );
    expect(container.querySelector(".cache-chip--cold")).not.toBeNull();

    rerender(<CacheChip usage={payload()} labels={labels} />);
    expect(container.querySelector(".cache-chip--good")).not.toBeNull();
  });

  it("omits cost when the provider reported none", () => {
    render(
      <CacheChip
        usage={payload({
          total: {
            input: 2000,
            output: 0,
            cacheRead: 8000,
            cacheWrite: 0,
            totalTokens: 10_000,
            costTotal: null,
          },
        })}
        labels={labels}
      />,
    );
    expect(screen.getByLabelText(/Cache/).getAttribute("aria-label")).not.toContain(
      "Cost",
    );
  });
});
