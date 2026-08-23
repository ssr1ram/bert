// Spec archive command implementation
use super::task_archive::{find_notes_file, find_task_file};
use crate::errors::{BertError, Result};
use crate::format;
use crate::models::config::BertConfig;
use crate::utils::normalize_task_number;
use std::fs;
use std::path::PathBuf;

/// Archive a spec directory and all associated tasks
///
/// # Arguments
///
/// * `config` - Bert configuration
/// * `spec_number` - Spec number to archive (e.g., "08" or "8")
///
/// # Returns
///
/// Returns count of items archived (spec directory + tasks)
pub fn archive_spec(config: &BertConfig, spec_number: &str) -> Result<usize> {
    // Normalize spec number (e.g., "8" -> "08")
    let normalized = normalize_task_number(spec_number);

    // Find and archive the spec directory
    let spec_dir = find_spec_dir(config, &normalized)?;
    let archived_spec = archive_spec_directory(config, &spec_dir)?;

    // Find and archive all related tasks
    let archived_tasks = archive_related_tasks(config, &normalized)?;

    Ok(archived_spec + archived_tasks)
}

/// Find spec directory by spec number
///
/// Handles both old format (spec-08) and new format (spec-08-slug)
fn find_spec_dir(config: &BertConfig, spec_number: &str) -> Result<PathBuf> {
    let pattern = format!("spec-{}", spec_number);
    let entries = fs::read_dir(&config.specs_directory)?;

    let slug_pattern = format!("{}-", pattern);

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(dirname) = path.file_name().and_then(|n| n.to_str()) {
            // Match both "spec-08" and "spec-08-slug"
            if (dirname == pattern || dirname.starts_with(&slug_pattern)) && path.is_dir() {
                return Ok(path);
            }
        }
    }

    Err(BertError::NotFound(format!(
        "Spec directory not found for spec number: {}",
        spec_number
    )))
}

/// Archive the spec directory
fn archive_spec_directory(config: &BertConfig, spec_dir: &PathBuf) -> Result<usize> {
    let archive_specs_dir = config
        .archive_specs_directory
        .as_ref()
        .ok_or_else(|| {
            BertError::ConfigError("archive_specs_directory not configured".to_string())
        })?;

    // Ensure archive directory exists
    fs::create_dir_all(archive_specs_dir)?;

    // Get the directory name
    let dirname = spec_dir
        .file_name()
        .ok_or_else(|| BertError::FileError("Invalid spec directory name".to_string()))?;

    let archive_spec_path = archive_specs_dir.join(dirname);

    // Check if archive destination already exists
    if archive_spec_path.exists() {
        return Err(BertError::AlreadyExists(format!(
            "Archived spec already exists: {}",
            archive_spec_path.display()
        )));
    }

    // Move the entire directory
    fs::rename(spec_dir, &archive_spec_path)?;

    Ok(1)
}

/// Archive all tasks related to this spec number
///
/// This includes:
/// - The parent task (e.g., task-08-...)
/// - All child tasks (e.g., task-08.1-..., task-08.2-..., etc.)
/// - All nested children (e.g., task-08.1.1-...)
fn archive_related_tasks(config: &BertConfig, spec_number: &str) -> Result<usize> {
    let mut total_archived = 0;

    // Find the parent task (if it exists)
    if let Ok(parent_task) = find_task_file(config, spec_number) {
        total_archived += archive_single_task(config, &parent_task)?;
    }

    // Find and archive all child tasks
    let children = find_all_task_children(config, spec_number)?;

    for child_task_path in children {
        total_archived += archive_single_task(config, &child_task_path)?;
    }

    Ok(total_archived)
}

/// Find all child tasks of a spec
///
/// Matches any descendant depth (`task-08.1`, `task-08.1.1`, ...), including
/// bare-format files like `task-08.1.md`.
fn find_all_task_children(config: &BertConfig, parent_number: &str) -> Result<Vec<PathBuf>> {
    let mut children: Vec<PathBuf> = fs::read_dir(&config.tasks_directory)?
        .filter_map(|e| e.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .and_then(format::parse_task_filename)
                .map(|parsed| format::number_is_descendant(&parsed.number, parent_number))
                .unwrap_or(false)
        })
        .collect();

    children.sort();

    Ok(children)
}

/// Archive a single task file and its associated notes
fn archive_single_task(config: &BertConfig, task_file: &PathBuf) -> Result<usize> {
    let archive_tasks_dir = config
        .archive_tasks_directory
        .as_ref()
        .ok_or_else(|| {
            BertError::ConfigError("archive_tasks_directory not configured".to_string())
        })?;

    fs::create_dir_all(archive_tasks_dir)?;

    // Archive task file
    let task_filename = task_file
        .file_name()
        .ok_or_else(|| BertError::FileError("Invalid task filename".to_string()))?;
    let archive_task_path = archive_tasks_dir.join(task_filename);
    fs::rename(task_file, &archive_task_path)?;

    let mut archived_count = 1;

    // Try to archive associated notes file
    // Extract task number from filename (e.g., "task-08.1-foo.md" -> "08.1")
    if let Some(filename_str) = task_filename.to_str() {
        if let Some(task_num) = format::parse_task_filename(filename_str).map(|p| p.number) {
            if let Some(notes_path) = find_notes_file(config, &task_num) {
                if let Some(archive_notes_dir) = config.archive_notes_directory.as_ref() {
                    fs::create_dir_all(archive_notes_dir)?;

                    let notes_filename = notes_path.file_name().ok_or_else(|| {
                        BertError::FileError("Invalid notes filename".to_string())
                    })?;
                    let archive_notes_path = archive_notes_dir.join(notes_filename);

                    fs::rename(&notes_path, &archive_notes_path)?;
                    archived_count += 1;
                }
            }
        }
    }

    Ok(archived_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::test_support::test_config;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_spec_dir_success() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        fs::create_dir_all(config.specs_directory.join("spec-08")).unwrap();

        let found = find_spec_dir(&config, "08").unwrap();
        assert!(found.to_str().unwrap().contains("spec-08"));
    }

    #[test]
    fn test_find_spec_dir_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        fs::create_dir_all(&config.specs_directory).unwrap();

        let result = find_spec_dir(&config, "08");
        assert!(result.is_err());
    }

    #[test]
    fn test_archive_spec_directory() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create spec directory with files
        let spec_dir = config.specs_directory.join("spec-08");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("spec.md"), "content").unwrap();
        fs::write(spec_dir.join("requirements.md"), "content").unwrap();

        let count = archive_spec_directory(&config, &spec_dir).unwrap();
        assert_eq!(count, 1);

        // Verify spec was moved
        assert!(!spec_dir.exists());
        assert!(config
            .archive_specs_directory
            .as_ref()
            .unwrap()
            .join("spec-08")
            .exists());
        assert!(config
            .archive_specs_directory
            .as_ref()
            .unwrap()
            .join("spec-08/spec.md")
            .exists());
    }

    #[test]
    fn test_archive_spec_with_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create spec directory
        let spec_dir = config.specs_directory.join("spec-08");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("spec.md"), "content").unwrap();

        // Create related tasks
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(
            config.tasks_directory.join("task-08-parent.md"),
            "parent",
        )
        .unwrap();
        fs::write(config.tasks_directory.join("task-08.1-child.md"), "child").unwrap();
        fs::write(
            config.tasks_directory.join("task-08.2-child2.md"),
            "child2",
        )
        .unwrap();

        let count = archive_spec(&config, "08").unwrap();
        assert_eq!(count, 4); // 1 spec dir + 3 tasks

        // Verify spec was moved
        assert!(!spec_dir.exists());
        assert!(config
            .archive_specs_directory
            .as_ref()
            .unwrap()
            .join("spec-08")
            .exists());

        // Verify tasks were moved
        assert!(!config.tasks_directory.join("task-08-parent.md").exists());
        assert!(!config.tasks_directory.join("task-08.1-child.md").exists());
        assert!(!config.tasks_directory.join("task-08.2-child2.md").exists());
    }

    #[test]
    fn test_archive_spec_with_nested_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create spec directory
        let spec_dir = config.specs_directory.join("spec-08");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("spec.md"), "content").unwrap();

        // Create nested task hierarchy
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(
            config.tasks_directory.join("task-08-parent.md"),
            "parent",
        )
        .unwrap();
        fs::write(config.tasks_directory.join("task-08.1-child.md"), "child").unwrap();
        fs::write(
            config
                .tasks_directory
                .join("task-08.1.1-grandchild.md"),
            "grandchild",
        )
        .unwrap();

        let count = archive_spec(&config, "08").unwrap();
        assert_eq!(count, 4); // 1 spec + 3 tasks

        // Verify all tasks were moved
        assert!(!config.tasks_directory.join("task-08-parent.md").exists());
        assert!(!config.tasks_directory.join("task-08.1-child.md").exists());
        assert!(!config
            .tasks_directory
            .join("task-08.1.1-grandchild.md")
            .exists());
    }

    #[test]
    fn test_archive_spec_only() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create spec directory without any tasks
        let spec_dir = config.specs_directory.join("spec-08");
        fs::create_dir_all(&spec_dir).unwrap();
        fs::write(spec_dir.join("spec.md"), "content").unwrap();

        // Create tasks directory but no matching tasks
        fs::create_dir_all(&config.tasks_directory).unwrap();

        let count = archive_spec(&config, "08").unwrap();
        assert_eq!(count, 1); // Just the spec directory

        assert!(!spec_dir.exists());
        assert!(config
            .archive_specs_directory
            .as_ref()
            .unwrap()
            .join("spec-08")
            .exists());
    }

    #[test]
    fn test_archive_related_tasks_bare_format() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::create_dir_all(config.archive_tasks_directory.as_ref().unwrap()).unwrap();

        // Bare-format parent and child, slugged grandchild
        fs::write(config.tasks_directory.join("task-13.md"), "p").unwrap();
        fs::write(config.tasks_directory.join("task-13.1.md"), "c").unwrap();
        fs::write(config.tasks_directory.join("task-13.1.1-x.md"), "g").unwrap();

        let n = super::archive_related_tasks(&config, "13").unwrap();
        assert_eq!(n, 3, "parent + child + grandchild all archived");
        assert!(!config.tasks_directory.join("task-13.1.md").exists());
    }

    #[test]
    fn test_archive_related_tasks_cross_padding_parent() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::create_dir_all(config.archive_tasks_directory.as_ref().unwrap()).unwrap();

        // Requesting spec "8" must find the zero-padded parent and its children
        fs::write(config.tasks_directory.join("task-08-parent.md"), "p").unwrap();
        fs::write(config.tasks_directory.join("task-08.1-child.md"), "c").unwrap();

        let n = super::archive_related_tasks(&config, "8").unwrap();
        assert_eq!(n, 2);
    }
}
