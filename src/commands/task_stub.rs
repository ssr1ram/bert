// Task stub command implementation
use crate::errors::{BertError, Result};
use crate::format::{self, FormatProfile};
use crate::models::config::BertConfig;
use crate::numbering::find_next_number;
use std::fs;

/// Create a new task stub file
///
/// The filename shape, number padding, frontmatter keys and status word mimic
/// the existing tasks directory (see [`format::detect_profile`]).
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

    let profile = format::apply_overrides(
        format::detect_profile(&config.tasks_directory),
        config.format.as_ref(),
    );

    // Determine task number based on parent
    let task_number = if let Some(parent_num) = parent {
        // One directory pass validates the parent (returning its canonical
        // on-disk form) and computes the next sibling slot under it
        find_next_sibling_under_parent(config, parent_num)?
    } else {
        // Get next top-level task number (width-aware)
        find_next_number(config)?
    };

    // Construct filename following the directory's convention
    let filename = if profile.use_slug {
        let slug = generate_task_slug(description);
        format!("task-{}-{}.md", task_number, slug)
    } else {
        format!("task-{}.md", task_number)
    };
    let filepath = config.tasks_directory.join(&filename);

    // Ensure tasks directory exists
    fs::create_dir_all(&config.tasks_directory)?;

    // Generate task content
    let content = generate_task_template(&profile, &task_number, description);

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
fn generate_task_slug(description: &str) -> String {
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

/// Find the next available sibling number under `parent`, validating that the
/// parent exists.
///
/// Returns the next number using the parent's on-disk spelling, so children
/// reuse the directory's own padding (e.g. parent file `task-033.md` → child
/// `033.1`).
///
/// Examples:
/// - Parent "03" with existing children 03.1, 03.2 -> returns "03.3"
/// - Parent "03.1" with existing children 03.1.1, 03.1.2 -> returns "03.1.3"
/// - Bare-format dirs work too: parent "033" with no children -> "033.1"
fn find_next_sibling_under_parent(config: &BertConfig, parent: &str) -> Result<String> {
    let mut canonical: Option<String> = None;
    let mut max_sibling = 0;

    for entry in fs::read_dir(&config.tasks_directory)?.flatten() {
        let Some(filename) = entry.file_name().into_string().ok() else {
            continue;
        };
        let Some(parsed) = format::parse_task_filename(&filename) else {
            continue;
        };

        // The parent may appear after its children in read_dir order, so keep
        // scanning for it even after children have been seen.
        if canonical.is_none() && format::number_matches(&parsed.number, parent) {
            canonical = Some(parsed.number.clone());
        }

        if format::number_is_descendant(&parsed.number, parent) {
            // The segment directly below the parent decides the sibling slot
            if let Some(next_seg) = parsed.number.split('.').nth(parent.split('.').count()) {
                if let Ok(num) = next_seg.parse::<u32>() {
                    max_sibling = max_sibling.max(num);
                }
            }
        }
    }

    let canonical_parent = canonical.ok_or_else(|| BertError::ParentNotFound(parent.to_string()))?;

    Ok(format!("{}.{}", canonical_parent, max_sibling + 1))
}

/// Generate task file content following the detected profile
fn generate_task_template(profile: &FormatProfile, task_number: &str, description: &str) -> String {
    let frontmatter = format::render_frontmatter(profile, task_number, description);

    let h1 = if profile.h1_lowercase {
        format!("# task-{}: {}", task_number, description)
    } else {
        let trimmed = task_number.trim_start_matches('0');
        let num = if trimmed.is_empty() { "0" } else { trimmed };
        format!("# Task {}: {}", num, description)
    };

    format!(
        r#"{frontmatter}{h1}

## Context

<!-- Describe what needs to be done and why -->
"#,
        frontmatter = frontmatter,
        h1 = h1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_slug_simple() {
        assert_eq!(generate_task_slug("Hello World"), "hello-world");
    }

    #[test]
    fn test_generate_slug_with_special_chars() {
        assert_eq!(
            generate_task_slug("Fix bug #123 (urgent!)"),
            "fix-bug-123-urgent"
        );
    }

    #[test]
    fn test_generate_slug_multiple_spaces() {
        assert_eq!(generate_task_slug("  Multiple   Spaces  "), "multiple-spaces");
    }

    #[test]
    fn test_generate_slug_underscores() {
        assert_eq!(generate_task_slug("task_with_underscores"), "task-with-underscores");
    }

    #[test]
    fn test_generate_slug_max_length() {
        let long_desc = "This is a very long description that exceeds the fifty character limit";
        let slug = generate_task_slug(long_desc);
        assert!(slug.len() <= 50);
        assert!(!slug.ends_with('-'));
    }

    #[test]
    fn test_generate_slug_collapses_hyphens() {
        assert_eq!(generate_task_slug("hello----world"), "hello-world");
    }

    #[test]
    fn test_generate_slug_removes_leading_trailing_hyphens() {
        assert_eq!(generate_task_slug("-hello-world-"), "hello-world");
    }

    #[test]
    fn test_generate_template_default_profile() {
        let profile = FormatProfile::default();
        let content = generate_task_template(&profile, "08", "Test Task");
        assert!(content.contains("status: stub"));
        assert!(content.contains("# Task 8: Test Task"));
        assert!(content.contains("## Context"));
        assert!(content.starts_with("---\n"));
    }

    #[test]
    fn test_generate_template_wobase_style_profile() {
        let profile = FormatProfile {
            use_slug: false,
            h1_lowercase: true,
            todo_status_word: "open".to_string(),
            frontmatter_keys: vec![
                ("id".to_string(), format::FieldKind::Scalar),
                ("title".to_string(), format::FieldKind::Scalar),
                ("status".to_string(), format::FieldKind::Scalar),
                ("priority".to_string(), format::FieldKind::Scalar),
                ("tags".to_string(), format::FieldKind::List),
            ],
            ..Default::default()
        };
        let content = generate_task_template(&profile, "034", "Wire it up");
        assert!(content.contains("id: task-034"));
        assert!(content.contains("title: \"Wire it up\""));
        assert!(content.contains("status: open"));
        assert!(content.contains("priority: \"\""));
        assert!(content.contains("tags: []"));
        assert!(content.contains("# task-034: Wire it up"));
    }

    #[test]
    fn test_find_next_sibling_first_child() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::test_support::test_config(temp_dir.path());

        // Create tasks directory and parent task
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03-parent.md"), "").unwrap();

        // First child should be 03.1
        let next = find_next_sibling_under_parent(&config, "03").unwrap();
        assert_eq!(next, "03.1");
    }

    #[test]
    fn test_find_next_sibling_with_existing() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::test_support::test_config(temp_dir.path());

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03-parent.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03.1-child1.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03.2-child2.md"), "").unwrap();

        let next = find_next_sibling_under_parent(&config, "03").unwrap();
        assert_eq!(next, "03.3");
    }

    #[test]
    fn test_find_next_sibling_nested() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::test_support::test_config(temp_dir.path());

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03.1-child.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03.1.1-grandchild.md"), "").unwrap();

        let next = find_next_sibling_under_parent(&config, "03.1").unwrap();
        assert_eq!(next, "03.1.2");
    }

    #[test]
    fn test_validate_parent_exists_success() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::test_support::test_config(temp_dir.path());

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03-parent.md"), "").unwrap();

        assert!(find_next_sibling_under_parent(&config, "03").is_ok());
    }

    #[test]
    fn test_validate_parent_exists_failure() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::test_support::test_config(temp_dir.path());

        fs::create_dir_all(&config.tasks_directory).unwrap();

        let result = find_next_sibling_under_parent(&config, "03");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BertError::ParentNotFound(_)));
    }
}
