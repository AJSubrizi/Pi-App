//! Honest local activity summary derived from the app-owned session journal.

use chrono::{Duration, Local, NaiveDate};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};

use crate::store;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageDay {
    pub date: String,
    pub activities: u64,
    pub estimated_tokens: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageTool {
    pub name: String,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageProfile {
    pub days: Vec<UsageDay>,
    pub total_estimated_tokens: u64,
    pub max_session_tokens: u64,
    pub longest_activity_secs: u64,
    pub current_streak_days: u64,
    pub longest_streak_days: u64,
    pub total_sessions: u64,
    pub total_turns: u64,
    pub total_tool_calls: u64,
    pub models_used: u64,
    pub most_used_effort: Option<String>,
    pub top_tools: Vec<UsageTool>,
}

fn estimated_tokens(text: &str) -> u64 {
    if text.is_empty() {
        return 0;
    }
    ((text.chars().count() as u64) + 3) / 4
}

fn tool_name(line: &str) -> Option<&str> {
    let mut parts = line.trim().split('|');
    if parts.next()? != "tool_step" {
        return None;
    }
    let _status = parts.next()?;
    parts.next().map(str::trim).filter(|name| !name.is_empty())
}

fn streaks(days: &BTreeSet<NaiveDate>) -> (u64, u64) {
    let mut longest = 0_u64;
    let mut run = 0_u64;
    let mut previous: Option<NaiveDate> = None;
    for day in days {
        run = if previous
            .map(|value| *day == value + Duration::days(1))
            .unwrap_or(false)
        {
            run + 1
        } else {
            1
        };
        longest = longest.max(run);
        previous = Some(*day);
    }

    let today = Local::now().date_naive();
    let anchor = if days.contains(&today) {
        today
    } else if days.contains(&(today - Duration::days(1))) {
        today - Duration::days(1)
    } else {
        return (0, longest);
    };
    let mut current = 0_u64;
    let mut cursor = anchor;
    while days.contains(&cursor) {
        current += 1;
        cursor -= Duration::days(1);
    }
    (current, longest)
}

#[tauri::command]
pub async fn usage_profile() -> Result<UsageProfile, String> {
    let sessions = store::load_sessions_index();
    let mut days: BTreeMap<NaiveDate, (u64, u64)> = BTreeMap::new();
    let mut active_days = BTreeSet::new();
    let mut models = BTreeSet::new();
    let mut efforts: HashMap<String, u64> = HashMap::new();
    let mut tools: HashMap<String, u64> = HashMap::new();
    let mut total_estimated_tokens = 0_u64;
    let mut max_session_tokens = 0_u64;
    let mut longest_activity_secs = 0_u64;
    let mut total_turns = 0_u64;
    let mut total_tool_calls = 0_u64;

    for session in &sessions {
        if let Some(model) = session.model_id.as_deref().filter(|value| !value.is_empty()) {
            models.insert(model.to_string());
        }
        if let Some(effort) = session.effort.as_deref().filter(|value| !value.is_empty()) {
            *efforts.entry(effort.to_string()).or_default() += 1;
        }

        let messages = store::load_messages(&session.id);
        let mut session_tokens = 0_u64;
        let mut first = None;
        let mut last = None;
        for message in messages {
            first.get_or_insert(message.created_at);
            last = Some(message.created_at);
            let tokens = estimated_tokens(&message.content)
                + message
                    .thought
                    .as_deref()
                    .map(estimated_tokens)
                    .unwrap_or(0);
            session_tokens += tokens;
            let day = message.created_at.with_timezone(&Local).date_naive();
            let entry = days.entry(day).or_default();
            entry.1 += tokens;
            if message.role == "user" {
                entry.0 += 1;
                total_turns += 1;
                active_days.insert(day);
            }
            for line in message.content.lines() {
                if let Some(name) = tool_name(line) {
                    *tools.entry(name.to_string()).or_default() += 1;
                    total_tool_calls += 1;
                }
            }
        }
        total_estimated_tokens += session_tokens;
        max_session_tokens = max_session_tokens.max(session_tokens);
        if let (Some(start), Some(end)) = (first, last) {
            longest_activity_secs =
                longest_activity_secs.max((end - start).num_seconds().max(0) as u64);
        }
    }

    let (current_streak_days, longest_streak_days) = streaks(&active_days);
    let mut top_tools: Vec<UsageTool> = tools
        .into_iter()
        .map(|(name, count)| UsageTool { name, count })
        .collect();
    top_tools.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.name.cmp(&b.name)));
    top_tools.truncate(5);
    let most_used_effort = efforts
        .into_iter()
        .max_by(|a, b| a.1.cmp(&b.1).then_with(|| b.0.cmp(&a.0)))
        .map(|(name, _)| name);

    Ok(UsageProfile {
        days: days
            .into_iter()
            .map(|(date, (activities, estimated_tokens))| UsageDay {
                date: date.format("%Y-%m-%d").to_string(),
                activities,
                estimated_tokens,
            })
            .collect(),
        total_estimated_tokens,
        max_session_tokens,
        longest_activity_secs,
        current_streak_days,
        longest_streak_days,
        total_sessions: sessions.len() as u64,
        total_turns,
        total_tool_calls,
        models_used: models.len() as u64,
        most_used_effort,
        top_tools,
    })
}

#[cfg(test)]
mod tests {
    use super::{estimated_tokens, streaks, tool_name};
    use chrono::NaiveDate;
    use std::collections::BTreeSet;

    #[test]
    fn token_estimate_and_tool_parser_are_stable() {
        assert_eq!(estimated_tokens("12345"), 2);
        assert_eq!(
            tool_name("tool_step|completed|run_terminal_command|pnpm test"),
            Some("run_terminal_command")
        );
        assert_eq!(tool_name("ordinary text"), None);
    }

    #[test]
    fn longest_streak_counts_consecutive_days() {
        let days = ["2026-07-20", "2026-07-21", "2026-07-23"]
            .into_iter()
            .map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap())
            .collect::<BTreeSet<_>>();
        let (_, longest) = streaks(&days);
        assert_eq!(longest, 2);
    }
}
