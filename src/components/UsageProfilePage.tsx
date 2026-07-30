import { useEffect, useMemo, useState } from "react";
import { Heatmap } from "@/components/Heatmap";
import * as api from "@/lib/api";
import type { MessageKey, Vars } from "@/i18n";
import {
  buildUsageTimeline,
  type UsageTimelineView,
} from "@/lib/usageTimeline";

type T = (key: MessageKey | string, vars?: Vars) => string;

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "PI";
  return parts
    .slice(0, 2)
    .map((part) => part[0]?.toUpperCase() ?? "")
    .join("");
}

function compactNumber(value: number): string {
  return new Intl.NumberFormat("en", {
    notation: "compact",
    maximumFractionDigits: 1,
  }).format(value);
}

function duration(value: number): string {
  const hours = Math.floor(value / 3600);
  const minutes = Math.floor((value % 3600) / 60);
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

export function UsageProfilePage({
  t,
  userName,
  onUserName,
}: {
  t: T;
  userName: string;
  onUserName: (value: string) => void;
}) {
  const [profile, setProfile] = useState<api.UsageProfile | null>(null);
  const [loading, setLoading] = useState(true);
  const [draftName, setDraftName] = useState(userName);
  const [timelineView, setTimelineView] =
    useState<UsageTimelineView>("daily");

  useEffect(() => setDraftName(userName), [userName]);
  useEffect(() => {
    let cancelled = false;
    void api
      .usageProfile()
      .then((value) => {
        if (!cancelled) setProfile(value);
      })
      .catch(() => {
        if (!cancelled) setProfile(null);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const heatmap = useMemo(() => {
    const timeline = buildUsageTimeline(profile?.days ?? [], timelineView);
    return timeline.map((day) => ({
        date: day.date,
        requests: day.activities,
        tokens: day.estimatedTokens,
        costUsd: 0,
      }));
  }, [profile, timelineView]);

  const timelineViews: { id: UsageTimelineView; label: string }[] = [
    { id: "daily", label: t("usage.view.daily") },
    { id: "weekly", label: t("usage.view.weekly") },
    { id: "cumulative", label: t("usage.view.cumulative") },
  ];

  const stats = [
    {
      value: compactNumber(profile?.totalEstimatedTokens ?? 0),
      label: t("usage.totalTokens"),
    },
    {
      value: compactNumber(profile?.maxSessionTokens ?? 0),
      label: t("usage.maxSession"),
    },
    {
      value: duration(profile?.longestActivitySecs ?? 0),
      label: t("usage.longestActivity"),
    },
    {
      value: String(profile?.currentStreakDays ?? 0),
      label: t("usage.currentStreak"),
    },
    {
      value: String(profile?.longestStreakDays ?? 0),
      label: t("usage.longestStreak"),
    },
  ];

  const commitName = () => {
    const next = draftName.trim().slice(0, 80);
    if (!next || next === userName) {
      setDraftName(userName);
      return;
    }
    onUserName(next);
  };

  return (
    <section className="usage-profile" aria-busy={loading}>
      <header className="usage-profile__identity">
        <div className="usage-profile__monogram" aria-hidden>
          {initials(userName)}
        </div>
        <label className="usage-profile__name">
          <span className="sr-only">{t("usage.name")}</span>
          <input
            value={draftName}
            maxLength={80}
            placeholder={t("usage.namePlaceholder")}
            onChange={(event) => setDraftName(event.target.value)}
            onBlur={commitName}
            onKeyDown={(event) => {
              if (event.key === "Enter") event.currentTarget.blur();
              if (event.key === "Escape") {
                setDraftName(userName);
                event.currentTarget.blur();
              }
            }}
          />
        </label>
        <p>{t("usage.localProfile")}</p>
      </header>

      <div className="usage-profile__stats" aria-label={t("usage.summary")}>
        {stats.map((stat) => (
          <div className="usage-profile__stat" key={stat.label}>
            <strong>{loading ? "…" : stat.value}</strong>
            <span>{stat.label}</span>
          </div>
        ))}
      </div>

      <section className="usage-profile__activity">
        <div className="usage-profile__section-head">
          <h2>{t("usage.activity")}</h2>
          <div
            className="usage-profile__view-switcher"
            role="group"
            aria-label={t("usage.view.aria")}
          >
            {timelineViews.map((view) => (
              <button
                key={view.id}
                type="button"
                className={timelineView === view.id ? "is-active" : undefined}
                aria-pressed={timelineView === view.id}
                onClick={() => setTimelineView(view.id)}
              >
                {view.label}
              </button>
            ))}
          </div>
        </div>
        <Heatmap
          days={heatmap}
          metric="tokens"
          labels={{
            less: t("account.heatmap.less"),
            more: t("account.heatmap.more"),
            noData: loading ? t("common.loading") : t("usage.noActivity"),
            aria: t("usage.heatmapAria"),
            requests: t("usage.turns"),
            tokens: t("usage.estimatedTokens"),
          }}
        />
        <p className="usage-profile__scroll-hint">
          {t("usage.scrollHint")}
        </p>
      </section>

      <div className="usage-profile__details">
        <section>
          <h2>{t("usage.insights")}</h2>
          <dl>
            <div>
              <dt>{t("usage.reasoning")}</dt>
              <dd>{profile?.mostUsedEffort || t("usage.notRecorded")}</dd>
            </div>
            <div>
              <dt>{t("usage.sessions")}</dt>
              <dd>{profile?.totalSessions ?? 0}</dd>
            </div>
            <div>
              <dt>{t("usage.turns")}</dt>
              <dd>{profile?.totalTurns ?? 0}</dd>
            </div>
            <div>
              <dt>{t("usage.models")}</dt>
              <dd>{profile?.modelsUsed ?? 0}</dd>
            </div>
            <div>
              <dt>{t("usage.toolCalls")}</dt>
              <dd>{profile?.totalToolCalls ?? 0}</dd>
            </div>
          </dl>
        </section>

        <section>
          <h2>{t("usage.topTools")}</h2>
          {profile?.topTools.length ? (
            <ol className="usage-profile__tools">
              {profile.topTools.map((tool) => (
                <li key={tool.name}>
                  <code>{tool.name}</code>
                  <span>{t("usage.executions", { n: tool.count })}</span>
                </li>
              ))}
            </ol>
          ) : (
            <p className="usage-profile__empty">{t("usage.noTools")}</p>
          )}
        </section>
      </div>

      <p className="usage-profile__method">{t("usage.method")}</p>
    </section>
  );
}
