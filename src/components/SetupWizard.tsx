/**
 * Full-screen first-run gate: install Pi (required) → enter the workbench.
 * No page scrollbars; content is centered and compact.
 */

import { useCallback, useEffect, useMemo, useState } from "react";
import { PiLogo } from "@/components/PiLogo";
import { Spinner } from "@/components/ui/spinner";
import * as api from "@/lib/api";
import type { createT } from "@/i18n";

type Tr = ReturnType<typeof createT>;

export type SetupCliInfo = {
  found: boolean;
  path: string | null;
  version: string | null;
  source: string;
  cliAuthPresent: boolean;
};

type Step = "runtime" | "ready";

type Props = {
  tr: Tr;
  platform: "mac" | "win" | "other";
  useCustomWindowChrome: boolean;
  initialCli: SetupCliInfo;
  onComplete: (cli: SetupCliInfo) => void;
};

function mirrorHost(url: string | null | undefined): string {
  if (!url) return "";
  try {
    return new URL(url).host;
  } catch {
    return url.replace(/^https?:\/\//, "").split("/")[0] || url;
  }
}

export function SetupWizard({
  tr,
  platform,
  useCustomWindowChrome,
  initialCli,
  onComplete,
}: Props) {
  const [step, setStep] = useState<Step>(initialCli.found ? "ready" : "runtime");
  const [cli, setCli] = useState<SetupCliInfo>(initialCli);
  const [probing, setProbing] = useState(false);
  const [installing, setInstalling] = useState(false);
  const [progress, setProgress] = useState<api.CliInstallProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [statusMsg, setStatusMsg] = useState<string | null>(null);
  const [installCmds, setInstallCmds] = useState<api.CliInstallCommands | null>(
    null,
  );
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    void api.cliInstallCommands().then(setInstallCmds).catch(() => null);
  }, []);

  // Live install progress from Host
  useEffect(() => {
    if (!api.isTauri()) return;
    let unlisten: (() => void) | undefined;
    let cancelled = false;
    void (async () => {
      try {
        const { listen } = await import("@tauri-apps/api/event");
        unlisten = await listen<api.CliInstallProgress>(
          "setup://cli-install-progress",
          (ev) => {
            if (!cancelled) setProgress(ev.payload);
          },
        );
      } catch {
        /* ignore */
      }
    })();
    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  const recheck = useCallback(async (manualPath?: string | null) => {
    setProbing(true);
    setError(null);
    try {
      const r = await api.probeCli(manualPath || undefined);
      const next: SetupCliInfo = {
        found: r.found,
        path: r.path,
        version: r.version,
        source: r.source || "",
        cliAuthPresent: !!r.cliAuthPresent,
      };
      setCli(next);
      if (next.found) {
        setStatusMsg(null);
      }
      return next;
    } catch (e) {
      setError(String(e));
      return null;
    } finally {
      setProbing(false);
    }
  }, []);

  // Soft auto-detect once when opening runtime step without CLI
  useEffect(() => {
    if (step !== "runtime" || cli.found) return;
    void recheck();
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const runInstall = useCallback(async () => {
    if (installing) return;
    setInstalling(true);
    setError(null);
    setProgress({
      phase: "resolving",
      message: tr("setup.detecting"),
      percent: 0,
    });
    try {
      const res = await api.cliInstallLatest();
      if (!res.ok) {
        setError(res.message || tr("setup.error"));
        return;
      }
      const next = await recheck(res.path);
      if (next?.found) {
        setStep("ready");
      } else {
        setError(tr("setup.cli.missing"));
      }
    } catch (e) {
      const msg = String(e);
      setError(msg);
      setProgress((p) =>
        p
          ? { ...p, phase: "error", message: msg }
          : { phase: "error", message: msg },
      );
    } finally {
      setInstalling(false);
    }
  }, [installing, recheck, tr]);

  const pickBinary = useCallback(async () => {
    setError(null);
    try {
      const path = await api.pickCliBinary();
      if (!path) return;
      await api.settingsGet().then((s) =>
        api.settingsSet({ ...s, manualCliPath: path }),
      );
      const next = await recheck(path);
      if (next?.found) {
        setStep("ready");
      } else {
        setError(tr("setup.cli.missing"));
      }
    } catch (e) {
      setError(String(e));
    }
  }, [recheck, tr]);

  const copyCmd = useCallback(async () => {
    const cmd = installCmds?.primary;
    if (!cmd) return;
    try {
      await navigator.clipboard.writeText(cmd);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1800);
    } catch {
      setError(tr("setup.error"));
    }
  }, [installCmds, tr]);

  const openDocs = useCallback(() => {
    const url = installCmds?.docsUrl || "https://pi.dev/docs/latest";
    void api.openExternalUrl(url).catch((e) => setError(String(e)));
  }, [installCmds]);

  const finishWizard = useCallback(async () => {
    try {
      const s = await api.settingsGet();
      await api.settingsSet({
        ...s,
        setupWizardCompleted: true,
        authSetupDeferred: false,
        onboardingDone: true,
        setupSkipped: false,
      });
    } catch {
      /* still enter if probe succeeded */
    }
    onComplete(cli);
  }, [cli, onComplete]);

  const percent = useMemo(() => {
    const p = progress?.percent;
    if (p == null || Number.isNaN(p)) return installing ? 8 : 0;
    return Math.max(0, Math.min(100, Math.round(p)));
  }, [progress, installing]);

  const stepIndex = step === "runtime" ? 0 : 1;

  return (
    <div
      className={
        "setup-gate" +
        (useCustomWindowChrome ? " setup-gate--custom-chrome" : "")
      }
      data-platform={platform}
      data-testid="setup-wizard"
    >
      <div className="setup-gate__drag" data-tauri-drag-region />

      <div className="setup-gate__center">
        <div className="setup-hero">
          <div
            className={
              "setup-logo" +
              (installing || probing ? " setup-logo--spin" : " setup-logo--pulse")
            }
          >
            <PiLogo size={44} />
          </div>
          <h1 className="setup-title">{tr("setup.title")}</h1>
          <p className="setup-subtitle">{tr("setup.subtitle")}</p>
        </div>

        <ol className="setup-steps" aria-label="Setup steps">
          {(
            [
              ["runtime", "setup.step.runtime"],
              ["ready", "setup.step.ready"],
            ] as const
          ).map(([id, key], i) => (
            <li
              key={id}
              className={
                "setup-steps__item" +
                (i === stepIndex ? " is-active" : "") +
                (i < stepIndex ? " is-done" : "")
              }
            >
              <span className="setup-steps__dot" />
              <span className="setup-steps__label">{tr(key)}</span>
            </li>
          ))}
        </ol>

        <div className="setup-card">
          {step === "runtime" && (
            <>
              <div className="setup-card__head">
                <h2>
                  {cli.found
                    ? tr("setup.cli.found")
                    : tr("setup.cli.required")}
                </h2>
                <p>
                  {cli.found
                    ? tr("setup.cli.foundHint", {
                        version: cli.version || "—",
                      })
                    : tr("setup.cli.requiredHint")}
                </p>
                {cli.path && (
                  <p className="setup-mono">
                    {tr("setup.cli.path", { path: cli.path })}
                  </p>
                )}
              </div>

              {(installing || progress) && (
                <div className="setup-progress" aria-live="polite">
                  <div className="setup-progress__track">
                    <div
                      className="setup-progress__fill"
                      style={{ width: `${percent}%` }}
                    />
                  </div>
                  <div className="setup-progress__meta">
                    <span>
                      {progress?.message ||
                        (installing
                          ? tr("setup.installing")
                          : tr("setup.detecting"))}
                    </span>
                    <span>{tr("setup.progress", { percent })}</span>
                  </div>
                  {progress?.mirror && (
                    <div className="setup-progress__mirror">
                      {tr("setup.mirror", {
                        host: mirrorHost(progress.mirror),
                      })}
                    </div>
                  )}
                </div>
              )}

              {!cli.found && !installing && (
                <p className="setup-hint">{tr("setup.manualHint")}</p>
              )}

              <div className="setup-actions">
                {cli.found ? (
                  <button
                    type="button"
                    className="btn btn--primary setup-btn-primary"
                    onClick={() => setStep("ready")}
                  >
                    {tr("setup.continue")}
                  </button>
                ) : (
                  <button
                    type="button"
                    className="btn btn--primary setup-btn-primary"
                    disabled={installing || probing}
                    onClick={() => void runInstall()}
                  >
                    {installing ? (
                      <>
                        <Spinner className="size-4" />
                        {tr("setup.installing")}
                      </>
                    ) : (
                      tr("setup.install")
                    )}
                  </button>
                )}
                <div className="setup-actions__row">
                  <button
                    type="button"
                    className="btn btn--ghost"
                    disabled={installing || probing}
                    onClick={() => void recheck()}
                  >
                    {probing ? <Spinner className="size-3.5" /> : null}
                    {tr("setup.recheck")}
                  </button>
                  <button
                    type="button"
                    className="btn btn--ghost"
                    disabled={installing}
                    onClick={() => void pickBinary()}
                  >
                    {tr("setup.pickBinary")}
                  </button>
                </div>
                <div className="setup-actions__row">
                  <button
                    type="button"
                    className="btn btn--ghost"
                    disabled={!installCmds?.primary}
                    onClick={() => void copyCmd()}
                  >
                    {copied ? tr("setup.copied") : tr("setup.copyCmd")}
                  </button>
                  <button
                    type="button"
                    className="btn btn--ghost"
                    onClick={openDocs}
                  >
                    {tr("setup.openDocs")}
                  </button>
                </div>
                {installCmds?.primary && (
                  <code className="setup-cmd">{installCmds.primary}</code>
                )}
              </div>
            </>
          )}

          {step === "ready" && (
            <>
              <div className="setup-card__head">
                <h2>{tr("setup.ready.title")}</h2>
              </div>
              <ul className="setup-checklist">
                <li className="is-ok">
                  <span className="setup-check" />
                  {tr("setup.ready.cliOk")}
                  {cli.version ? (
                    <span className="setup-check-meta">{cli.version}</span>
                  ) : null}
                </li>
                <li className="is-ok">
                  <span className="setup-check" />
                  {tr("setup.ready.authOk")}
                </li>
              </ul>
              <div className="setup-actions">
                <button
                  type="button"
                  className="btn btn--primary setup-btn-primary"
                  disabled={!cli.found}
                  onClick={() => void finishWizard()}
                >
                  {tr("setup.ready.enter")}
                </button>
              </div>
            </>
          )}

          {error && (
            <div className="setup-error" role="alert">
              <strong>{tr("setup.error")}</strong>
              <span>{error}</span>
              {/network|timeout|mirror|download|HTTP|failed/i.test(error) && (
                <span className="setup-error__hint">
                  {tr("setup.networkHint")}
                </span>
              )}
            </div>
          )}
          {statusMsg && !error && (
            <div className="setup-status" role="status">
              {statusMsg}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
