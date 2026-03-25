use std::time::{SystemTime, UNIX_EPOCH};

/// A single command entered by the user.
#[derive(Debug, Clone)]
pub struct Command {
    /// The command text (trimmed).
    pub text: String,
    /// Unix timestamp (seconds since epoch) when the command was entered.
    pub timestamp: u64,
}

/// Append-only command history.
///
/// Empty commands are silently ignored.  Consecutive identical commands are
/// deduplicated so that rapidly re-entering the same command does not flood the
/// history with redundant entries.  Non-consecutive duplicates are always
/// recorded — the "don't re-add" logic for navigated-history re-runs is handled
/// at the call site in the key handler, not here.
#[derive(Debug, Default)]
pub struct CommandHistory {
    entries: Vec<Command>,
}

impl CommandHistory {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// Append a command to the history.
    ///
    /// Rules:
    /// - Blank (whitespace-only) commands are ignored.
    /// - If the new text is identical to the most recent entry it is not added
    ///   again (consecutive-duplicate suppression).
    pub fn push(&mut self, text: impl Into<String>) {
        let text = text.into();
        let text = text.trim().to_string();
        if text.is_empty() {
            return;
        }
        if self.entries.last().map(|e| e.text.as_str()) == Some(text.as_str()) {
            return;
        }
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.entries.push(Command { text, timestamp });
    }

    /// All entries, ordered from oldest (index 0) to most recent.
    pub fn entries(&self) -> &[Command] {
        &self.entries
    }

    /// Number of recorded commands.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return the index of the (skip+1)-th most-recent entry whose text
    /// contains `query` (case-insensitive). Returns `None` when no such entry
    /// exists.
    pub fn search_reverse(&self, query: &str, skip: usize) -> Option<usize> {
        let q = query.to_lowercase();
        self.entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.text.to_lowercase().contains(&q))
            .rev()
            .nth(skip)
            .map(|(i, _)| i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_commands_not_added() {
        let mut h = CommandHistory::new();
        h.push("");
        h.push("   ");
        assert!(h.is_empty());
    }

    #[test]
    fn consecutive_duplicates_suppressed() {
        let mut h = CommandHistory::new();
        h.push("users");
        h.push("users");
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn non_consecutive_duplicates_allowed() {
        let mut h = CommandHistory::new();
        h.push("users");
        h.push("orders");
        h.push("users");
        assert_eq!(h.len(), 3);
    }

    #[test]
    fn search_reverse_finds_most_recent_first() {
        let mut h = CommandHistory::new();
        h.push("users where id = 1");
        h.push("orders");
        h.push("users where id = 2");
        // Most recent match for "users" is index 2.
        assert_eq!(h.search_reverse("users", 0), Some(2));
        // Second most recent is index 0.
        assert_eq!(h.search_reverse("users", 1), Some(0));
        // No third match.
        assert_eq!(h.search_reverse("users", 2), None);
    }

    #[test]
    fn search_reverse_no_match() {
        let mut h = CommandHistory::new();
        h.push("orders");
        assert_eq!(h.search_reverse("users", 0), None);
    }
}
