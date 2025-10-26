// Task stub command implementation
use crate::errors::{BertError, Result};
use crate::models::config::BertConfig;
use crate::numbering::find_next_number;
use crate::utils::normalize_task_number;
use chrono::Local;
use std::fs;

/// Create a new task stub file
///
/// # Arguments
///
/// * `config` - Bert configuration
/// * `description` - Task description
/// * `parent` - Optional parent task number (e.g., "03" or "03.1")
///
/// # Returns
///
/// Returns tuple of (task_number, file_path) on success
pub fn create_task_stub(
    config: &BertConfig,
    description: &str,
    parent: Option<&str>,
) -> Result<(String, String)> {
    if description.trim().is_empty() {
        return Err(BertError::InvalidInput("Description cannot be empty".to_string()));
    }

    // Determine task number based on parent
    let task_number = if let Some(parent_num) = parent {
        // Validate parent exists
        validate_parent_exists(config, parent_num)?;

        // Find next sibling number
        find_next_sibling(config, parent_num)?
    } else {
        // Get next top-level task number
        find_next_number(config)?
    };

    // Generate slug from description
    let slug = generate_slug(description);

    // Construct filename
    let filename = format!("task-{}-{}.md", task_number, slug);
    let filepath = config.tasks_directory.join(&filename);

    // Ensure tasks directory exists
    fs::create_dir_all(&config.tasks_directory)?;

    // Generate task content
    let content = generate_task_template(&task_number, description);

    // Write file
    fs::write(&filepath, content)?;

    Ok((task_number, filepath.display().to_string()))
}

/// Generate a URL-friendly slug from description
///
/// Rules:
/// - Convert to lowercase
/// - Replace spaces and underscores with hyphens
/// - Remove special characters (keep only alphanumeric and hyphens)
/// - Collapse multiple hyphens into one
/// - Trim hyphens from start and end
/// - Limit to 50 characters
fn generate_slug(description: &str) -> String {
    let mut slug = description.to_lowercase();

    // Replace spaces and underscores with hyphens
    slug = slug.replace(|c: char| c.is_whitespace() || c == '_', "-");

    // Keep only alphanumeric and hyphens
    slug = slug
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect();

    // Collapse multiple hyphens
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }

    // Trim hyphens from start and end
    slug = slug.trim_matches('-').to_string();

    // Limit to 50 characters
    if slug.len() > 50 {
        slug.truncate(50);
        slug = slug.trim_end_matches('-').to_string();
    }

    slug
}

/// Validate that parent task exists in tasks directory
fn validate_parent_exists(config: &BertConfig, parent: &str) -> Result<()> {
    let entries = fs::read_dir(&config.tasks_directory)?;
    // Normalize parent number (e.g., "1" -> "01")
    let normalized = normalize_task_number(parent);
    let pattern = format!("task-{}-", normalized);

    for entry in entries.flatten() {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.starts_with(&pattern) && filename.ends_with(".md") {
                return Ok(());
            }
        }
    }

    Err(BertError::ParentNotFound(parent.to_string()))
}

/// Find the next available sibling number for a parent task
///
/// Examples:
/// - Parent "03" with existing children 03.1, 03.2 -> returns "03.3"
/// - Parent "03.1" with existing children 03.1.1, 03.1.2 -> returns "03.1.3"
fn find_next_sibling(config: &BertConfig, parent: &str) -> Result<String> {
    let entries = fs::read_dir(&config.tasks_directory)?;
    // Normalize parent number (e.g., "1" -> "01")
    let normalized = normalize_task_number(parent);
    let pattern = format!("task-{}.", normalized);
    let mut max_sibling = 0;

    for entry in entries.flatten() {
        if let Some(filename) = entry.file_name().to_str() {
            if filename.starts_with(&pattern) {
                // Extract the sibling number after parent
                // e.g., "task-03.2-foo.md" -> extract "2"
                if let Some(rest) = filename.strip_prefix(&pattern) {
                    if let Some(num_part) = rest.split('-').next() {
                        // Handle nested tasks: "1.2" should extract "1"
                        if let Some(first_num) = num_part.split('.').next() {
                            if let Ok(num) = first_num.parse::<u32>() {
                                max_sibling = max_sibling.max(num);
                            }
                        }
                    }
                }
            }
        }
    }

    let next_sibling = max_sibling + 1;
    Ok(format!("{}.{}", normalized, next_sibling))
}

/// Generate task file content with frontmatter and minimal template
fn generate_task_template(task_number: &str, description: &str) -> String {
    let today = Local::now().format("%Y-%m-%d");

    format!(
        r#"---
status: pending
created: {date}
updated: {date}
---

# Task {number}: {title}

## Context

<!-- Describe what needs to be done and why -->
"#,
        date = today,
        number = task_number,
        title = description
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug_simple() {
        assert_eq!(generate_slug("Hello World"), "hello-world");
    }

    #[test]
    fn test_generate_slug_with_special_chars() {
        assert_eq!(
            generate_slug("Fix bug #123 (urgent!)"),
            "fix-bug-123-urgent"
        );
    }

    #[test]
    fn test_generate_slug_multiple_spaces() {
        assert_eq!(generate_slug("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_generate_slug_underscores() {
        assert_eq!(generate_slug("task_with_underscores"), "task-with-underscores");
    }

    #[test]
    fn test_generate_slug_max_length() {
        let long_desc = "This is a very long description that exceeds the fifty character limit";
        let slug = generate_slug(long_desc);
        assert!(slug.len() <= 50);
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn test_generate_slug_collapses_hyphens() {
        assert_eq!(generate_slug("hello----world"), "hello-world");
    }

    #[test]
    fn test_generate_slug_removes_leading_trailing_hyphens() {
        assert_eq!(generate_slug("-hello-world-"), "hello-world");
    }

    #[test]
    fn test_generate_template() {
        let content = generate_task_template("08", "Test Task");
        assert!(content.contains("status: pending"));
        assert!(content.contains("# Task 08: Test Task"));
        assert!(content.contains("## Context"));
    }

    #[test]
    fn test_find_next_sibling_first_child() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::BertConfig {
            project_root: temp_dir.path().to_path_buf(),
            tasks_directory: temp_dir.path().join("tasks"),
            notes_directory: None,
            archive_tasks_directory: None,
            archive_notes_directory: None,
            specs_directory: temp_dir.path().join("specs"),
            archive_specs_directory: None,
            product_directory: None,
        };

        // Create tasks directory and parent task
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03-parent.md"), "").unwrap();

        // First child should be 03.1
        let next = find_next_sibling(&config, "03").unwrap();
        assert_eq!(next, "03.1");
    }

    #[test]
    fn test_find_next_sibling_with_existing() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::BertConfig {
            project_root: temp_dir.path().to_path_buf(),
            tasks_directory: temp_dir.path().join("tasks"),
            notes_directory: None,
            archive_tasks_directory: None,
            archive_notes_directory: None,
            specs_directory: temp_dir.path().join("specs"),
            archive_specs_directory: None,
            product_directory: None,
        };

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03-parent.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03.1-child1.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03.2-child2.md"), "").unwrap();

        let next = find_next_sibling(&config, "03").unwrap();
        assert_eq!(next, "03.3");
    }

    #[test]
    fn test_find_next_sibling_nested() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::BertConfig {
            project_root: temp_dir.path().to_path_buf(),
            tasks_directory: temp_dir.path().join("tasks"),
            notes_directory: None,
            archive_tasks_directory: None,
            archive_notes_directory: None,
            specs_directory: temp_dir.path().join("specs"),
            archive_specs_directory: None,
            product_directory: None,
        };

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03.1-child.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03.1.1-grandchild.md"), "").unwrap();

        let next = find_next_sibling(&config, "03.1").unwrap();
        assert_eq!(next, "03.1.2");
    }

    #[test]
    fn test_validate_parent_exists_success() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::BertConfig {
            project_root: temp_dir.path().to_path_buf(),
            tasks_directory: temp_dir.path().join("tasks"),
            notes_directory: None,
            archive_tasks_directory: None,
            archive_notes_directory: None,
            specs_directory: temp_dir.path().join("specs"),
            archive_specs_directory: None,
            product_directory: None,
        };

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03-parent.md"), "").unwrap();

        assert!(validate_parent_exists(&config, "03").is_ok());
    }

    #[test]
    fn test_validate_parent_exists_failure() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::BertConfig {
            project_root: temp_dir.path().to_path_buf(),
            tasks_directory: temp_dir.path().join("tasks"),
            notes_directory: None,
            archive_tasks_directory: None,
            archive_notes_directory: None,
            specs_directory: temp_dir.path().join("specs"),
            archive_specs_directory: None,
            product_directory: None,
        };

        fs::create_dir_all(&config.tasks_directory).unwrap();

        let result = validate_parent_exists(&config, "03");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BertError::ParentNotFound(_)));
    }
}
