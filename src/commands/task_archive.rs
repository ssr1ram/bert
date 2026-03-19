// Task archive command implementation
use crate::errors::{BertError, Result};
use crate::models::config::BertConfig;
use crate::utils::normalize_task_number;
use std::fs;
use std::path::PathBuf;

/// Archive a task and its associated notes
///
/// # Arguments
///
/// * `config` - Bert configuration
/// * `task_number` - Task number to archive (e.g., "08" or "08.1")
/// * `recursive` - Whether to recursively archive child tasks (Task 08.9)
///
/// # Returns
///
/// Returns count of files archived (tasks + notes)
pub fn archive_task(
    config: &BertConfig,
    task_number: &str,
    recursive: bool,
) -> Result<usize> {
    if recursive {
        // Archive parent and all children recursively
        archive_with_children(config, task_number)
    } else {
        // Archive only the specified task
        archive_single_task(config, task_number)
    }
}

/// Archive a single task without children
fn archive_single_task(config: &BertConfig, task_number: &str) -> Result<usize> {
    // Find task file
    let task_file = find_task_file(config, task_number)?;

    // Find associated notes file (may not exist)
    let notes_file = find_notes_file(config, task_number);

    // Ensure archive directories exist
    let archive_tasks_dir = config.archive_tasks_directory.as_ref()
        .ok_or_else(|| BertError::ConfigError("archive_tasks_directory not configured".to_string()))?;

    fs::create_dir_all(archive_tasks_dir)?;

    // Archive task file
    let task_filename = task_file.file_name()
        .ok_or_else(|| BertError::FileError("Invalid task filename".to_string()))?;
    let archive_task_path = archive_tasks_dir.join(task_filename);
    fs::rename(&task_file, &archive_task_path)?;

    let mut archived_count = 1;

    // Archive notes file if it exists
    if let Some(notes_path) = notes_file {
        if let Some(archive_notes_dir) = config.archive_notes_directory.as_ref() {
            fs::create_dir_all(archive_notes_dir)?;

            let notes_filename = notes_path.file_name()
                .ok_or_else(|| BertError::FileError("Invalid notes filename".to_string()))?;
            let archive_notes_path = archive_notes_dir.join(notes_filename);

            fs::rename(&notes_path, &archive_notes_path)?;
            archived_count += 1;
        }
    }

    Ok(archived_count)
}

/// Archive a task and all its children recursively
fn archive_with_children(config: &BertConfig, task_number: &str) -> Result<usize> {
    let mut total_archived = 0;

    // First archive the parent task itself
    total_archived += archive_single_task(config, task_number)?;

    // Find and archive all children recursively
    let children = find_all_children(config, task_number)?;

    for child_number in children {
        // Recursively archive each child (which will archive its children too)
        total_archived += archive_single_task(config, &child_number)?;
    }

    Ok(total_archived)
}

/// Find all children (and grandchildren, etc.) of a task
fn find_all_children(config: &BertConfig, parent_number: &str) -> Result<Vec<String>> {
    let mut children = Vec::new();
    // Normalize parent number (e.g., "1" -> "01")
    let normalized = normalize_task_number(parent_number);
    let pattern = format!("task-{}.", normalized);
    let entries = fs::read_dir(&config.tasks_directory)?;

    for entry in entries.flatten() {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.starts_with(&pattern) && filename.ends_with(".md") {
                // Extract the task number from filename
                // e.g., "task-08.1-foo.md" -> "08.1"
                if let Some(task_num_part) = filename
                    .strip_prefix("task-")
                    .and_then(|s| s.split('-').next())
                {
                    children.push(task_num_part.to_string());

                    // Recursively find children of this child
                    let grandchildren = find_all_children(config, task_num_part)?;
                    children.extend(grandchildren);
                }
            }
        }
    }

    // Remove duplicates and sort
    children.sort();
    children.dedup();

    Ok(children)
}

/// Find task file by task number
///
/// Returns PathBuf to the task file, or error if not found
fn find_task_file(config: &BertConfig, task_number: &str) -> Result<PathBuf> {
    // Normalize task number (e.g., "1" -> "01")
    let normalized = normalize_task_number(task_number);
    let pattern = format!("task-{}-", normalized);
    let entries = fs::read_dir(&config.tasks_directory)?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.starts_with(&pattern) && filename.ends_with(".md") {
                return Ok(path);
            }
        }
    }

    Err(BertError::TaskNotFound(task_number.to_string()))
}

/// Find notes file by task number
///
/// Returns Some(PathBuf) if found, None if not found (not an error)
fn find_notes_file(config: &BertConfig, task_number: &str) -> Option<PathBuf> {
    let notes_dir = config.notes_directory.as_ref()?;

    if !notes_dir.exists() {
        return None;
    }

    // Normalize task number (e.g., "1" -> "01")
    let normalized = normalize_task_number(task_number);
    let pattern = format!("note-{}-", normalized);
    let entries = fs::read_dir(notes_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.starts_with(&pattern) && filename.ends_with(".md") {
                return Some(path);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_config(temp_dir: &TempDir) -> BertConfig {
        let root = temp_dir.path();
        BertConfig {
            project_root: root.to_path_buf(),
            bert_root: root.join("bert"),
            tasks_directory: root.join("tasks"),
            notes_directory: Some(root.join("notes")),
            archive_tasks_directory: Some(root.join("archive/tasks")),
            archive_notes_directory: Some(root.join("archive/notes")),
            specs_directory: root.join("specs"),
            archive_specs_directory: Some(root.join("archive/specs")),
            archive_directory: Some(root.join("archive")),
            product_directory: Some(root.join("product")),
            prompt_logs: Some(root.join("prompt-logs")),
            library_directory: Some(root.join("library")),
            sets_directory: Some(root.join("sets")),
            tui: crate::models::config::TuiConfig {
                pane_widths: Some(crate::models::config::PaneWidths::default()),
            },
        }
    }

    #[test]
    fn test_find_task_file_success() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-08-test.md"), "test").unwrap();

        let found = find_task_file(&config, "08").unwrap();
        assert!(found.to_str().unwrap().contains("task-08-test.md"));
    }

    #[test]
    fn test_find_task_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        fs::create_dir_all(&config.tasks_directory).unwrap();

        let result = find_task_file(&config, "08");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BertError::TaskNotFound(_)));
    }

    #[test]
    fn test_find_notes_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        fs::create_dir_all(config.notes_directory.as_ref().unwrap()).unwrap();
        fs::write(
            config.notes_directory.as_ref().unwrap().join("note-08-test.md"),
            "note"
        ).unwrap();

        let found = find_notes_file(&config, "08");
        assert!(found.is_some());
    }

    #[test]
    fn test_find_notes_file_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        fs::create_dir_all(config.notes_directory.as_ref().unwrap()).unwrap();

        let found = find_notes_file(&config, "08");
        assert!(found.is_none());
    }

    #[test]
    fn test_archive_task_without_notes() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        // Create task file
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-08-test.md"), "task content").unwrap();

        // Archive
        let count = archive_task(&config, "08", false).unwrap();
        assert_eq!(count, 1);

        // Verify task was moved
        assert!(!config.tasks_directory.join("task-08-test.md").exists());
        assert!(config.archive_tasks_directory.as_ref().unwrap()
            .join("task-08-test.md").exists());
    }

    #[test]
    fn test_archive_task_with_notes() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        // Create task and notes files
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::create_dir_all(config.notes_directory.as_ref().unwrap()).unwrap();

        fs::write(config.tasks_directory.join("task-08-test.md"), "task").unwrap();
        fs::write(
            config.notes_directory.as_ref().unwrap().join("note-08-test.md"),
            "notes"
        ).unwrap();

        // Archive
        let count = archive_task(&config, "08", false).unwrap();
        assert_eq!(count, 2);

        // Verify both were moved
        assert!(!config.tasks_directory.join("task-08-test.md").exists());
        assert!(!config.notes_directory.as_ref().unwrap().join("note-08-test.md").exists());

        assert!(config.archive_tasks_directory.as_ref().unwrap()
            .join("task-08-test.md").exists());
        assert!(config.archive_notes_directory.as_ref().unwrap()
            .join("note-08-test.md").exists());
    }

    #[test]
    fn test_archive_subtask() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        // Create subtask file
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-08.1-subtask.md"), "subtask").unwrap();

        // Archive
        let count = archive_task(&config, "08.1", false).unwrap();
        assert_eq!(count, 1);

        // Verify subtask was moved
        assert!(!config.tasks_directory.join("task-08.1-subtask.md").exists());
        assert!(config.archive_tasks_directory.as_ref().unwrap()
            .join("task-08.1-subtask.md").exists());
    }

    #[test]
    fn test_archive_with_children() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        // Create parent and children
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-08-parent.md"), "parent").unwrap();
        fs::write(config.tasks_directory.join("task-08.1-child1.md"), "child1").unwrap();
        fs::write(config.tasks_directory.join("task-08.2-child2.md"), "child2").unwrap();

        // Archive recursively
        let count = archive_task(&config, "08", true).unwrap();
        assert_eq!(count, 3);

        // Verify all were moved
        assert!(!config.tasks_directory.join("task-08-parent.md").exists());
        assert!(!config.tasks_directory.join("task-08.1-child1.md").exists());
        assert!(!config.tasks_directory.join("task-08.2-child2.md").exists());

        assert!(config.archive_tasks_directory.as_ref().unwrap()
            .join("task-08-parent.md").exists());
        assert!(config.archive_tasks_directory.as_ref().unwrap()
            .join("task-08.1-child1.md").exists());
        assert!(config.archive_tasks_directory.as_ref().unwrap()
            .join("task-08.2-child2.md").exists());
    }

    #[test]
    fn test_archive_with_grandchildren() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        // Create nested hierarchy
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-08-parent.md"), "parent").unwrap();
        fs::write(config.tasks_directory.join("task-08.1-child.md"), "child").unwrap();
        fs::write(config.tasks_directory.join("task-08.1.1-grandchild.md"), "grandchild").unwrap();

        // Archive recursively
        let count = archive_task(&config, "08", true).unwrap();
        assert_eq!(count, 3);

        // Verify all were moved
        assert!(!config.tasks_directory.join("task-08-parent.md").exists());
        assert!(!config.tasks_directory.join("task-08.1-child.md").exists());
        assert!(!config.tasks_directory.join("task-08.1.1-grandchild.md").exists());
    }

    #[test]
    fn test_archive_child_only_with_recursive() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        // Create parent and children
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-08-parent.md"), "parent").unwrap();
        fs::write(config.tasks_directory.join("task-08.1-child.md"), "child").unwrap();
        fs::write(config.tasks_directory.join("task-08.1.1-grandchild.md"), "grandchild").unwrap();
        fs::write(config.tasks_directory.join("task-08.2-child2.md"), "child2").unwrap();

        // Archive only 08.1 recursively (should get 08.1 and 08.1.1)
        let count = archive_task(&config, "08.1", true).unwrap();
        assert_eq!(count, 2);

        // Verify only 08.1 and 08.1.1 were moved
        assert!(config.tasks_directory.join("task-08-parent.md").exists());
        assert!(!config.tasks_directory.join("task-08.1-child.md").exists());
        assert!(!config.tasks_directory.join("task-08.1.1-grandchild.md").exists());
        assert!(config.tasks_directory.join("task-08.2-child2.md").exists());
    }

    #[test]
    fn test_archive_with_children_and_notes() {
        let temp_dir = TempDir::new().unwrap();
        let config = create_test_config(&temp_dir);

        // Create parent and children with notes
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::create_dir_all(config.notes_directory.as_ref().unwrap()).unwrap();

        fs::write(config.tasks_directory.join("task-08-parent.md"), "parent").unwrap();
        fs::write(config.tasks_directory.join("task-08.1-child.md"), "child").unwrap();

        fs::write(
            config.notes_directory.as_ref().unwrap().join("note-08-parent-notes.md"),
            "parent notes"
        ).unwrap();
        fs::write(
            config.notes_directory.as_ref().unwrap().join("note-08.1-child-notes.md"),
            "child notes"
        ).unwrap();

        // Archive recursively
        let count = archive_task(&config, "08", true).unwrap();
        assert_eq!(count, 4); // 2 tasks + 2 notes

        // Verify notes were also moved
        assert!(!config.notes_directory.as_ref().unwrap()
            .join("note-08-parent-notes.md").exists());
        assert!(!config.notes_directory.as_ref().unwrap()
            .join("note-08.1-child-notes.md").exists());
    }
}
