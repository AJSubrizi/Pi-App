//! Durable, append-only token usage ledger.
//!
//! A session's live total is useful for the composer, but it is not a history.
//! Keep one measured row per completed Pi turn so usage survives process recycle
//! and restart without making the session journal carry billing data.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use crate::paths;
use crate::token_usage::TokenUsage;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UsageLedgerRow {
    pub id: String,
    pub session_id: String,
    pub project_id: Option<String>,
    pub model_id: Option<String>,
    pub recorded_at: DateTime<Utc>,
    pub usage: TokenUsage,
    /// Wall-clock time from prompt dispatch to provider completion.
    #[serde(default)]
    pub latency_ms: Option<u64>,
    /// `success` for measured turns, `failure` for a provider/process error.
    #[serde(default = "default_outcome")]
    pub outcome: String,
}

fn default_outcome() -> String {
    "success".into()
}

pub fn append_with_metadata(
    session_id: &str,
    project_id: Option<&str>,
    model_id: Option<&str>,
    usage: TokenUsage,
    latency_ms: Option<u64>,
    outcome: &str,
) -> Result<UsageLedgerRow, String> {
    let row = UsageLedgerRow {
        id: uuid::Uuid::new_v4().to_string(),
        session_id: session_id.to_string(),
        project_id: project_id.map(str::to_string),
        model_id: model_id.map(str::to_string),
        recorded_at: Utc::now(),
        usage,
        latency_ms,
        outcome: if outcome == "failure" {
            "failure".into()
        } else {
            "success".into()
        },
    };
    append_to(&paths::usage_ledger_file(), &row)?;
    Ok(row)
}

pub fn append_to(path: &Path, row: &UsageLedgerRow) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    serde_json::to_writer(&mut file, row).map_err(|e| e.to_string())?;
    file.write_all(b"\n").map_err(|e| e.to_string())
}

pub fn load() -> Vec<UsageLedgerRow> {
    load_from(&paths::usage_ledger_file())
}

pub fn load_from(path: &Path) -> Vec<UsageLedgerRow> {
    let Ok(file) = fs::File::open(path) else {
        return Vec::new();
    };
    // `map_while`, not `filter_map`: a read error means the rest of the file is
    // unreadable too, so skipping past it would spin rather than terminate.
    BufReader::new(file)
        .lines()
        .map_while(Result::ok)
        .filter_map(|line| serde_json::from_str(&line).ok())
        .collect()
}

/// Fold measured rows for a query scope while preserving the provider's
/// `cost_total: None` semantics.
pub fn sum_usage(rows: &[UsageLedgerRow]) -> TokenUsage {
    let mut total = TokenUsage::default();
    for row in rows {
        total.add(&row.usage);
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "pi-app-usage-{name}-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn round_trips_rows_across_a_restart() {
        let path = temp_path("roundtrip");
        let row = UsageLedgerRow {
            id: "turn-1".into(),
            session_id: "session-1".into(),
            project_id: Some("project-1".into()),
            model_id: Some("gpt-5.6".into()),
            recorded_at: Utc::now(),
            usage: TokenUsage {
                input: 12,
                output: 8,
                cache_read: 30,
                total_tokens: 50,
                cost_total: None,
                ..Default::default()
            },
            latency_ms: Some(123),
            outcome: "success".into(),
        };
        append_to(&path, &row).unwrap();
        assert_eq!(load_from(&path), vec![row]);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn ignores_a_torn_last_line() {
        let path = temp_path("torn");
        fs::write(&path, "{\"not-complete\"").unwrap();
        assert!(load_from(&path).is_empty());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn grouping_keeps_known_cost_when_another_row_has_no_cost() {
        let rows = vec![
            UsageLedgerRow {
                id: "a".into(),
                session_id: "s".into(),
                project_id: None,
                model_id: Some("m".into()),
                recorded_at: Utc::now(),
                usage: TokenUsage {
                    total_tokens: 4,
                    cost_total: Some(0.25),
                    ..Default::default()
                },
                latency_ms: None,
                outcome: "success".into(),
            },
            UsageLedgerRow {
                id: "b".into(),
                session_id: "s".into(),
                project_id: None,
                model_id: Some("m".into()),
                recorded_at: Utc::now(),
                usage: TokenUsage {
                    total_tokens: 6,
                    cost_total: None,
                    ..Default::default()
                },
                latency_ms: None,
                outcome: "success".into(),
            },
        ];
        let total = sum_usage(&rows);
        assert_eq!(total.total_tokens, 10);
        assert_eq!(total.cost_total, Some(0.25));
    }
}
