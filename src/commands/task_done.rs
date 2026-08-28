// Task done command implementation
//
// Composes three steps that, until now, had to be done by hand:
//   a) flip the task's frontmatter `status:` to the directory's "done" word
//   b) archive the task (and notes/children) out of the active tasks directory
//   c) keep a tasks-directory `README.md` index in sync: drop the task's
//      active-section link lines and add a row to its "Completed" table
//
// Step (c) is best-effort. README shapes vary by project; this only edits
// lines it can identify with confidence (a link to the task being closed, or
// an existing "Completed"-style table) and never touches anything else.
use crate::commands::task_archive;
use crate::errors::Result;
use crate::format::{self, Status};
use crate::models::config::BertConfig;
use chrono::Local;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Outcome of marking one or more tasks done, for CLI reporting.
pub struct DoneReport {
    /// Task + notes + (if recursive) children files moved to the archive.
    pub archived_count: usize,
    /// Active-section lines removed from the tasks-directory README, if any.
    pub readme_rows_removed: usize,
    /// Rows added to the README's "Completed" table, if any.
    pub readme_rows_added: bool,
}

/// Mark a task (and optionally its children) done: update frontmatter,
/// archive the files, and sync the tasks-directory README if present.
pub fn mark_task_done(config: &BertConfig, task_number: &str, recursive: bool) -> Result<DoneReport> {
    let done_word = detect_done_word(config);
    let today = Local::now().format("%Y-%m-%d").to_string();

    // Canonical on-disk numbers for the target and, if requested, its
    // descendants — resolved before anything moves.
    let target = canonical_number(config, task_number)?;
    let mut numbers = vec![target.clone()];
    if recursive {
        numbers.extend(task_archive::find_all_children(config, &target)?);
    }

    // Snapshot title/track/filename for the README before the files move.
    let entries = format::scan_task_entries(&config.tasks_directory);
    let readme_rows: Vec<(String, String, String, String)> = numbers
        .iter()
        .filter_map(|num| {
            let entry = entries.iter().find(|e| &e.number == num)?;
            let filename = task_archive::find_task_file(config, num)
                .ok()?
                .file_name()?
                .to_str()?
                .to_string();
            Some((entry.number.clone(), entry.title.clone(), entry.track.clone().unwrap_or_default(), filename))
        })
        .collect();

    for num in &numbers {
        let path = task_archive::find_task_file(config, num)?;
        update_status_frontmatter(&path, &done_word, &today)?;
    }

    let archived_count = task_archive::archive_task(config, &target, recursive)?;

    // Now that files have moved, resolve each one's README-relative link.
    let readme_rows: Vec<(String, String, String, String)> = readme_rows
        .into_iter()
        .map(|(num, title, track, filename)| {
            let link = config
                .archive_tasks_directory
                .as_deref()
                .map(|dir| format::relative_link(&config.tasks_directory, &dir.join(&filename)))
                .unwrap_or(filename);
            (num, title, track, link)
        })
        .collect();

    let readme_path = config.tasks_directory.join("README.md");
    let (readme_rows_removed, readme_rows_added) = if readme_path.exists() {
        sync_readme(&readme_path, &readme_rows, &today)?
    } else {
        (0, false)
    };

    Ok(DoneReport {
        archived_count,
        readme_rows_removed,
        readme_rows_added,
    })
}

/// Resolve the on-disk spelling of a task number (handles any padding width).
fn canonical_number(config: &BertConfig, task_number: &str) -> Result<String> {
    let path = task_archive::find_task_file(config, task_number)?;
    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
    Ok(format::parse_task_filename(filename)
        .map(|p| p.number)
        .unwrap_or_else(|| task_number.to_string()))
}

/// The directory's raw word for a finished task: the observed status word
/// (across both the active and archive directories) that normalizes to
/// [`Status::Done`], falling back to `"done"` when none has been seen yet.
fn detect_done_word(config: &BertConfig) -> String {
    let mut candidates = format::detect_profile(&config.tasks_directory).observed_statuses;
    if let Some(archive_dir) = &config.archive_tasks_directory {
        candidates.extend(format::analyze_tasks(archive_dir).0.observed_statuses);
    }
    candidates
        .into_iter()
        .find(|word| format::normalize_status(word) == Some(Status::Done))
        .unwrap_or_else(|| "done".to_string())
}

/// Rewrite `status:` (and `updated:`, if present) in a task file's
/// frontmatter in place. A no-op when the file has no frontmatter block.
fn update_status_frontmatter(path: &Path, done_word: &str, today: &str) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let Some(rest) = content.strip_prefix("---\n") else {
        return Ok(());
    };
    let Some(end) = rest.find("\n---") else {
        return Ok(());
    };
    let block = &rest[..end];
    let after = &rest[end..];

    let new_block: Vec<String> = block
        .lines()
        .map(|line| {
            if line.starts_with("status:") {
                format!("status: {}", done_word)
            } else if line.starts_with("updated:") {
                format!("updated: {}", today)
            } else {
                line.to_string()
            }
        })
        .collect();

    let new_content = format!("---\n{}{}", new_block.join("\n"), after);
    fs::write(path, new_content)?;
    Ok(())
}

/// Does this line link to `task-{number}` (any padding-insensitive form)?
/// Matches both table cells and bullet items: `[\`task-009\`](task-009.md)`.
fn line_references_task(line: &str, number: &str, link_re: &Regex) -> bool {
    link_re
        .captures(line)
        .map(|caps| format::number_matches(&caps[1], number))
        .unwrap_or(false)
}

/// Drop active-section lines linking to any of `rows`, then append a row per
/// task to the README's "Completed" table (found by heading text). Rows are
/// `(number, title, track, link)`. Returns (rows removed, whether a
/// completed-table row was appended).
fn sync_readme(readme_path: &Path, rows: &[(String, String, String, String)], today: &str) -> Result<(usize, bool)> {
    if rows.is_empty() {
        return Ok((0, false));
    }

    let content = fs::read_to_string(readme_path)?;
    let lines: Vec<String> = content.lines().map(str::to_string).collect();
    let link_re = Regex::new(r"\[`task-(\d+(?:\.\d+)*)`\]").expect("valid regex");

    let heading_idx = lines.iter().position(|l| format::is_completed_heading(l));
    let split_at = heading_idx.unwrap_or(lines.len());
    let (mut before, mut after) = (lines[..split_at].to_vec(), lines[split_at..].to_vec());

    let before_len = before.len();
    before.retain(|line| {
        !rows
            .iter()
            .any(|(num, ..)| line_references_task(line, num, &link_re))
    });
    let removed = before_len - before.len();

    // Find the last row of the completed table (the last line starting with
    // `|` under the heading) and append new rows right after it.
    let mut added = false;
    if let Some(last_table_line) = after.iter().rposition(|l| l.trim_start().starts_with('|')) {
        let mut insert_at = last_table_line + 1;
        for (num, title, track, link) in rows {
            let row = format!(
                "| [`task-{num}`]({link}) | {title} | `{track}` | {today} |",
                num = num,
                link = link,
                title = title,
                track = track,
                today = today,
            );
            after.insert(insert_at, row);
            insert_at += 1;
            added = true;
        }
    }

    if removed == 0 && !added {
        return Ok((0, false));
    }

    let mut new_lines = before;
    new_lines.extend(after);
    let mut new_content = new_lines.join("\n");
    if content.ends_with('\n') {
        new_content.push('\n');
    }
    fs::write(readme_path, new_content)?;

    Ok((removed, added))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::test_support::test_config;
    use tempfile::TempDir;

    #[test]
    fn test_update_status_frontmatter_sets_done_and_updated() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("task-01.md");
        fs::write(
            &path,
            "---\nstatus: open\ncreated: 2026-01-01\nupdated: 2026-01-01\n---\n\n# task-01: Thing\n",
        )
        .unwrap();

        update_status_frontmatter(&path, "done", "2026-08-28").unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("status: done"));
        assert!(content.contains("updated: 2026-08-28"));
        assert!(content.contains("created: 2026-01-01"));
        assert!(content.contains("# task-01: Thing"));
    }

    #[test]
    fn test_update_status_frontmatter_noop_without_frontmatter() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("task-01.md");
        fs::write(&path, "# task-01: Thing\n").unwrap();

        update_status_frontmatter(&path, "done", "2026-08-28").unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "# task-01: Thing\n");
    }

    #[test]
    fn test_detect_done_word_prefers_observed_synonym() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = test_config(temp_dir.path());
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(
            config.tasks_directory.join("task-01.md"),
            "---\nstatus: open\n---\n",
        )
        .unwrap();
        let archive_dir = config.archive_tasks_directory.clone().unwrap();
        fs::create_dir_all(&archive_dir).unwrap();
        fs::write(archive_dir.join("task-02.md"), "---\nstatus: completed\n---\n").unwrap();
        config.archive_tasks_directory = Some(archive_dir);

        assert_eq!(detect_done_word(&config), "completed");
    }

    #[test]
    fn test_detect_done_word_defaults_to_done() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        fs::create_dir_all(&config.tasks_directory).unwrap();

        assert_eq!(detect_done_word(&config), "done");
    }

    #[test]
    fn test_sync_readme_removes_active_rows_and_appends_completed_row() {
        let temp_dir = TempDir::new().unwrap();
        let readme_path = temp_dir.path().join("README.md");
        fs::write(
            &readme_path,
            "\
# Tasks

| ID | Title | Track | Priority | Status |
| :--- | :--- | :--- | :---: | :---: |
| [`task-009`](task-009.md) | Product decision | `wire` | `p3` | `open` |
| [`task-010`](task-010.md) | Other task | `wire` | `p2` | `open` |

### Wire Track
- [`task-009`](task-009.md) **Product decision** (`p3`, `open`)
- [`task-010`](task-010.md) **Other task** (`p2`, `open`)

## Completed Tasks

| ID | Title | Track | Completed |
| :--- | :--- | :--- | :---: |
| [`task-012`](done/task-012.md) | Push origin/main | `wire` | 2026-08-28 |
",
        )
        .unwrap();

        let rows = vec![(
            "009".to_string(),
            "Product decision".to_string(),
            "wire".to_string(),
            "done/task-009.md".to_string(),
        )];
        let (removed, added) = sync_readme(&readme_path, &rows, "2026-08-29").unwrap();

        assert_eq!(removed, 2);
        assert!(added);

        let content = fs::read_to_string(&readme_path).unwrap();
        assert!(!content.contains("(task-009.md)"), "active references to task-009 should be gone");
        assert!(content.contains("[`task-010`](task-010.md)"), "unrelated rows must survive");
        assert!(content.contains("[`task-012`](done/task-012.md)"), "existing completed row must survive");
        assert!(content.contains("[`task-009`](done/task-009.md) | Product decision | `wire` | 2026-08-29 |"));

        // The new row lands after the existing completed row, not before it.
        let idx_012 = content.find("task-012").unwrap();
        let idx_009_done = content.find("done/task-009").unwrap();
        assert!(idx_012 < idx_009_done);
    }

    #[test]
    fn test_sync_readme_noop_without_completed_table() {
        let temp_dir = TempDir::new().unwrap();
        let readme_path = temp_dir.path().join("README.md");
        fs::write(&readme_path, "# Tasks\n\nno links, no table\n").unwrap();

        let rows = vec![(
            "009".to_string(),
            "Product decision".to_string(),
            "wire".to_string(),
            "done/task-009.md".to_string(),
        )];
        let (removed, added) = sync_readme(&readme_path, &rows, "2026-08-29").unwrap();

        assert_eq!(removed, 0);
        assert!(!added);
    }
}
