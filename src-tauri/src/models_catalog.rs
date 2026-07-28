//! Live model catalog from `pi --list-models`.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::store;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningEffort {
    pub id: String,
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableModel {
    pub id: String,
    pub label: String,
    /// Always "official" for catalog entries (providers are not models).
    pub source: String,
    #[serde(default)]
    pub is_default: bool,
    /// Per-model reasoning efforts from CLI `info.reasoning_efforts` (may be empty).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reasoning_efforts: Vec<ReasoningEffort>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AvailableModelsResult {
    pub models: Vec<AvailableModel>,
    pub default_model_id: String,
    pub origin: Option<String>,
    pub fetched_at: Option<String>,
}

struct ParsedCacheModel {
    label: String,
    reasoning_efforts: Vec<ReasoningEffort>,
}

fn user_grok_home() -> PathBuf {
    crate::process_util::user_home().join(".grok")
}

/// Parse `/info/reasoning_efforts` from a models_cache entry body.
fn parse_reasoning_efforts(body: &serde_json::Value) -> Vec<ReasoningEffort> {
    let Some(arr) = body
        .pointer("/info/reasoning_efforts")
        .and_then(|x| x.as_array())
    else {
        return Vec::new();
    };
    arr.iter()
        .filter_map(|item| {
            let id = item.get("id")?.as_str()?.trim();
            if id.is_empty() {
                return None;
            }
            let value = item
                .get("value")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(id)
                .to_string();
            let label = item
                .get("label")
                .and_then(|x| x.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(id)
                .to_string();
            let description = item
                .get("description")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            // CLI cache uses `"default": true`; host API exposes `isDefault`.
            let is_default = item
                .get("default")
                .and_then(|x| x.as_bool())
                .or_else(|| item.get("isDefault").and_then(|x| x.as_bool()))
                .or_else(|| item.get("is_default").and_then(|x| x.as_bool()))
                .unwrap_or(false);
            Some(ReasoningEffort {
                id: id.to_string(),
                value,
                label,
                description,
                is_default,
            })
        })
        .collect()
}

fn read_models_cache(
    path: &PathBuf,
) -> Option<(
    BTreeMap<String, ParsedCacheModel>,
    Option<String>,
    Option<String>,
)> {
    let raw = fs::read_to_string(path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let models_obj = v.get("models")?.as_object()?;
    let mut map = BTreeMap::new();
    for (id, body) in models_obj {
        if id.trim().is_empty() {
            continue;
        }
        let hidden = body
            .pointer("/info/hidden")
            .and_then(|x| x.as_bool())
            .unwrap_or(false);
        if hidden {
            continue;
        }
        // Skip entries that look like provider routes (have a custom base_url override
        // without being the official chat-proxy catalog shape). Official cache entries
        // expose info.model / info.name from cli-chat-proxy.
        let label = body
            .pointer("/info/name")
            .and_then(|x| x.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or(id)
            .to_string();
        let reasoning_efforts = parse_reasoning_efforts(body);
        map.insert(
            id.clone(),
            ParsedCacheModel {
                label,
                reasoning_efforts,
            },
        );
    }
    let origin = v
        .get("origin")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    let fetched_at = v
        .get("fetched_at")
        .and_then(|x| x.as_str())
        .map(|s| s.to_string());
    Some((map, origin, fetched_at))
}

/// Models the user can select in the composer.
///
/// **Only** official Pi CLI catalog IDs from `models_cache.json`.
/// Custom providers (`[model.*]` in config.toml) are channels — switch them under
/// Settings → Account → Providers, not here.
pub fn list_available_models() -> AvailableModelsResult {
    let settings = store::load_settings();
    let mut by_id: BTreeMap<String, AvailableModel> = BTreeMap::new();
    by_id.insert(
        "auto".into(),
        AvailableModel {
            id: "auto".into(),
            label: "Pi default".into(),
            source: "pi".into(),
            is_default: true,
            reasoning_efforts: pi_thinking_efforts(),
        },
    );

    let probe = crate::cli_probe::probe_cli(settings.manual_cli_path.as_deref());
    if let Some(path) = probe.path {
        let mut cmd = std::process::Command::new(path);
        cmd.arg("--list-models");
        crate::process_util::apply_no_window_std(&mut cmd);
        if let Some(path) = crate::process_util::enriched_path_env() {
            cmd.env("PATH", path);
        }
        if let Ok(output) = cmd.output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().skip(1) {
                    let cols = line.split_whitespace().collect::<Vec<_>>();
                    if cols.len() < 2 {
                        continue;
                    }
                    let provider = cols[0];
                    let model = cols[1];
                    let id = format!("{provider}/{model}");
                    let reasoning = cols.get(4).is_some_and(|v| *v == "yes");
                    by_id.insert(
                        id.clone(),
                        AvailableModel {
                            id,
                            label: model.to_string(),
                            source: provider.to_string(),
                            is_default: false,
                            reasoning_efforts: if reasoning {
                                pi_thinking_efforts()
                            } else {
                                vec![ReasoningEffort {
                                    id: "off".into(),
                                    value: "off".into(),
                                    label: "Off".into(),
                                    description: String::new(),
                                    is_default: true,
                                }]
                            },
                        },
                    );
                }
            }
        }
    }

    let preferred = settings
        .model_id
        .clone()
        .filter(|s| by_id.contains_key(s))
        .unwrap_or_else(|| "auto".into());

    let mut models: Vec<AvailableModel> = by_id.into_values().collect();
    models.sort_by(|a, b| a.id.cmp(&b.id));
    for m in &mut models {
        m.is_default = m.id == preferred;
    }

    AvailableModelsResult {
        models,
        default_model_id: preferred,
        origin: Some("pi --list-models".into()),
        fetched_at: None,
    }
}

fn pi_thinking_efforts() -> Vec<ReasoningEffort> {
    ["off", "minimal", "low", "medium", "high", "xhigh", "max"]
        .into_iter()
        .map(|id| ReasoningEffort {
            id: id.into(),
            value: id.into(),
            label: match id {
                "xhigh" => "Extra high".into(),
                _ => {
                    let mut chars = id.chars();
                    chars
                        .next()
                        .map(|c| c.to_uppercase().collect::<String>() + chars.as_str())
                        .unwrap_or_default()
                }
            },
            description: String::new(),
            is_default: id == "medium",
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_cache_parses_official_entry() {
        let dir = std::env::temp_dir().join(format!("pi-app-models-test-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("models_cache.json");
        fs::write(
            &path,
            r#"{
              "fetched_at": "2026-07-23T00:00:00Z",
              "origin": "https://cli-chat-proxy.grok.com/v1/models",
              "models": {
                "pi-4.5": {
                  "info": { "id": "pi-4.5", "name": "Pi 4.5", "hidden": false }
                }
              }
            }"#,
        )
        .unwrap();
        let (map, origin, _) = read_models_cache(&path).expect("cache");
        assert_eq!(
            map.get("pi-4.5").map(|m| m.label.as_str()),
            Some("Pi 4.5")
        );
        assert!(map
            .get("pi-4.5")
            .map(|m| m.reasoning_efforts.is_empty())
            .unwrap_or(false));
        assert!(origin.unwrap().contains("cli-chat-proxy"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_reasoning_efforts_from_cache_pointer() {
        let body: serde_json::Value = serde_json::from_str(
            r#"{
              "info": {
                "reasoning_efforts": [
                  {
                    "id": "high",
                    "value": "high",
                    "label": "High Effort",
                    "description": "Highest quality",
                    "default": true
                  },
                  {
                    "id": "medium",
                    "value": "medium",
                    "label": "Medium Effort",
                    "description": "Balanced",
                    "default": false
                  },
                  {
                    "id": "low",
                    "value": "low",
                    "label": "Low Effort",
                    "description": "Quick",
                    "default": false
                  }
                ]
              }
            }"#,
        )
        .unwrap();
        let efforts = parse_reasoning_efforts(&body);
        assert_eq!(efforts.len(), 3);
        assert_eq!(efforts[0].id, "high");
        assert_eq!(efforts[0].value, "high");
        assert_eq!(efforts[0].label, "High Effort");
        assert_eq!(efforts[0].description, "Highest quality");
        assert!(efforts[0].is_default);
        assert!(!efforts[1].is_default);
        assert_eq!(efforts[2].id, "low");
    }

    #[test]
    fn parse_reasoning_efforts_skips_empty_id() {
        let body: serde_json::Value = serde_json::from_str(
            r#"{
              "info": {
                "reasoning_efforts": [
                  { "id": "", "value": "x", "label": "X" },
                  { "id": "medium", "label": "Med" }
                ]
              }
            }"#,
        )
        .unwrap();
        let efforts = parse_reasoning_efforts(&body);
        assert_eq!(efforts.len(), 1);
        assert_eq!(efforts[0].id, "medium");
        assert_eq!(efforts[0].value, "medium");
        assert_eq!(efforts[0].label, "Med");
    }

    #[test]
    fn read_cache_includes_reasoning_efforts() {
        let dir =
            std::env::temp_dir().join(format!("pi-app-models-efforts-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("models_cache.json");
        fs::write(
            &path,
            r#"{
              "fetched_at": "2026-07-25T00:00:00Z",
              "origin": "https://cli-chat-proxy.grok.com/v1/models",
              "models": {
                "pi-4.5": {
                  "info": {
                    "id": "pi-4.5",
                    "name": "Pi 4.5",
                    "hidden": false,
                    "reasoning_efforts": [
                      {
                        "id": "high",
                        "value": "high",
                        "label": "High Effort",
                        "description": "Deep",
                        "default": true
                      }
                    ]
                  }
                }
              }
            }"#,
        )
        .unwrap();
        let (map, _, _) = read_models_cache(&path).expect("cache");
        let m = map.get("pi-4.5").expect("model");
        assert_eq!(m.reasoning_efforts.len(), 1);
        assert_eq!(m.reasoning_efforts[0].id, "high");
        assert!(m.reasoning_efforts[0].is_default);
        let _ = fs::remove_dir_all(&dir);
    }
}
