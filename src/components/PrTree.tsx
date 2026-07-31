/**
 * PR workspace sidebar — repositories and their open pull requests.
 *
 * The Code workspace's projects/chats tree, aimed at a forge instead of a
 * filesystem. Selecting a PR hands it to Pi as a review chat.
 *
 * Presentational: fetching is the caller's job, so this stays testable and the
 * loading policy lives in one place.
 */

import {
  IconChevronDown,
  IconChevronRight,
  IconPlus,
  IconTrash,
} from "@/components/icons";
import type { GhPullRequest } from "@/lib/api";

export type PrRepoState = {
  slug: string;
  expanded: boolean;
  loading: boolean;
  /** Null until first load; empty array means "loaded, none open". */
  pulls: GhPullRequest[] | null;
  error: string | null;
};

export function PrTree({
  repos,
  activePr,
  labels,
  onToggle,
  onAddRepo,
  onRemoveRepo,
  onOpenPr,
}: {
  repos: PrRepoState[];
  /** `owner/name#number` of the PR whose review chat is open. */
  activePr: string | null;
  labels: {
    repos: string;
    addRepo: string;
    removeRepo: string;
    empty: string;
    noPulls: string;
    loading: string;
    draft: string;
  };
  onToggle: (slug: string) => void;
  onAddRepo: () => void;
  onRemoveRepo: (slug: string) => void;
  onOpenPr: (slug: string, pr: GhPullRequest) => void;
}) {
  return (
    <div className="pr-tree">
      <div className="tree-section__head">
        <span className="tree-section__label">{labels.repos}</span>
        <button
          type="button"
          className="tree-icon-btn"
          title={labels.addRepo}
          aria-label={labels.addRepo}
          onClick={onAddRepo}
        >
          <IconPlus size={14} />
        </button>
      </div>

      {repos.length === 0 ? (
        <p className="tree-empty">{labels.empty}</p>
      ) : (
        repos.map((repo) => (
          <div key={repo.slug} className="pr-tree__repo">
            <div className="pr-tree__repo-row">
              <button
                type="button"
                className="pr-tree__repo-btn"
                aria-expanded={repo.expanded}
                onClick={() => onToggle(repo.slug)}
              >
                <span className="pr-tree__chev" aria-hidden>
                  {repo.expanded ? (
                    <IconChevronDown size={13} />
                  ) : (
                    <IconChevronRight size={13} />
                  )}
                </span>
                <span className="pr-tree__repo-name">{repo.slug}</span>
                {repo.pulls && repo.pulls.length > 0 ? (
                  <span className="pr-tree__count">{repo.pulls.length}</span>
                ) : null}
              </button>
              <button
                type="button"
                className="tree-icon-btn"
                title={labels.removeRepo}
                aria-label={`${labels.removeRepo} ${repo.slug}`}
                onClick={() => onRemoveRepo(repo.slug)}
              >
                <IconTrash size={13} />
              </button>
            </div>

            {repo.expanded ? (
              <div className="pr-tree__pulls">
                {repo.loading ? (
                  <p className="tree-empty">{labels.loading}</p>
                ) : repo.error ? (
                  <p className="tree-empty tree-empty--error">{repo.error}</p>
                ) : !repo.pulls || repo.pulls.length === 0 ? (
                  <p className="tree-empty">{labels.noPulls}</p>
                ) : (
                  repo.pulls.map((pr) => {
                    const id = `${repo.slug}#${pr.number}`;
                    return (
                      <button
                        key={id}
                        type="button"
                        className={
                          "pr-tree__pr" +
                          (activePr === id ? " pr-tree__pr--active" : "")
                        }
                        title={`#${pr.number} ${pr.title}`}
                        onClick={() => onOpenPr(repo.slug, pr)}
                      >
                        <span className="pr-tree__pr-num">#{pr.number}</span>
                        <span className="pr-tree__pr-title">{pr.title}</span>
                        {pr.isDraft ? (
                          <span className="pr-tree__badge">{labels.draft}</span>
                        ) : null}
                      </button>
                    );
                  })
                )}
              </div>
            ) : null}
          </div>
        ))
      )}
    </div>
  );
}
