//! Live model catalog from `pi --list-models`.

use std::collections::BTreeMap;

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
mod tests {}
