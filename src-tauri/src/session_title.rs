//! Auto-title sessions from the first user message.
//! Instant, local heuristic naming. Pi packages may provide richer workflows.

use tauri::AppHandle;

use crate::store::{self, SessionMeta};
use crate::tray_i18n::{self, Locale};

const PLACEHOLDERS: &[&str] = &[
    "New chat",
    "新会话",
    "新对话",
    "新對話",
    "Untitled",
    "未命名",
    "New conversation",
    "新建会话",
];

pub fn is_placeholder_title(title: &str) -> bool {
    let t = title.trim();
    t.is_empty() || PLACEHOLDERS.iter().any(|p| p.eq_ignore_ascii_case(t))
}

/// Offline title: first non-empty line, collapsed whitespace, max ~28 display chars.
pub fn heuristic_title(message: &str) -> String {
    let line = message
        .lines()
        .map(|l| l.trim())
        .find(|l| !l.is_empty())
        .unwrap_or("Chat");
    let collapsed: String = line.split_whitespace().collect::<Vec<_>>().join(" ");
    truncate_chars(&collapsed, 28)
}

fn truncate_chars(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn clean_llm_title(raw: &str) -> Option<String> {
    let mut t = raw.trim().to_string();
    for _ in 0..3 {
        if (t.starts_with('"') && t.ends_with('"'))
            || (t.starts_with('「') && t.ends_with('」'))
            || (t.starts_with('“') && t.ends_with('”'))
            || (t.starts_with('\'') && t.ends_with('\''))
        {
            t = t[1..t.len() - 1].trim().to_string();
        }
    }
    if let Some(rest) = t
        .strip_prefix("标题：")
        .or_else(|| t.strip_prefix("标题:"))
        .or_else(|| t.strip_prefix("Title:"))
        .or_else(|| t.strip_prefix("Title："))
    {
        t = rest.trim().to_string();
    }
    if let Some(line) = t.lines().next() {
        t = line.trim().to_string();
    }
    if t.is_empty() || t.len() > 120 || is_placeholder_title(&t) {
        return None;
    }
    Some(truncate_chars(&t, 32))
}

/// Prompt for the headless title LLM, matching the app UI locale.
fn title_prompt(snippet: &str, locale: Locale) -> String {
    match locale {
        Locale::En => format!(
            "Write a short session title for the user message below.\nRequirements: at most 8 English words (or match the message language if it is not English); output the title only; no quotes, prefixes, or explanation.\n\nUser message:\n{snippet}"
        ),
        Locale::Zh => format!(
            "为下面这条用户消息起一个简短会话标题。要求：最多16个汉字或8个英文单词；只输出标题；不要引号、标点前缀、解释。\n\n用户消息：\n{snippet}"
        ),
        Locale::ZhTw => format!(
            "為下面這則使用者訊息起一個簡短對話標題。要求：最多16個漢字或8個英文單詞；只輸出標題；不要引號、標點前綴、解釋。\n\n使用者訊息：\n{snippet}"
        ),
    }
}

/// Immediate heuristic rename when the title is still a placeholder.
pub fn auto_title_session_fast(id: &str, first_message: &str) -> Result<SessionMeta, String> {
    let list = store::load_sessions_index();
    let current = list
        .iter()
        .find(|s| s.id == id)
        .cloned()
        .ok_or_else(|| "session not found".to_string())?;

    if !is_placeholder_title(&current.title) {
        return Ok(current);
    }

    let heuristic = heuristic_title(first_message);
    store::rename_session(id, &heuristic)
}

/// Pi RPC intentionally has no hidden second model turn for title generation.
/// Keep this compatibility hook as a no-op so extensions can own richer naming.
pub fn refine_title_in_background(
    _app: AppHandle,
    _mgr: std::sync::Arc<crate::session_manager::SessionManager>,
    _id: String,
    _first_message: String,
) {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholders() {
        assert!(is_placeholder_title("新会话"));
        assert!(is_placeholder_title("新对话"));
        assert!(is_placeholder_title("新對話"));
        assert!(is_placeholder_title("New chat"));
        assert!(!is_placeholder_title("修权限条 bug"));
        assert!(!is_placeholder_title("馬斯克最近有發什麼貼文"));
    }

    #[test]
    fn heuristic_uses_first_line() {
        let t = heuristic_title("  帮我改一下登录页样式\n第二行");
        assert!(t.contains("登录") || t.contains("帮我"));
        assert!(t.chars().count() <= 28);
    }

    #[test]
    fn clean_strips_quotes() {
        assert_eq!(
            clean_llm_title("  \"修复登录样式\" \n"),
            Some("修复登录样式".into())
        );
    }

    #[test]
    fn clean_strips_english_title_prefix() {
        assert_eq!(
            clean_llm_title("Title: List open PRs\n"),
            Some("List open PRs".into())
        );
    }

    #[test]
    fn title_prompt_follows_locale() {
        let en = title_prompt("list open prs", Locale::En);
        assert!(en.contains("User message:"));
        assert!(en.contains("list open prs"));
        assert!(!en.contains("用户消息"));

        let zh = title_prompt("list open prs", Locale::Zh);
        assert!(zh.contains("用户消息："));
        assert!(zh.contains("为下面这条用户消息"));
        assert!(zh.contains("list open prs"));
        assert!(!zh.contains("User message:"));

        let zhtw = title_prompt("list open prs", Locale::ZhTw);
        assert!(zhtw.contains("為下面這則使用者訊息"));
        assert!(zhtw.contains("使用者訊息："));
        assert!(zhtw.contains("list open prs"));
        assert!(!zhtw.contains("为下面这条用户消息"));
        assert!(!zhtw.contains("User message:"));
    }
}
