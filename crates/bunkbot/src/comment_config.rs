use std::collections::HashMap;
use std::sync::RwLock;

/// Parses a raw text input (pipe-separated or newline-separated) into a
/// deduplicated, trimmed list of non-empty comment strings.
pub fn parse_comment_input(text: &str) -> Vec<String> {
    let separator = if text.contains('|') { '|' } else { '\n' };
    text.split(separator)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Runtime response-pool override service.
///
/// Allows admins to replace or extend a bot's response pool without editing
/// YAML. The engine checks this service before falling back to the YAML pool.
pub trait CommentConfigService: Send + Sync + std::fmt::Debug {
    /// Replace the response pool for `bot_name` with the parsed items from
    /// `text` (pipe- or newline-separated). Returns the new pool size.
    fn set_comments(&self, bot_name: &str, entries: Vec<String>) -> usize;

    /// Append parsed items from `text` to the existing override pool.
    /// Returns the new total pool size.
    fn append_comments(&self, bot_name: &str, entries: Vec<String>) -> usize;

    /// Returns the current override pool for `bot_name`, or `None` if no
    /// override has been set (or after `clear_comments` is called).
    fn get_comments(&self, bot_name: &str) -> Option<Vec<String>>;

    /// Remove the override for `bot_name`, restoring YAML-defined responses.
    fn clear_comments(&self, bot_name: &str);

    /// Returns all bot names that currently have an active override pool.
    fn list_all(&self) -> Vec<(String, usize)>;
}

/// In-memory implementation of [`CommentConfigService`].
///
/// Uses `std::sync::RwLock` — all operations are tiny synchronous map
/// lookups/writes that never span `.await` boundaries.
#[derive(Debug, Default)]
pub struct InMemoryCommentConfigService {
    overrides: RwLock<HashMap<String, Vec<String>>>,
}

impl InMemoryCommentConfigService {
    pub fn new() -> Self {
        Self::default()
    }
}

impl CommentConfigService for InMemoryCommentConfigService {
    fn set_comments(&self, bot_name: &str, entries: Vec<String>) -> usize {
        let count = entries.len();
        let mut map = self.overrides.write().unwrap_or_else(|e| e.into_inner());
        map.insert(bot_name.to_string(), entries);
        count
    }

    fn append_comments(&self, bot_name: &str, entries: Vec<String>) -> usize {
        let mut map = self.overrides.write().unwrap_or_else(|e| e.into_inner());
        let pool = map.entry(bot_name.to_string()).or_default();
        pool.extend(entries);
        pool.len()
    }

    fn get_comments(&self, bot_name: &str) -> Option<Vec<String>> {
        let map = self.overrides.read().unwrap_or_else(|e| e.into_inner());
        map.get(bot_name).cloned()
    }

    fn clear_comments(&self, bot_name: &str) {
        let mut map = self.overrides.write().unwrap_or_else(|e| e.into_inner());
        map.remove(bot_name);
    }

    fn list_all(&self) -> Vec<(String, usize)> {
        let map = self.overrides.read().unwrap_or_else(|e| e.into_inner());
        let mut entries: Vec<(String, usize)> =
            map.iter().map(|(k, v)| (k.clone(), v.len())).collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn svc() -> InMemoryCommentConfigService {
        InMemoryCommentConfigService::new()
    }

    // --- parse_comment_input ---

    #[test]
    fn parse_pipe_separated() {
        let result = parse_comment_input("a|b|c");
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_newline_separated() {
        let result = parse_comment_input("a\nb");
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn parse_trims_and_filters_empty() {
        let result = parse_comment_input("  a  |  |b");
        assert_eq!(result, vec!["a", "b"]);
    }

    #[test]
    fn parse_pipe_takes_priority_over_newline() {
        // When text contains '|', split on '|', not '\n'.
        let result = parse_comment_input("a|b\nc");
        // "b\nc" is a single token since '|' is the separator
        assert_eq!(result, vec!["a", "b\nc"]);
    }

    // --- set_comments ---

    #[test]
    fn set_comments_replaces_existing() {
        let s = svc();
        s.set_comments("bot", vec!["first".into()]);
        s.set_comments("bot", vec!["second".into(), "third".into()]);
        let pool = s.get_comments("bot").expect("pool exists");
        assert_eq!(pool, vec!["second", "third"]);
    }

    #[test]
    fn set_comments_returns_count() {
        let s = svc();
        let count = s.set_comments("bot", vec!["a".into(), "b".into(), "c".into()]);
        assert_eq!(count, 3);
    }

    // --- append_comments ---

    #[test]
    fn append_comments_extends_existing() {
        let s = svc();
        s.set_comments("bot", vec!["a".into()]);
        let count = s.append_comments("bot", vec!["b".into(), "c".into()]);
        assert_eq!(count, 3);
        let pool = s.get_comments("bot").expect("pool exists");
        assert_eq!(pool, vec!["a", "b", "c"]);
    }

    #[test]
    fn append_comments_creates_pool_if_not_set() {
        let s = svc();
        let count = s.append_comments("bot", vec!["x".into()]);
        assert_eq!(count, 1);
    }

    // --- get_comments ---

    #[test]
    fn get_comments_returns_none_before_set() {
        let s = svc();
        assert!(s.get_comments("bot").is_none());
    }

    // --- clear_comments ---

    #[test]
    fn clear_comments_removes_override() {
        let s = svc();
        s.set_comments("bot", vec!["a".into()]);
        s.clear_comments("bot");
        assert!(s.get_comments("bot").is_none());
    }

    #[test]
    fn clear_comments_noop_when_not_set() {
        let s = svc();
        // Should not panic
        s.clear_comments("nonexistent");
    }

    // --- list_all ---

    #[test]
    fn list_all_returns_only_bots_with_overrides() {
        let s = svc();
        s.set_comments("botA", vec!["a".into()]);
        s.set_comments("botB", vec!["b".into(), "c".into()]);
        s.clear_comments("botA");
        let list = s.list_all();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].0, "botB");
        assert_eq!(list[0].1, 2);
    }

    #[test]
    fn list_all_empty_when_no_overrides() {
        let s = svc();
        assert!(s.list_all().is_empty());
    }

    #[test]
    fn list_all_sorted_alphabetically() {
        let s = svc();
        s.set_comments("zebra", vec!["z".into()]);
        s.set_comments("alpha", vec!["a".into()]);
        let list = s.list_all();
        assert_eq!(list[0].0, "alpha");
        assert_eq!(list[1].0, "zebra");
    }
}
