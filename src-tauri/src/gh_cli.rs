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
pub async fn gh_available(project_path: String) -> Result<GhAvailability, String> {
    let version = match run_gh(&project_path, &["--version"]) {
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
    let (auth_ok, _, auth_err) = run_gh(&project_path, &["auth", "status"])?;
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
    project_path: String,
    limit: Option<u32>,
) -> Result<Vec<GhPullRequest>, String> {
    let limit = limit.unwrap_or(30).clamp(1, 100).to_string();
    let (ok, stdout, stderr) = run_gh(
        &project_path,
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

/// Metadata + unified diff for one pull request, ready to become prompt text.
#[tauri::command]
pub async fn gh_pr_diff(project_path: String, number: u64) -> Result<GhPrDiff, String> {
    let num = number.to_string();
    let (ok, meta_raw, stderr) = run_gh(
        &project_path,
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

    let (diff_ok, diff_raw, diff_err) = run_gh(&project_path, &["pr", "diff", &num])?;
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
