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
    /// Catalog source, usually a Pi provider id or "custom" for app-configured providers.
    pub source: String,
    /// Context window in tokens. Pi CLI does not expose this consistently, so
    /// the host fills a small conservative table for known families.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
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
/// Official Pi CLI ids plus configured custom provider ids. Custom entries are
/// deliberately marked as `custom` so a per-session choice cannot be mistaken
/// for an official catalog model.
pub fn list_available_models() -> AvailableModelsResult {
    let settings = store::load_settings();
    let mut by_id: BTreeMap<String, AvailableModel> = BTreeMap::new();
    by_id.insert(
        "auto".into(),
        AvailableModel {
            id: "auto".into(),
            label: "Pi default".into(),
            source: "pi".into(),
            context_window: Some(context_window_for("auto")),
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
                            id: id.clone(),
                            label: model.to_string(),
                            source: provider.to_string(),
                            context_window: Some(context_window_for(&id)),
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

    if let Ok(custom) = crate::providers::list_custom_providers() {
        for provider in custom.providers {
            if models.iter().any(|model| model.id == provider.id) {
                continue;
            }
            models.push(AvailableModel {
                id: provider.id.clone(),
                label: format!("{} · {}", provider.name, provider.model),
                source: "custom".into(),
                context_window: Some(context_window_for(&provider.id)),
                is_default: provider.is_default,
                reasoning_efforts: pi_thinking_efforts(),
            });
        }
        models.sort_by(|a, b| a.source.cmp(&b.source).then_with(|| a.id.cmp(&b.id)));
    }

    AvailableModelsResult {
        models,
        default_model_id: preferred,
        origin: Some("pi --list-models".into()),
        fetched_at: None,
    }
}

fn context_window_for(id: &str) -> u64 {
    let lower = id.to_ascii_lowercase();
    if lower.contains("claude") {
        return 200_000;
    }
    if lower.contains("gpt-5") {
        return 400_000;
    }
    if lower.contains("grok") {
        return 131_072;
    }
    if lower.contains("gemini") {
        return 1_000_000;
    }
    128_000
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
