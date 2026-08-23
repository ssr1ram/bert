// Task list command implementation
//
// Reads the tasks directory leniently (see crate::format) and renders an
// overview table — or machine-readable JSON with --json. Filtering matches
// both canonical statuses (todo/doing/done/blocked/parked) and whatever raw
// vocabulary the directory uses (open/paused/deferred/...).
use crate::errors::{BertError, Result};
use crate::format::{self, TaskEntry};
use crate::models::config::BertConfig;
use std::collections::BTreeMap;

/// Filters applied to scanned entries; every set field must match.
#[derive(Debug, Default, Clone)]
pub struct ListFilter {
    pub status: Option<String>,
    pub track: Option<String>,
    pub priority: Option<String>,
    pub tag: Option<String>,
}

impl ListFilter {
    pub fn is_empty(&self) -> bool {
        self.status.is_none() && self.track.is_none() && self.priority.is_none() && self.tag.is_none()
    }

    pub fn matches(&self, entry: &TaskEntry) -> bool {
        self.status
            .as_deref()
            .map(|f| format::status_matches(&entry.status, f))
            .unwrap_or(true)
            && self
                .track
                .as_deref()
                .map(|f| format::field_matches(&entry.track, f))
                .unwrap_or(true)
            && self
                .priority
                .as_deref()
                .map(|f| format::field_matches(&entry.priority, f))
                .unwrap_or(true)
            && self
                .tag
                .as_deref()
                .map(|f| format::has_tag(entry, f))
                .unwrap_or(true)
    }
}

/// Scan, filter and print the task listing.
pub fn list_tasks(config: &BertConfig, filter: &ListFilter, json: bool) -> Result<()> {
    let tasks = format::scan_task_entries(&config.tasks_directory);
    let total = tasks.len();
    let selected: Vec<TaskEntry> = tasks
        .into_iter()
        .filter(|t| filter.matches(t))
        .collect();

    if json {
        let out = serde_json::to_string_pretty(&selected)
            .map_err(|e| BertError::ConfigError(format!("failed to serialize JSON: {}", e)))?;
        println!("{}", out);
        return Ok(());
    }

    if selected.is_empty() {
        if filter.is_empty() {
            println!(
                "No tasks found in {}.",
                config.tasks_directory.display()
            );
        } else {
            println!("No tasks match the given filters.");
        }
        return Ok(());
    }

    print!("{}", render_table(&selected));

    // Summary over the selected entries, in canonical vocabulary.
    let mut counts: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut unmatched = 0usize;
    for t in &selected {
        match t.canonical_status {
            Some(s) => *counts.entry(s.as_str()).or_insert(0) += 1,
            None => unmatched += 1,
        }
    }
    let parts: Vec<String> = counts
        .iter()
        .map(|(k, v)| format!("{} {}", v, k))
        .chain((unmatched > 0).then(|| format!("{} unknown", unmatched)))
        .collect();
    print!(
        "\n{} task(s): {}",
        selected.len(),
        parts.join(", ")
    );
    if !filter.is_empty() {
        print!(" (filtered from {})", total);
    }
    println!();

    Ok(())
}

/// Render entries as an aligned table; subtasks are indented under parents.
fn render_table(entries: &[TaskEntry]) -> String {
    const HEADER: [&str; 5] = ["NUM", "STATUS", "PRI", "TRACK", "TITLE"];

    let num_w = entries
        .iter()
        .map(|e| e.number.len() + e.depth() * 2)
        .max()
        .unwrap_or(0)
        .max(HEADER[0].len());
    let status_w = entries
        .iter()
        .map(|e| e.status.len())
        .max()
        .unwrap_or(0)
        .max(HEADER[1].len());
    let pri_w = entries
        .iter()
        .filter_map(|e| e.priority.as_deref().map(str::len))
        .max()
        .unwrap_or(0)
        .max(HEADER[2].len());
    let track_w = entries
        .iter()
        .filter_map(|e| e.track.as_deref().map(str::len))
        .max()
        .unwrap_or(0)
        .max(HEADER[3].len());

    let mut out = String::new();
    out.push_str(&format!(
        "{:<num_w$}  {:<status_w$}  {:<pri_w$}  {:<track_w$}  {}\n",
        HEADER[0],
        HEADER[1],
        HEADER[2],
        HEADER[3],
        HEADER[4],
        num_w = num_w,
        status_w = status_w,
        pri_w = pri_w,
        track_w = track_w,
    ));

    for e in entries {
        let indent = "  ".repeat(e.depth());
        out.push_str(&format!(
            "{:<num_w$}  {:<status_w$}  {:<pri_w$}  {:<track_w$}  {}\n",
            format!("{}{}", indent, e.number),
            if e.status.is_empty() { "-" } else { &e.status },
            e.priority.as_deref().unwrap_or("-"),
            e.track.as_deref().unwrap_or("-"),
            e.title,
            num_w = num_w,
            status_w = status_w,
            pri_w = pri_w,
            track_w = track_w,
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(number: &str, status: &str, track: Option<&str>, priority: Option<&str>, tags: &[&str]) -> TaskEntry {
        TaskEntry {
            number: number.to_string(),
            slug: None,
            title: format!("Task {}", number),
            status: status.to_string(),
            canonical_status: format::normalize_status(status),
            track: track.map(|s| s.to_string()),
            priority: priority.map(|s| s.to_string()),
            created: None,
            updated: None,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            references: Vec::new(),
            path: std::path::PathBuf::from(format!("task-{}.md", number)),
        }
    }

    #[test]
    fn test_status_filter_matches_synonyms_across_vocabulary() {
        let _open = entry("001", "open", None, None, &[]);
        let deferred = entry("010", "deferred", None, None, &[]);
        let paused = entry("013", "paused", None, None, &[]);

        assert!(format::status_matches("open", "todo"));
        assert!(format::status_matches("pending", "open"));
        assert!(format::status_matches("paused", "parked"));
        assert!(format::status_matches("deferred", "parked"));
        assert!(!format::status_matches("open", "done"));

        // A filter value no synonym table knows falls back to raw compare
        assert!(format::status_matches("shipped", "SHIPPED"));
        assert!(!format::status_matches("open", "shipped"));
        assert!(!deferred.status.is_empty() && !paused.status.is_empty());
    }

    #[test]
    fn test_combined_filters() {
        let f = ListFilter {
            status: Some("todo".into()),
            track: Some("wire".into()),
            priority: Some("p1".into()),
            tag: Some("cron".into()),
        };
        let yes = entry("001", "open", Some("wire"), Some("p1"), &["cron", "history"]);
        let wrong_track = entry("002", "open", Some("posts"), Some("p1"), &["cron"]);
        let wrong_pri = entry("003", "open", Some("wire"), Some("p2"), &["cron"]);
        let wrong_status = entry("004", "done", Some("wire"), Some("p1"), &["cron"]);
        let missing_tag = entry("005", "open", Some("wire"), Some("p1"), &[]);

        assert!(f.matches(&yes));
        assert!(!f.matches(&wrong_track));
        assert!(!f.matches(&wrong_pri));
        assert!(!f.matches(&wrong_status));
        assert!(!f.matches(&missing_tag));
    }

    #[test]
    fn test_render_table_aligns_and_indents_subtasks() {
        let entries = vec![
            entry("8", "open", Some("wire"), Some("p1"), &[]),
            entry("08.1", "done", None, None, &[]),
        ];
        let table = render_table(&entries);
        let lines: Vec<&str> = table.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("NUM"));
        assert!(lines[0].contains("TITLE"));
        // Subtask indented two spaces within the number column
        assert!(lines[2].trim_start().starts_with("08.1"));
        assert!(lines[2].starts_with("    08.1") || lines[2].contains("  08.1"));
        // Missing fields render as dash
        assert!(lines[2].contains("-"));
    }

    #[test]
    fn test_render_table_raw_status_words_survive() {
        let entries = vec![entry("013", "paused", Some("newsletter"), Some("p1"), &[])];
        let table = render_table(&entries);
        assert!(table.contains("paused"), "raw word should be displayed");
        assert!(table.contains("newsletter"));
    }
}
