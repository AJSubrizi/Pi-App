//! Check for newer App releases on GitHub (AJSubrizi/Pi-App only).
//!
//! Does **not** auto-install. Opens the release page or the best installer asset.

use std::time::Duration;

use serde::Serialize;
use serde_json::Value;

const OWNER_REPO: &str = "AJSubrizi/Pi-App";
const DEFAULT_RELEASES_URL: &str =
    "https://api.github.com/repos/AJSubrizi/Pi-App/releases/latest";
const DEFAULT_RELEASES_PAGE: &str = "https://github.com/AJSubrizi/Pi-App/releases";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(12);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateCheck {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub release_name: Option<String>,
    /// Always an AJSubrizi/Pi-App releases URL.
    pub html_url: String,
    /// Best-effort direct installer URL for this OS/arch.
    pub download_url: Option<String>,
    pub published_at: Option<String>,
    pub body: Option<String>,
    pub asset_names: Vec<String>,
}

pub fn parse_semver(raw: &str) -> Option<(u64, u64, u64)> {
    let s = raw.trim().trim_start_matches(['v', 'V']);
    if s.is_empty() {
        return None;
    }
    let core = s.split(['-', '+']).next().unwrap_or(s);
    let mut parts = core.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some((major, minor, patch))
}

pub fn is_remote_newer(current: &str, remote: &str) -> bool {
    match (parse_semver(current), parse_semver(remote)) {
        (Some(a), Some(b)) => b > a,
        _ => false,
    }
}

/// Only accept URLs on AJSubrizi/Pi-App; rewrite anything else.
pub fn sanitize_release_url(url: &str, tag: &str) -> String {
    let u = url.trim();
    let lower = u.to_ascii_lowercase();
    let on_pi = lower.contains("github.com/ajsubrizi/pi-app");
    let legacy = lower.contains("ronglecat") || lower.contains(concat!("grok", "-app"));
    if on_pi && !legacy {
        return u.to_string();
    }
    let tag = tag.trim().trim_start_matches(['v', 'V']);
    if tag.is_empty() {
        DEFAULT_RELEASES_PAGE.to_string()
    } else {
        format!("https://github.com/{OWNER_REPO}/releases/tag/v{tag}")
    }
}

fn asset_score(name: &str) -> i32 {
    let n = name.to_ascii_lowercase();
    // Prefer Pi_* installers; reject obvious non-Pi brand prefixes.
    if n.starts_with("grok") {
        return -1000;
    }
    let mut score = 0i32;
    if n.starts_with("pi_") || n.starts_with("pi-") {
        score += 20;
    }
    #[cfg(target_os = "macos")]
    {
        if n.ends_with(".dmg") {
            score += 50;
        }
        #[cfg(target_arch = "aarch64")]
        {
            if n.contains("aarch64") || n.contains("arm64") {
                score += 30;
            }
            if n.contains("x64") || n.contains("x86_64") || n.contains("amd64") {
                score -= 10;
            }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            if n.contains("x64") || n.contains("x86_64") || n.contains("amd64") {
                score += 30;
            }
            if n.contains("aarch64") || n.contains("arm64") {
                score -= 10;
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        if n.contains("setup") && n.ends_with(".exe") {
            score += 60;
        } else if n.ends_with(".exe") {
            score += 40;
        } else if n.contains("portable") && n.ends_with(".zip") {
            score += 20;
        }
    }
    #[cfg(target_os = "linux")]
    {
        if n.ends_with(".appimage") {
            score += 50;
        } else if n.ends_with(".deb") {
            score += 40;
        } else if n.ends_with(".rpm") {
            score += 30;
        }
    }
    score
}

pub fn pick_download_url(assets: &Value) -> Option<String> {
    let arr = assets.as_array()?;
    let mut best: Option<(i32, String)> = None;
    for a in arr {
        let name = a.get("name").and_then(|x| x.as_str()).unwrap_or("");
        let url = a
            .get("browser_download_url")
            .and_then(|x| x.as_str())
            .unwrap_or("");
        if name.is_empty() || url.is_empty() {
            continue;
        }
        let s = asset_score(name);
        if s < 10 {
            continue;
        }
        match &best {
            None => best = Some((s, url.to_string())),
            Some((bs, _)) if s > *bs => best = Some((s, url.to_string())),
            _ => {}
        }
    }
    best.map(|(_, u)| u)
}

pub fn parse_github_release(current_version: &str, v: &Value) -> Result<AppUpdateCheck, String> {
    let tag = v
        .get("tag_name")
        .and_then(|x| x.as_str())
        .ok_or_else(|| "release missing tag_name".to_string())?
        .trim();
    if tag.is_empty() {
        return Err("empty tag_name".into());
    }
    let raw_html = v.get("html_url").and_then(|x| x.as_str()).unwrap_or("");
    let html_url = sanitize_release_url(raw_html, tag);
    let release_name = v
        .get("name")
        .and_then(|x| x.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let published_at = v
        .get("published_at")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let body = v
        .get("body")
        .and_then(|x| x.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let assets_val = v.get("assets").cloned().unwrap_or(Value::Array(vec![]));
    let asset_names = assets_val
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|a| a.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                .filter(|n| !n.to_ascii_lowercase().starts_with("grok"))
                .collect()
        })
        .unwrap_or_default();
    let download_url = pick_download_url(&assets_val);

    let latest_version = tag.trim_start_matches(['v', 'V']).to_string();
    let update_available = is_remote_newer(current_version, tag);

    Ok(AppUpdateCheck {
        current_version: current_version.to_string(),
        latest_version,
        update_available,
        release_name,
        html_url,
        download_url,
        published_at,
        body,
        asset_names,
    })
}

fn releases_url() -> String {
    std::env::var("PI_APP_RELEASES_URL").unwrap_or_else(|_| DEFAULT_RELEASES_URL.into())
}

pub async fn check_app_update() -> Result<AppUpdateCheck, String> {
    let current = env!("CARGO_PKG_VERSION");
    let url = releases_url();
    if !(url.starts_with("https://")
        || url.starts_with("http://127.0.0.1")
        || url.starts_with("http://localhost"))
    {
        return Err("update check URL must be https (or localhost for tests)".into());
    }
    let lower = url.to_ascii_lowercase();
    if lower.contains("ronglecat") || lower.contains(concat!("grok", "-app")) {
        return Err("refusing non-Pi update endpoint".into());
    }

    let client = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
        .user_agent(format!(
            "Pi-App/{} (desktop; check-update; +https://github.com/{OWNER_REPO})",
            current
        ))
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .get(&url)
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .send()
        .await
        .map_err(|e| format!("update check network: {e}"))?;

    if !res.status().is_success() {
        return Err(format!(
            "GitHub releases returned HTTP {}",
            res.status().as_u16()
        ));
    }

    let v: Value = res
        .json()
        .await
        .map_err(|e| format!("update check parse: {e}"))?;
    parse_github_release(current, &v)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_semver_strips_v_and_prerelease() {
        assert_eq!(parse_semver("v0.1.5"), Some((0, 1, 5)));
        assert_eq!(parse_semver("1.2.3-beta.1"), Some((1, 2, 3)));
        assert!(parse_semver("").is_none());
    }

    #[test]
    fn is_remote_newer_orders() {
        assert!(!is_remote_newer("0.1.5", "v0.1.5"));
        assert!(is_remote_newer("0.2.0", "0.2.1"));
    }

    #[test]
    fn sanitize_rewrites_foreign_urls() {
        let bad = "https://github.com/other/repo/releases/tag/v0.2.0";
        let got = sanitize_release_url(bad, "v0.2.0");
        assert!(got.contains("AJSubrizi/Pi-App"));
        let good = "https://github.com/AJSubrizi/Pi-App/releases/tag/v0.2.1";
        assert_eq!(sanitize_release_url(good, "v0.2.1"), good);
    }

    #[test]
    fn parse_github_release_picks_pi_asset() {
        let sample = json!({
            "tag_name": "v0.2.1",
            "name": "Pi v0.2.1",
            "html_url": "https://github.com/AJSubrizi/Pi-App/releases/tag/v0.2.1",
            "body": "fix",
            "assets": [
                {
                    "name": "Grok_0.2.1_x64.dmg",
                    "browser_download_url": "https://example.com/Grok.dmg"
                },
                {
                    "name": "Pi_0.2.1_aarch64.dmg",
                    "browser_download_url": "https://github.com/AJSubrizi/Pi-App/releases/download/v0.2.1/Pi_0.2.1_aarch64.dmg"
                }
            ]
        });
        let up = parse_github_release("0.2.0", &sample).unwrap();
        assert!(up.update_available);
        assert!(up.html_url.contains("AJSubrizi/Pi-App"));
        assert!(up.asset_names.iter().all(|n| !n.to_ascii_lowercase().starts_with("grok")));
        if let Some(d) = &up.download_url {
            assert!(d.contains("Pi_"));
        }
    }
}
