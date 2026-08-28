// Task readme command implementation
//
// Generates and maintains a `README.md` index for a tasks directory. This is
// opt-in (a CLI flag / subcommand, never automatic) so the shape of the file
// is always something the user asked for, not a surprise on every command.
//
// Two entry points:
//   - `create_readme`: (re)generate the whole file from the current task
//     files — used by the standalone `bert task readme` command, and by
//     `bert task stub --readme` the first time a README doesn't exist yet.
//   - `add_task_row`: append one row to an *existing* README's active-tasks
//     table, without touching anything else — used by `bert task stub
//     --readme` on every stub after the first.
use crate::errors::{BertError, Result};
use crate::format::{self, TaskEntry};
use crate::models::config::BertConfig;
use std::fs;
use std::path::PathBuf;

/// What happened when syncing the README after creating a task stub.
pub enum StubReadmeAction {
    /// No README existed yet; a fresh one (including the new task) was written.
    Created,
    /// An existing README's active-tasks table gained a row for the new task.
    RowAdded,
    /// An existing README has no active-tasks table this could confidently
    /// append to; left untouched.
    NoActiveTable,
}

/// (Re)generate `README.md` in the tasks directory from the current task
/// files. Fails with [`BertError::AlreadyExists`] if the file is already
/// there and `force` is false — regenerating overwrites any hand-written
/// content, so that always has to be opted into explicitly.
pub fn create_readme(config: &BertConfig, force: bool) -> Result<PathBuf> {
    let readme_path = config.tasks_directory.join("README.md");
    if readme_path.exists() && !force {
        return Err(BertError::AlreadyExists(readme_path.display().to_string()));
    }

    fs::create_dir_all(&config.tasks_directory)?;
    fs::write(&readme_path, render_readme(config))?;
    Ok(readme_path)
}

/// Sync the README after `bert task stub` created a new task: create the
/// file if missing (the fresh generation already includes the new task,
/// since it scans the directory after the stub was written), otherwise
/// append just that task's row to the existing active-tasks table.
pub fn sync_after_stub(config: &BertConfig, task_number: &str) -> Result<StubReadmeAction> {
    let readme_path = config.tasks_directory.join("README.md");
    if !readme_path.exists() {
        create_readme(config, true)?;
        return Ok(StubReadmeAction::Created);
    }

    let entries = format::scan_task_entries(&config.tasks_directory);
    let Some(entry) = entries.iter().find(|e| e.number == task_number) else {
        return Ok(StubReadmeAction::NoActiveTable);
    };

    if append_active_row(&readme_path, &active_row(entry))? {
        Ok(StubReadmeAction::RowAdded)
    } else {
        Ok(StubReadmeAction::NoActiveTable)
    }
}

fn cell(value: Option<&str>) -> String {
    match value {
        Some(v) if !v.is_empty() => format!("`{}`", v),
        _ => "-".to_string(),
    }
}

/// Render one active-tasks table row for an entry, in the same shape
/// [`render_readme`] generates (and that hand-written READMEs in the wild
/// tend to use): backtick-wrapped track/priority/status.
fn active_row(entry: &TaskEntry) -> String {
    let filename = entry
        .path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or_default();
    let status = if entry.status.is_empty() { "-".to_string() } else { format!("`{}`", entry.status) };
    format!(
        "| [`task-{num}`]({file}) | {title} | {track} | {priority} | {status} |",
        num = entry.number,
        file = filename,
        title = entry.title,
        track = cell(entry.track.as_deref()),
        priority = cell(entry.priority.as_deref()),
        status = status,
    )
}

/// Append `row` right after the last row of the first markdown table found
/// before any "Completed"-style heading (i.e. the active-tasks table).
/// Returns false, touching nothing, if no such table can be found.
fn append_active_row(readme_path: &PathBuf, row: &str) -> Result<bool> {
    let content = fs::read_to_string(readme_path)?;
    let mut lines: Vec<String> = content.lines().map(str::to_string).collect();

    let boundary = lines
        .iter()
        .position(|l| format::is_completed_heading(l))
        .unwrap_or(lines.len());

    let Some(last_table_line) = lines[..boundary].iter().rposition(|l| l.trim_start().starts_with('|')) else {
        return Ok(false);
    };

    lines.insert(last_table_line + 1, row.to_string());
    let mut new_content = lines.join("\n");
    if content.ends_with('\n') {
        new_content.push('\n');
    }
    fs::write(readme_path, new_content)?;
    Ok(true)
}

/// Build the full README content from scratch: an active-tasks table
/// (everything in the tasks directory) and, when an archive directory is
/// configured and has files in it, a completed-tasks table.
fn render_readme(config: &BertConfig) -> String {
    let active = format::scan_task_entries(&config.tasks_directory);
    let done: Vec<TaskEntry> = config
        .archive_tasks_directory
        .as_deref()
        .map(format::scan_task_entries)
        .unwrap_or_default();

    let mut out = String::new();
    out.push_str("# Task Tracking Index\n\n");
    out.push_str("Tasks are tracked as individual files in this directory. Completed tasks move to the archive location with `status` set to done — the full file stays intact there rather than being deleted or summarized.\n\n");

    out.push_str("## Active Tasks\n\n");
    if active.is_empty() {
        out.push_str("_No active tasks yet._\n\n");
    } else {
        out.push_str("| ID | Title | Track | Priority | Status |\n");
        out.push_str("| :--- | :--- | :--- | :---: | :---: |\n");
        for entry in &active {
            out.push_str(&active_row(entry));
            out.push('\n');
        }
        out.push('\n');
    }

    out.push_str("## Completed Tasks\n\n");
    if done.is_empty() {
        out.push_str("_No completed tasks yet._\n");
    } else {
        out.push_str("| ID | Title | Track | Completed |\n");
        out.push_str("| :--- | :--- | :--- | :---: |\n");
        for entry in &done {
            let filename = entry.path.file_name().and_then(|f| f.to_str()).unwrap_or_default();
            let link = config
                .archive_tasks_directory
                .as_deref()
                .map(|dir| format::relative_link(&config.tasks_directory, &dir.join(filename)))
                .unwrap_or_else(|| filename.to_string());
            out.push_str(&format!(
                "| [`task-{num}`]({link}) | {title} | {track} | {completed} |\n",
                num = entry.number,
                link = link,
                title = entry.title,
                track = cell(entry.track.as_deref()),
                completed = entry.updated.as_deref().unwrap_or("-"),
            ));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::test_support::test_config;
    use tempfile::TempDir;

    #[test]
    fn test_create_readme_writes_active_and_completed_tables() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(
            config.tasks_directory.join("task-01-alpha.md"),
            "---\nstatus: open\ntrack: wire\npriority: p1\n---\n\n# task-01: Alpha\n",
        )
        .unwrap();
        let archive_dir = config.archive_tasks_directory.clone().unwrap();
        fs::create_dir_all(&archive_dir).unwrap();
        fs::write(
            archive_dir.join("task-02-beta.md"),
            "---\nstatus: done\ntrack: newsletter\nupdated: 2026-08-20\n---\n\n# task-02: Beta\n",
        )
        .unwrap();

        let path = create_readme(&config, false).unwrap();
        let content = fs::read_to_string(&path).unwrap();

        assert!(content.contains("| [`task-01`](task-01-alpha.md) | Alpha | `wire` | `p1` | `open` |"));
        assert!(content.contains("[`task-02`]"));
        assert!(content.contains("| Beta | `newsletter` | 2026-08-20 |"));
    }

    #[test]
    fn test_create_readme_refuses_to_overwrite_without_force() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("README.md"), "hand-written\n").unwrap();

        let err = create_readme(&config, false).unwrap_err();
        assert!(matches!(err, BertError::AlreadyExists(_)));
        assert_eq!(fs::read_to_string(config.tasks_directory.join("README.md")).unwrap(), "hand-written\n");
    }

    #[test]
    fn test_create_readme_force_overwrites() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("README.md"), "hand-written\n").unwrap();

        create_readme(&config, true).unwrap();

        let content = fs::read_to_string(config.tasks_directory.join("README.md")).unwrap();
        assert!(content.contains("# Task Tracking Index"));
    }

    #[test]
    fn test_sync_after_stub_creates_when_missing() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-01-alpha.md"), "---\nstatus: open\n---\n\n# task-01: Alpha\n").unwrap();

        let action = sync_after_stub(&config, "01").unwrap();
        assert!(matches!(action, StubReadmeAction::Created));
        assert!(config.tasks_directory.join("README.md").exists());
    }

    #[test]
    fn test_sync_after_stub_appends_row_to_existing_readme() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(
            config.tasks_directory.join("task-01-alpha.md"),
            "---\nstatus: open\ntrack: wire\npriority: p1\n---\n\n# task-01: Alpha\n",
        )
        .unwrap();
        fs::write(
            config.tasks_directory.join("task-02-beta.md"),
            "---\nstatus: open\ntrack: newsletter\npriority: p2\n---\n\n# task-02: Beta\n",
        )
        .unwrap();
        fs::write(
            config.tasks_directory.join("README.md"),
            "# Tasks\n\n| ID | Title | Track | Priority | Status |\n| :--- | :--- | :--- | :---: | :---: |\n| [`task-01`](task-01-alpha.md) | Alpha | `wire` | `p1` | `open` |\n\n## Completed Tasks\n\n_No completed tasks yet._\n",
        )
        .unwrap();

        let action = sync_after_stub(&config, "02").unwrap();
        assert!(matches!(action, StubReadmeAction::RowAdded));

        let content = fs::read_to_string(config.tasks_directory.join("README.md")).unwrap();
        assert!(content.contains("[`task-01`](task-01-alpha.md)"), "existing row must survive");
        assert!(content.contains("| [`task-02`](task-02-beta.md) | Beta | `newsletter` | `p2` | `open` |"));
        // New row lands in the active table, before the Completed heading.
        assert!(content.find("task-02-beta").unwrap() < content.find("Completed Tasks").unwrap());
    }

    #[test]
    fn test_sync_after_stub_reports_no_active_table() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-01-alpha.md"), "---\nstatus: open\n---\n\n# task-01: Alpha\n").unwrap();
        fs::write(config.tasks_directory.join("README.md"), "# Tasks\n\nno tables here\n").unwrap();

        let action = sync_after_stub(&config, "01").unwrap();
        assert!(matches!(action, StubReadmeAction::NoActiveTable));
        assert_eq!(
            fs::read_to_string(config.tasks_directory.join("README.md")).unwrap(),
            "# Tasks\n\nno tables here\n"
        );
    }
}
