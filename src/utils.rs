// Utility functions for bert CLI

/// Normalize a task number to match file naming conventions
///
/// This pads single-digit top-level numbers to 2 digits and handles subtasks.
///
/// # Examples
///
/// - "1" -> "01"
/// - "01" -> "01"
/// - "8" -> "08"
/// - "1.2" -> "01.2"
/// - "01.2" -> "01.2"
/// - "3.1.4" -> "03.1.4"
///
/// # Arguments
///
/// * `task_number` - The task number to normalize
///
/// # Returns
///
/// Normalized task number with padded top-level digit
pub fn normalize_task_number(task_number: &str) -> String {
    if task_number.contains('.') {
        // Subtask: only pad the first part before the first dot
        let parts: Vec<&str> = task_number.split('.').collect();
        if let Some(first) = parts.first() {
            if let Ok(num) = first.parse::<u32>() {
                let mut result = format!("{:02}", num);
                // Append the rest of the parts
                for part in &parts[1..] {
                    result.push('.');
                    result.push_str(part);
                }
                return result;
            }
        }
    } else {
        // Top-level task: pad to 2 digits
        if let Ok(num) = task_number.parse::<u32>() {
            return format!("{:02}", num);
        }
    }

    // If parsing fails, return as-is
    task_number.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_single_digit() {
        assert_eq!(normalize_task_number("1"), "01");
        assert_eq!(normalize_task_number("8"), "08");
        assert_eq!(normalize_task_number("9"), "09");
    }

    #[test]
    fn test_normalize_already_padded() {
        assert_eq!(normalize_task_number("01"), "01");
        assert_eq!(normalize_task_number("08"), "08");
        assert_eq!(normalize_task_number("10"), "10");
    }

    #[test]
    fn test_normalize_subtask() {
        assert_eq!(normalize_task_number("1.2"), "01.2");
        assert_eq!(normalize_task_number("01.2"), "01.2");
        assert_eq!(normalize_task_number("8.1"), "08.1");
    }

    #[test]
    fn test_normalize_nested_subtask() {
        assert_eq!(normalize_task_number("1.2.3"), "01.2.3");
        assert_eq!(normalize_task_number("8.1.4"), "08.1.4");
        assert_eq!(normalize_task_number("01.2.3"), "01.2.3");
    }

    #[test]
    fn test_normalize_double_digit() {
        assert_eq!(normalize_task_number("10"), "10");
        assert_eq!(normalize_task_number("15"), "15");
        assert_eq!(normalize_task_number("10.1"), "10.1");
    }
}
