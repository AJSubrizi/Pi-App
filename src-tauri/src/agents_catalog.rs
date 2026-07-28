//! Discover selectable Pi CLI agent definition names for Settings.
//!
//! Sources mirror CLI `--agent <NAME>` resolution:
//! - Built-ins: explore, plan, general-purpose
//! - User: `~/.grok/agents/*.md`
//! - Project: `<cwd>/.grok/agents/*.md`
//! - Bundled reference: `~/.grok/bundled/agents/*.md` (same names as built-ins)

use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

/// Well-known built-in agent names (always listed even if files are missing).
pub const BUILTIN_AGENT_NAMES: &[&str] = &["explore", "general-purpose", "plan"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentCatalogSource {
    Builtin,
    User,
    Project,
    Bundled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCatalogEntry {
    pub name: String,
    pub source: AgentCatalogSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentsCatalogResult {
    pub agents: Vec<AgentCatalogEntry>,
    pub user_dir: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_dir: Option<String>,
    pub bundled_dir: String,
}

/// Pure: normalize settings value → spawn name, or `None` for CLI default.
pub fn normalize_preferred_agent(raw: &str) -> Option<String> {
    let name = raw.trim();
    if name.is_empty() {
        return None;
    }
    let lower = name.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "default" | "none" | "cli-default" | "pi-build"
    ) {
        return None;
    }
    if name.chars().any(|c| c == '\0' || c == '\n' || c == '\r') {
        return None;
    }
    Some(name.to_string())
}

/// Pure: top-level CLI args `["--agent", name]` when set.
pub fn agent_spawn_cli_args(raw: &str) -> Option<Vec<String>> {
    let name = normalize_preferred_agent(raw)?;
    Some(vec!["--agent".into(), name])
}

/// File stem for agent def (`explore.md` → `explore`).
pub fn agent_name_from_file_name(file_name: &str) -> Option<String> {
    let base = Path::new(file_name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(file_name)
        .trim();
    if base.is_empty() || base.starts_with('.') {
        return None;
    }
    let lower = base.to_ascii_lowercase();
    let stem = if let Some(s) = lower.strip_suffix(".markdown") {
        &base[..s.len()]
    } else if let Some(s) = lower.strip_suffix(".md") {
        &base[..s.len()]
    } else {
        return None;
    };
    let stem = stem.trim();
    if stem.is_empty() || stem.eq_ignore_ascii_case("readme") {
        return None;
    }
    Some(stem.to_string())
}

fn is_agent_md(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .and_then(agent_name_from_file_name)
        .is_some()
}

fn scan_agent_dir(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut out = Vec::new();
    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return out,
    };
    for ent in rd.flatten() {
        let path = ent.path();
        if !path.is_file() {
            continue;
        }
        if !is_agent_md(&path) {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(agent_name_from_file_name);
        if let Some(name) = name {
            out.push((name, path));
        }
    }
    out.sort_by(|a, b| a.0.to_ascii_lowercase().cmp(&b.0.to_ascii_lowercase()));
    out
}

/// Pure merge: project > user > bundled file > builtin name-only.
pub fn merge_agent_catalog(
    builtins: &[&str],
    user: &[(String, PathBuf)],
    project: &[(String, PathBuf)],
    bundled: &[(String, PathBuf)],
) -> Vec<AgentCatalogEntry> {
    use std::collections::BTreeMap;
    // BTreeMap keyed by lowercase name for stable sort of keys; we re-sort at end by display name.
    let mut map: BTreeMap<String, AgentCatalogEntry> = BTreeMap::new();

    for name in builtins {
        let n = name.trim();
        if n.is_empty() {
            continue;
        }
        map.insert(
            n.to_ascii_lowercase(),
            AgentCatalogEntry {
                name: n.to_string(),
                source: AgentCatalogSource::Builtin,
                path: None,
            },
        );
    }

    for (name, path) in bundled {
        map.insert(
            name.to_ascii_lowercase(),
            AgentCatalogEntry {
                name: name.clone(),
                source: AgentCatalogSource::Bundled,
                path: Some(path.display().to_string()),
            },
        );
    }

    for (name, path) in user {
        map.insert(
            name.to_ascii_lowercase(),
            AgentCatalogEntry {
                name: name.clone(),
                source: AgentCatalogSource::User,
                path: Some(path.display().to_string()),
            },
        );
    }

    for (name, path) in project {
        map.insert(
            name.to_ascii_lowercase(),
            AgentCatalogEntry {
                name: name.clone(),
                source: AgentCatalogSource::Project,
                path: Some(path.display().to_string()),
            },
        );
    }

    let mut agents: Vec<_> = map.into_values().collect();
    agents.sort_by(|a, b| {
        a.name
            .to_ascii_lowercase()
            .cmp(&b.name.to_ascii_lowercase())
    });
    agents
}

fn user_grok_home() -> PathBuf {
    crate::process_util::user_home().join(".grok")
}

/// Live catalog for Settings agent picker.
pub fn list_agents_catalog(project_path: Option<&str>) -> AgentsCatalogResult {
    let pi = user_grok_home();
    let user_dir = pi.join("agents");
    let bundled_dir = pi.join("bundled").join("agents");
    let project_dir = project_path
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|p| PathBuf::from(p).join(".grok").join("agents"));

    let user = scan_agent_dir(&user_dir);
    let bundled = scan_agent_dir(&bundled_dir);
    let project = project_dir
        .as_ref()
        .map(|d| scan_agent_dir(d))
        .unwrap_or_default();

    let agents = merge_agent_catalog(BUILTIN_AGENT_NAMES, &user, &project, &bundled);

    AgentsCatalogResult {
        agents,
        user_dir: user_dir.display().to_string(),
        project_dir: project_dir.map(|p| p.display().to_string()),
        bundled_dir: bundled_dir.display().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn normalize_empty_and_sentinels() {
        assert!(normalize_preferred_agent("").is_none());
        assert!(normalize_preferred_agent("  ").is_none());
        assert!(normalize_preferred_agent("default").is_none());
        assert!(normalize_preferred_agent("NONE").is_none());
        assert!(normalize_preferred_agent("pi-build").is_none());
        assert!(normalize_preferred_agent("cli-default").is_none());
        assert!(normalize_preferred_agent("ex\nplore").is_none());
    }

    #[test]
    fn normalize_keeps_names() {
        assert_eq!(
            normalize_preferred_agent("  explore  ").as_deref(),
            Some("explore")
        );
        assert_eq!(
            normalize_preferred_agent("general-purpose").as_deref(),
            Some("general-purpose")
        );
        assert_eq!(
            normalize_preferred_agent("/tmp/a.md").as_deref(),
            Some("/tmp/a.md")
        );
    }

    #[test]
    fn spawn_args_top_level() {
        assert!(agent_spawn_cli_args("").is_none());
        assert!(agent_spawn_cli_args("default").is_none());
        assert_eq!(
            agent_spawn_cli_args("explore"),
            Some(vec!["--agent".into(), "explore".into()])
        );
        assert_eq!(
            agent_spawn_cli_args("  plan  "),
            Some(vec!["--agent".into(), "plan".into()])
        );
    }

    #[test]
    fn file_name_stems() {
        assert_eq!(
            agent_name_from_file_name("explore.md").as_deref(),
            Some("explore")
        );
        assert_eq!(
            agent_name_from_file_name("my.markdown").as_deref(),
            Some("my")
        );
        assert!(agent_name_from_file_name("x.txt").is_none());
        assert!(agent_name_from_file_name(".hidden.md").is_none());
        assert!(agent_name_from_file_name("README.md").is_none());
        assert_eq!(
            agent_name_from_file_name("/a/b/plan.md").as_deref(),
            Some("plan")
        );
    }

    #[test]
    fn merge_priority_project_user_bundled_builtin() {
        let user = vec![(
            "explore".into(),
            PathBuf::from("/u/.grok/agents/explore.md"),
        )];
        let project = vec![(
            "explore".into(),
            PathBuf::from("/p/.grok/agents/explore.md"),
        )];
        let bundled = vec![(
            "plan".into(),
            PathBuf::from("/u/.grok/bundled/agents/plan.md"),
        )];
        let custom = vec![("custom".into(), PathBuf::from("/u/.grok/agents/custom.md"))];
        let user_all = [user, custom].concat();
        let agents = merge_agent_catalog(BUILTIN_AGENT_NAMES, &user_all, &project, &bundled);
        let by: std::collections::HashMap<_, _> =
            agents.into_iter().map(|e| (e.name.clone(), e)).collect();
        assert_eq!(by["explore"].source, AgentCatalogSource::Project);
        assert_eq!(
            by["explore"].path.as_deref(),
            Some("/p/.grok/agents/explore.md")
        );
        assert_eq!(by["custom"].source, AgentCatalogSource::User);
        assert_eq!(by["plan"].source, AgentCatalogSource::Bundled);
        assert_eq!(by["general-purpose"].source, AgentCatalogSource::Builtin);
        assert!(by["general-purpose"].path.is_none());
    }
}
