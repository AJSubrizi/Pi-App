//! Auto-title sessions from the first user message.
//! Instant, local heuristic naming. Pi packages may provide richer workflows.

use tauri::AppHandle;

use crate::store::{self, SessionMeta};

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
}
