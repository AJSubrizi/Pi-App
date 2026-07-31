//! GitHub CLI (`gh`) bridge — pull-request review and creation.
//!
//! The shell composes primitives rather than reimplementing them: `gh` already
//! knows about auth, remotes and the API, so the app shells out to it and hands
//! the result to Pi as ordinary prompt text.
//!
//! Nothing here mutates a remote except [`gh_pr_create`], which the UI must
//! gate behind an explicit confirmation.

use serde::{Deserialize, Serialize};

/// Cap on diff text handed to the agent. A 300k-line PR would blow the context
/// window and cost a fortune; truncation is reported so the prompt can say so.
const MAX_DIFF_BYTES: usize = 400_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhAvailability {
    pub installed: bool,
    /// `gh auth status` succeeded — a token is present for this host.
    pub authenticated: bool,
    pub version: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhPullRequest {
    pub number: u64,
    pub title: String,
    pub author: String,
    pub base_ref: String,
    pub head_ref: String,
    pub is_draft: bool,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhRepo {
    /// `owner/name` — the handle every other call takes.
    pub name_with_owner: String,
    pub name: String,
    pub owner: String,
    pub description: String,
    pub is_private: bool,
    pub updated_at: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhPrDiff {
    pub number: u64,
    pub title: String,
    pub body: String,
    pub base_ref: String,
    pub head_ref: String,
    pub url: String,
    pub diff: String,
    /// True when `diff` was cut at [`MAX_DIFF_BYTES`].
    pub truncated: bool,
    pub changed_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GhOpResult {
    pub ok: bool,
    pub output: String,
    pub reason: Option<String>,
}

/// What a `gh` call is aimed at.
///
/// The PR workspace has no checkout to run inside, so it addresses repositories
/// by `owner/name` and runs from the home directory. The Code workspace keeps
/// using the project directory, where `gh` infers the repo from the remote.
#[derive(Debug, Clone)]
pub enum GhTarget {
    /// A local checkout; `gh` reads the repo from its git remote.
    Project(String),
    /// An explicit `owner/name`, used with `--repo`.
    Repo(String),
}

impl GhTarget {
    /// `project_path` wins when both are supplied, so existing callers keep
    /// their behaviour unchanged.
    pub fn resolve(project_path: Option<String>, repo: Option<String>) -> Result<Self, String> {
        let project = project_path.unwrap_or_default();
        let project = project.trim();
        if !project.is_empty() {
            return Ok(GhTarget::Project(project.to_string()));
        }
        let repo = repo.unwrap_or_default();
        let repo = repo.trim();
        if repo.is_empty() {
            return Err("Either a project path or an owner/name repository is required".into());
        }
        if !is_repo_slug(repo) {
            return Err(format!("Not an owner/name repository: {repo}"));
        }
        Ok(GhTarget::Repo(repo.to_string()))
    }
}

/// Accept `owner/name` only — never a flag, a path, or a URL fragment.
///
/// The value reaches a subprocess argument, so anything shaped oddly is
/// rejected rather than escaped.
pub fn is_repo_slug(raw: &str) -> bool {
    let s = raw.trim();
    let Some((owner, name)) = s.split_once('/') else {
        return false;
    };
    if owner.is_empty() || name.is_empty() || name.contains('/') {
        return false;
    }
    let ok = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.');
    owner.chars().all(ok) && name.chars().all(ok) && !s.starts_with('-')
}

fn run_gh_target(target: &GhTarget, args: &[&str]) -> Result<(bool, String, String), String> {
    match target {
        GhTarget::Project(path) => run_gh(path, args),
        GhTarget::Repo(slug) => {
            let mut full: Vec<&str> = args.to_vec();
            full.push("--repo");
            full.push(slug);
            // No checkout to sit in; home is always a valid cwd.
            let home = crate::process_util::user_home();
            run_gh(&home.to_string_lossy(), &full)
        }
    }
}

fn run_gh(project: &str, args: &[&str]) -> Result<(bool, String, String), String> {
    let mut cmd = std::process::Command::new("gh");
    cmd.current_dir(project);
    for a in args {
        cmd.arg(a);
    }
    // `gh` paginates into a pager when it thinks it has a TTY.
    cmd.env("GH_PAGER", "");
    cmd.env("NO_COLOR", "1");
    let out = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            "GH_NOT_FOUND: the GitHub CLI (gh) is not installed or not on PATH".to_string()
        } else {
            e.to_string()
        }
    })?;
    Ok((
        out.status.success(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).trim().to_string(),
    ))
}

/// Cut `diff` to `max` bytes on a line boundary. Returns `(text, truncated)`.
///
/// Splitting mid-line would hand the agent a corrupt hunk header, so the cut
/// walks back to the last newline within the budget.
pub fn truncate_diff(diff: &str, max: usize) -> (String, bool) {
    if diff.len() <= max {
        return (diff.to_string(), false);
    }
    // Slicing by raw byte index panics when `max` lands inside a multi-byte
    // character, and diffs carry non-ASCII paths and content routinely.
    let mut end = max;
    while end > 0 && !diff.is_char_boundary(end) {
        end -= 1;
    }
    let cut = diff[..end].rfind('\n').unwrap_or(end);
    (diff[..cut].to_string(), true)
}

/// Is `gh` installed and authenticated for this project's remote?
#[tauri::command]
pub async fn gh_available(project_path: Option<String>) -> Result<GhAvailability, String> {
    // Works with no project: the PR workspace has no checkout to stand in.
    let cwd = project_path
        .filter(|p| !p.trim().is_empty())
        .unwrap_or_else(|| {
            crate::process_util::user_home()
                .to_string_lossy()
                .to_string()
        });
    let version = match run_gh(&cwd, &["--version"]) {
        Ok((true, stdout, _)) => stdout.lines().next().map(|l| l.trim().to_string()),
        Ok((false, _, stderr)) => {
            return Ok(GhAvailability {
                installed: false,
                authenticated: false,
                version: None,
                reason: Some(stderr),
            })
        }
        Err(e) => {
            return Ok(GhAvailability {
                installed: false,
                authenticated: false,
                version: None,
                reason: Some(e),
            })
        }
    };
    let (auth_ok, _, auth_err) = run_gh(&cwd, &["auth", "status"])?;
    Ok(GhAvailability {
        installed: true,
        authenticated: auth_ok,
        version,
        reason: if auth_ok { None } else { Some(auth_err) },
    })
}

/// Open pull requests for the project's repository.
#[tauri::command]
pub async fn gh_pr_list(
    project_path: Option<String>,
    repo: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<GhPullRequest>, String> {
    let target = GhTarget::resolve(project_path, repo)?;
    let limit = limit.unwrap_or(30).clamp(1, 100).to_string();
    let (ok, stdout, stderr) = run_gh_target(
        &target,
        &[
            "pr",
            "list",
            "--limit",
            &limit,
            "--json",
            "number,title,author,baseRefName,headRefName,isDraft,url",
        ],
    )?;
    if !ok {
        return Err(stderr);
    }
    let raw: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| e.to_string())?;
    let items = raw.as_array().cloned().unwrap_or_default();
    Ok(items
        .iter()
        .map(|v| GhPullRequest {
            number: v.get("number").and_then(|n| n.as_u64()).unwrap_or(0),
            title: v
                .get("title")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string(),
            author: v
                .get("author")
                .and_then(|a| a.get("login"))
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string(),
            base_ref: v
                .get("baseRefName")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string(),
            head_ref: v
                .get("headRefName")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string(),
            is_draft: v.get("isDraft").and_then(|b| b.as_bool()).unwrap_or(false),
            url: v
                .get("url")
                .and_then(|s| s.as_str())
                .unwrap_or_default()
                .to_string(),
        })
        .filter(|p| p.number > 0)
        .collect())
}

/// Repositories the signed-in user can reach, most recently updated first.
#[tauri::command]
pub async fn gh_repo_list(limit: Option<u32>) -> Result<Vec<GhRepo>, String> {
    let limit = limit.unwrap_or(60).clamp(1, 200).to_string();
    let home = crate::process_util::user_home();
    let (ok, stdout, stderr) = run_gh(
        &home.to_string_lossy(),
        &[
            "repo",
            "list",
            "--limit",
            &limit,
            "--json",
            "nameWithOwner,name,owner,description,isPrivate,updatedAt,url",
        ],
    )?;
    if !ok {
        return Err(stderr);
    }
    let raw: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| e.to_string())?;
    let s = |v: &serde_json::Value, k: &str| {
        v.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or_default()
            .to_string()
    };
    Ok(raw
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|v| GhRepo {
            name_with_owner: s(v, "nameWithOwner"),
            name: s(v, "name"),
            owner: v
                .get("owner")
                .and_then(|o| o.get("login"))
                .and_then(|x| x.as_str())
                .unwrap_or_default()
                .to_string(),
            description: s(v, "description"),
            is_private: v
                .get("isPrivate")
                .and_then(|b| b.as_bool())
                .unwrap_or(false),
            updated_at: s(v, "updatedAt"),
            url: s(v, "url"),
        })
        .filter(|r| !r.name_with_owner.is_empty())
        .collect())
}

/// Metadata + unified diff for one pull request, ready to become prompt text.
#[tauri::command]
pub async fn gh_pr_diff(
    project_path: Option<String>,
    repo: Option<String>,
    number: u64,
) -> Result<GhPrDiff, String> {
    let target = GhTarget::resolve(project_path, repo)?;
    let num = number.to_string();
    let (ok, meta_raw, stderr) = run_gh_target(
        &target,
        &[
            "pr",
            "view",
            &num,
            "--json",
            "number,title,body,baseRefName,headRefName,url,changedFiles",
        ],
    )?;
    if !ok {
        return Err(stderr);
    }
    let meta: serde_json::Value = serde_json::from_str(&meta_raw).map_err(|e| e.to_string())?;

    let (diff_ok, diff_raw, diff_err) = run_gh_target(&target, &["pr", "diff", &num])?;
    if !diff_ok {
        return Err(diff_err);
    }
    let (diff, truncated) = truncate_diff(&diff_raw, MAX_DIFF_BYTES);

    let s = |k: &str| {
        meta.get(k)
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string()
    };
    Ok(GhPrDiff {
        number: meta
            .get("number")
            .and_then(|n| n.as_u64())
            .unwrap_or(number),
        title: s("title"),
        body: s("body"),
        base_ref: s("baseRefName"),
        head_ref: s("headRefName"),
        url: s("url"),
        diff,
        truncated,
        changed_files: meta
            .get("changedFiles")
            .and_then(|n| n.as_u64())
            .unwrap_or(0) as u32,
    })
}

/// Open a pull request for the current branch.
///
/// **Publishes to the remote.** Callers must confirm with the user first.
#[tauri::command]
pub async fn gh_pr_create(
    project_path: String,
    title: String,
    body: String,
    draft: bool,
    base: Option<String>,
) -> Result<GhOpResult, String> {
    let title = title.trim();
    if title.is_empty() {
        return Ok(GhOpResult {
            ok: false,
            output: String::new(),
            reason: Some("A pull request title is required".into()),
        });
    }
    let mut args: Vec<&str> = vec!["pr", "create", "--title", title, "--body", &body];
    if draft {
        args.push("--draft");
    }
    let base_ref = base.unwrap_or_default();
    let base_ref = base_ref.trim();
    if !base_ref.is_empty() {
        args.push("--base");
        args.push(base_ref);
    }
    let (ok, stdout, stderr) = run_gh(&project_path, &args)?;
    let output = [stdout.trim(), stderr.as_str()]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    Ok(GhOpResult {
        ok,
        output: output.clone(),
        reason: if ok {
            None
        } else {
            Some(if output.is_empty() {
                "gh pr create failed".into()
            } else {
                output.chars().take(400).collect()
            })
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repo_slug_accepts_owner_name_only() {
        assert!(is_repo_slug("AJSubrizi/Pi-App"));
        assert!(is_repo_slug("owner/name.with.dots"));
        assert!(is_repo_slug(" owner/name "));
    }

    /// The slug becomes a subprocess argument, so shapes that could be read as
    /// a flag or a path are rejected outright rather than escaped.
    #[test]
    fn repo_slug_rejects_anything_that_is_not_owner_name() {
        for bad in [
            "",
            "noslash",
            "/name",
            "owner/",
            "owner/name/extra",
            "--repo=evil",
            "-x/y",
            "owner/na me",
            "owner/na;me",
            "https://github.com/o/r",
        ] {
            assert!(!is_repo_slug(bad), "should reject {bad:?}");
        }
    }

    #[test]
    fn target_prefers_the_project_path_when_both_are_given() {
        let t = GhTarget::resolve(Some("/tmp/p".into()), Some("o/r".into())).unwrap();
        assert!(matches!(t, GhTarget::Project(p) if p == "/tmp/p"));
    }

    #[test]
    fn target_falls_back_to_the_repo_slug() {
        let t = GhTarget::resolve(Some("   ".into()), Some("o/r".into())).unwrap();
        assert!(matches!(t, GhTarget::Repo(r) if r == "o/r"));
    }

    #[test]
    fn target_requires_one_of_the_two() {
        assert!(GhTarget::resolve(None, None).is_err());
        assert!(GhTarget::resolve(None, Some("not a slug".into())).is_err());
    }

    #[test]
    fn short_diff_is_untouched() {
        let (text, cut) = truncate_diff("diff --git a/x b/x\n+one\n", 1000);
        assert!(!cut);
        assert_eq!(text, "diff --git a/x b/x\n+one\n");
    }

    #[test]
    fn long_diff_cuts_on_a_line_boundary() {
        let diff = "aaaa\nbbbb\ncccc\ndddd\n";
        let (text, cut) = truncate_diff(diff, 12);
        assert!(cut);
        // Never ends mid-line: a half hunk header is worse than less context.
        assert!(text.ends_with('c') || text.ends_with('b'));
        assert!(!text.contains("dddd"));
        assert_eq!(text.matches('\n').count(), text.lines().count() - 1);
    }

    #[test]
    fn cut_without_newline_falls_back_to_hard_limit() {
        let diff = "a".repeat(100);
        let (text, cut) = truncate_diff(&diff, 10);
        assert!(cut);
        assert_eq!(text.len(), 10);
    }

    /// Diffs carry non-ASCII paths and content; a byte-index cut inside a
    /// multi-byte character would panic rather than truncate.
    #[test]
    fn cut_inside_a_multibyte_char_does_not_panic() {
        let diff = "+++ b/日本語のファイル.md\n".repeat(20);
        for max in 1..60 {
            let (text, cut) = truncate_diff(&diff, max);
            assert!(cut, "max={max} should truncate");
            assert!(diff.starts_with(&text), "max={max} must stay a prefix");
        }
    }
}
