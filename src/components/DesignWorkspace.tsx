import { useState } from "react";
import type { Locale } from "@/i18n";
import { EmbeddedBrowser } from "@/components/EmbeddedBrowser";

type T = (key: string, vars?: Record<string, string | number>) => string;

export function DesignWorkspace({ t, locale }: { t: T; locale: Locale }) {
  const [draft, setDraft] = useState("");
  const [url, setUrl] = useState("");
  return (
    <section className="design-workspace" aria-label={t("design.title")}>
      <header className="design-workspace__head">
        <div>
          <h2>{t("design.title")}</h2>
          <p>{t("design.description")}</p>
        </div>
        <form onSubmit={(event) => {
          event.preventDefault();
          const value = draft.trim();
          if (value) setUrl(/^[a-z]+:\/\//i.test(value) ? value : `http://${value}`);
        }}>
          <input value={draft} onChange={(event) => setDraft(event.target.value)} placeholder={t("design.urlPlaceholder")} aria-label={t("browser.address")} />
          <button type="submit" className="btn btn--solid">{t("rp.browser.go")}</button>
        </form>
      </header>
      {url ? (
        <EmbeddedBrowser
          url={url}
          locale={locale}
          title={t("design.title")}
          onNavigate={setUrl}
          navLabels={{
            back: t("browser.back"),
            forward: t("browser.forward"),
            address: t("browser.address"),
            soon: t("browser.soon"),
          }}
          className="design-workspace__browser"
        />
      ) : (
        <div className="design-workspace__empty">{t("design.empty")}</div>
      )}
    </section>
  );
}

