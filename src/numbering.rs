// Universal numbering system
use crate::errors::Result;
use crate::models::config::BertConfig;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Find the next available task/spec number
///
/// Scans both tasks and specs (active + archived) to find the highest
/// existing number, then returns the next number with 2-digit padding.
///
/// This implements universal numbering to prevent collisions between
/// task-01 and spec-01.
///
/// # Examples
///
/// ```no_run
/// use bert_cli::numbering::find_next_number;
/// use bert_cli::config::load_config;
///
/// let config = load_config(None, None)?;
/// let next = find_next_number(&config)?;
/// println!("Next number: {}", next); // e.g., "09"
/// # Ok::<(), bert_cli::errors::BertError>(())
/// ```
pub fn find_next_number(config: &BertConfig) -> Result<String> {
    let mut max_num = 0;
    // Mimic the directory's number width (e.g. bare 3-digit task-033.md files).
    let mut max_width = 2usize;

    // Spec pattern matches both "spec-08" and "spec-08-slug"
    let spec_pattern = Regex::new(r"^spec-(\d+)").unwrap();

    // Scan active tasks
    if let Some((max, width)) = scan_tasks_directory(&config.tasks_directory) {
        max_num = max_num.max(max);
        max_width = max_width.max(width);
    }

    // Scan archived tasks
    if let Some(ref archive_dir) = config.archive_tasks_directory {
        if let Some((max, width)) = scan_tasks_directory(archive_dir) {
            max_num = max_num.max(max);
            max_width = max_width.max(width);
        }
    }

    // Scan active specs
    if let Some(max) = scan_specs_directory(&config.specs_directory, &spec_pattern) {
        max_num = max_num.max(max);
    }

    // Scan archived specs
    if let Some(ref archive_dir) = config.archive_specs_directory {
        if let Some(max) = scan_specs_directory(archive_dir, &spec_pattern) {
            max_num = max_num.max(max);
        }
    }

    // Next number is max + 1, padded to the directory's observed width
    let next_num = max_num + 1;
    Ok(format!("{:0w$}", next_num, w = max_width))
}

/// Scan a tasks directory for task files of any convention
///
/// Returns `(highest_number, widest_digit_width)`, or None if the directory
/// doesn't exist or no valid tasks are found.
fn scan_tasks_directory(dir: &Path) -> Option<(u32, usize)> {
    let entries = fs::read_dir(dir).ok()?;
    let mut max_num = 0;
    let mut max_width = 2;
    let mut found_any = false;

    for entry in entries.flatten() {
        if let Some(filename) = entry.file_name().to_str() {
            if let Some(parsed) = crate::format::parse_task_filename(filename) {
                if let Ok(num) = parsed.number.parse::<u32>() {
                    max_num = max_num.max(num);
                    if !parsed.number.contains('.') {
                        max_width = max_width.max(parsed.number.len());
                    }
                    found_any = true;
                }
            }
        }
    }

    found_any.then_some((max_num, max_width))
}

/// Scan a specs directory for spec directories matching the pattern
///
/// Returns the highest spec number found, or None if directory doesn't exist
/// or no valid specs are found.
fn scan_specs_directory(dir: &Path, pattern: &Regex) -> Option<u32> {
    let entries = fs::read_dir(dir).ok()?;
    let mut max_num = 0;
    let mut found_any = false;

    for entry in entries.flatten() {
        if entry.path().is_dir() {
            if let Some(dirname) = entry.file_name().to_str() {
                if let Some(captures) = pattern.captures(dirname) {
                    if let Some(num_str) = captures.get(1) {
                        if let Ok(num) = num_str.as_str().parse::<u32>() {
                            max_num = max_num.max(num);
                            found_any = true;
                        }
                    }
                }
            }
        }
    }

    if found_any {
        Some(max_num)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::test_support::test_config;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_next_number_empty() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create empty directories
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::create_dir_all(&config.specs_directory).unwrap();

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "01");
    }

    #[test]
    fn test_find_next_number_with_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create tasks directory with some tasks
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-01-foo.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03-bar.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-05-baz.md"), "").unwrap();

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "06");
    }

    #[test]
    fn test_find_next_number_ignores_subtasks() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create tasks with subtasks
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-03-parent.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03.1-child.md"), "").unwrap();
        fs::write(config.tasks_directory.join("task-03.2-child.md"), "").unwrap();

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "04"); // Should only count task-03, not subtasks
    }

    #[test]
    fn test_find_next_number_with_specs() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create specs directories
        fs::create_dir_all(&config.specs_directory).unwrap();
        fs::create_dir_all(config.specs_directory.join("spec-02")).unwrap();
        fs::create_dir_all(config.specs_directory.join("spec-07")).unwrap();

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "08");
    }

    #[test]
    fn test_find_next_number_with_tasks_and_specs() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create both tasks and specs
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::create_dir_all(&config.specs_directory).unwrap();

        fs::write(config.tasks_directory.join("task-05-foo.md"), "").unwrap();
        fs::create_dir_all(config.specs_directory.join("spec-08")).unwrap();

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "09"); // Max is spec-08, so next is 09
    }

    #[test]
    fn test_find_next_number_with_archived() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create active and archived tasks/specs
        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::create_dir_all(config.archive_tasks_directory.as_ref().unwrap()).unwrap();
        fs::create_dir_all(&config.specs_directory).unwrap();
        fs::create_dir_all(config.archive_specs_directory.as_ref().unwrap()).unwrap();

        fs::write(config.tasks_directory.join("task-03-active.md"), "").unwrap();
        fs::write(
            config.archive_tasks_directory.as_ref().unwrap().join("task-10-archived.md"),
            "",
        )
        .unwrap();
        fs::create_dir_all(config.specs_directory.join("spec-05")).unwrap();
        fs::create_dir_all(
            config.archive_specs_directory.as_ref().unwrap().join("spec-12")
        ).unwrap();

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "13"); // Max is archived spec-12
    }

    #[test]
    fn test_find_next_number_nonexistent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Don't create any directories
        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "01");
    }

    #[test]
    fn test_find_next_number_with_padding() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        fs::create_dir_all(&config.tasks_directory).unwrap();
        fs::write(config.tasks_directory.join("task-01-foo.md"), "").unwrap();

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "02"); // Should be padded to 2 digits
    }

    #[test]
    fn test_find_next_number_with_spec_slugs() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Create specs with slugs (new format)
        fs::create_dir_all(&config.specs_directory).unwrap();
        fs::create_dir_all(config.specs_directory.join("spec-05-api")).unwrap();
        fs::create_dir_all(config.specs_directory.join("spec-08-auth")).unwrap();
        fs::create_dir_all(config.specs_directory.join("spec-12-dms")).unwrap();

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "13"); // Max is spec-12-dms
    }

    #[test]
    fn test_find_next_number_with_mixed_spec_formats() {
        let temp_dir = TempDir::new().unwrap();
        let config = test_config(temp_dir.path());

        // Mix of old format (spec-08) and new format (spec-12-slug)
        fs::create_dir_all(&config.specs_directory).unwrap();
        fs::create_dir_all(config.specs_directory.join("spec-08")).unwrap(); // old
        fs::create_dir_all(config.specs_directory.join("spec-12-dms")).unwrap(); // new

        let next = find_next_number(&config).unwrap();
        assert_eq!(next, "13"); // Max is spec-12-dms
    }
}
