// Prompt stub command implementation
use crate::errors::{BertError, Result};
use crate::models::config::BertConfig;
use chrono::Local;
use regex::Regex;
use std::fs;

/// Create a new prompt log stub file
///
/// # Arguments
///
/// * `config` - Bert configuration
/// * `description` - Brief description of the prompt log
/// * `verbose` - Enable verbose logging
///
/// # Returns
///
/// Returns tuple of (prompt_number, file_path, template_used) on success
pub fn create_prompt_stub(config: &BertConfig, description: &str, verbose: bool) -> Result<(String, String, Option<String>)> {
    if description.trim().is_empty() {
        return Err(BertError::InvalidInput(
            "Description cannot be empty".to_string(),
        ));
    }

    // Check if prompt_logs directory is configured
    let prompt_logs_dir = config.prompt_logs.as_ref()
        .ok_or_else(|| BertError::ConfigError(
            "prompt_logs directory not configured in skill.yml".to_string()
        ))?;

    // Ensure prompt logs directory exists
    fs::create_dir_all(prompt_logs_dir)?;

    // Get next prompt number (count up from 001)
    let prompt_number = find_next_prompt_number(prompt_logs_dir)?;

    // Generate short slug from description
    let slug = generate_prompt_slug(description);

    // Get today's date
    let today = Local::now().format("%Y-%m-%d");

    // Construct filename: 2025-10-28-001-{slug}.md
    let filename = format!("{}-{}-{}.md", today, prompt_number, slug);
    let filepath = prompt_logs_dir.join(&filename);

    // Check if file already exists
    if filepath.exists() {
        return Err(BertError::AlreadyExists(format!(
            "Prompt log file already exists: {}",
            filepath.display()
        )));
    }

    // Generate file content
    let (content, template_used) = generate_prompt_log_template(config, &prompt_number, description, verbose)?;
    fs::write(&filepath, content)?;

    Ok((prompt_number, filepath.display().to_string(), template_used))
}

/// Find the next available prompt number for today
///
/// Scans the prompt logs directory for files matching today's date pattern
/// and returns the next number in sequence (001, 002, 003, ...)
fn find_next_prompt_number(prompt_logs_dir: &std::path::Path) -> Result<String> {
    let today = Local::now().format("%Y-%m-%d");

    // Pattern: YYYY-MM-DD-NNN-*.md
    let pattern = Regex::new(&format!(r"^{}-(\d{{3}})-.*\.md$", regex::escape(&today.to_string())))
        .expect("Failed to compile regex pattern");

    let mut max_num = 0;

    // A missing logs directory is not an error (the first prompt starts at
    // 001); other read failures still propagate.
    let entries = match fs::read_dir(prompt_logs_dir) {
        Ok(entries) => Some(entries),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => return Err(e.into()),
    };

    if let Some(entries) = entries {
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                if let Some(captures) = pattern.captures(filename) {
                    if let Some(num_str) = captures.get(1) {
                        if let Ok(num) = num_str.as_str().parse::<u32>() {
                            max_num = max_num.max(num);
                        }
                    }
                }
            }
        }
    }

    // Return next number with 3-digit padding
    Ok(format!("{:03}", max_num + 1))
}

/// Generate a slug from description (lowercase, alphanumeric, max 20 chars)
fn generate_prompt_slug(description: &str) -> String {
    let slug: String = description
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Remove consecutive dashes
    let slug = Regex::new(r"-+").unwrap().replace_all(&slug, "-");

    // Trim dashes from start/end
    let slug = slug.trim_matches('-');

    // Truncate to max 20 chars
    slug.chars().take(20).collect()
}

/// Generate prompt log template
fn generate_prompt_log_template(config: &BertConfig, prompt_number: &str, description: &str, verbose: bool) -> Result<(String, Option<String>)> {
    let today = Local::now().format("%Y-%m-%d");

    // Determine template path based on bert_root
    let template_path = config.bert_root.join("config/templates/prompt-stub-frontmatter.yml");

    if verbose {
        eprintln!("[verbose] Checking for custom template at: {}", template_path.display());
    }

    let (frontmatter, template_used) = if template_path.exists() {
        if verbose {
            eprintln!("[verbose] Custom template found, reading...");
        }

        // Read and use custom template
        let template = fs::read_to_string(&template_path)?;

        if verbose {
            eprintln!("[verbose] Template loaded, replacing placeholders:");
            eprintln!("[verbose]   {{number}} -> {}", prompt_number);
            eprintln!("[verbose]   {{date}} -> {}", today);
            eprintln!("[verbose]   {{description}} -> {}", description);
        }

        // Replace placeholders
        let frontmatter = template
            .replace("{number}", prompt_number)
            .replace("{date}", &today.to_string())
            .replace("{description}", description);

        (frontmatter, Some(template_path.display().to_string()))
    } else {
        if verbose {
            eprintln!("[verbose] Custom template not found, using default template");
        }

        // Use default template
        let frontmatter = format!(
            r#"---
prompt_number: {number}
created: {date}
status: active
---"#,
            number = prompt_number,
            date = today
        );

        (frontmatter, None)
    };

    let content = format!(
        r#"{frontmatter}

# Prompt Log: {title}

## Context

"#,
        frontmatter = frontmatter,
        title = description
    );

    Ok((content, template_used))
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_generate_slug() {
        assert_eq!(generate_prompt_slug("System Creation"), "system-creation");
        assert_eq!(generate_prompt_slug("API v2.0 Design"), "api-v2-0-design");
        assert_eq!(generate_prompt_slug("Test  Multiple   Spaces"), "test-multiple-spaces");

        // Test truncation
        let long_desc = "This is a very long description that exceeds the maximum length";
        let slug = generate_prompt_slug(long_desc);
        assert!(slug.len() <= 20);
    }

    #[test]
    fn test_generate_prompt_log_template() {
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::test_support::test_config(temp_dir.path());

        let (content, template_used) = generate_prompt_log_template(&config, "001", "Test Prompt", false).unwrap();
        assert!(content.contains("prompt_number: 001"));
        assert!(content.contains("# Prompt Log: Test Prompt"));
        assert!(content.contains("## Context"));
        assert!(content.ends_with("## Context\n\n"));
        assert!(template_used.is_none()); // No bert_root set in this config
    }

    #[test]
    fn test_find_next_prompt_number_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_next_prompt_number(temp_dir.path()).unwrap();
        assert_eq!(result, "001");
    }

    #[test]
    fn test_find_next_prompt_number_with_existing() {
        let temp_dir = TempDir::new().unwrap();
        let today = Local::now().format("%Y-%m-%d");

        // Create some existing prompt log files
        fs::write(temp_dir.path().join(format!("{}-001-first.md", today)), "").unwrap();
        fs::write(temp_dir.path().join(format!("{}-003-third.md", today)), "").unwrap();

        let result = find_next_prompt_number(temp_dir.path()).unwrap();
        assert_eq!(result, "004");
    }

    #[test]
    fn test_create_prompt_stub() {
        let temp_dir = TempDir::new().unwrap();
        let config = crate::models::config::test_support::test_config(temp_dir.path());

        let (number, filepath, template_used) = create_prompt_stub(&config, "Test prompt", false).unwrap();

        assert_eq!(number, "001");
        assert!(PathBuf::from(&filepath).exists());
        assert!(template_used.is_none());

        let content = fs::read_to_string(&filepath).unwrap();
        assert!(content.contains("prompt_number: 001"));
        assert!(content.contains("# Prompt Log: Test prompt"));
    }

    #[test]
    fn test_create_prompt_stub_no_config() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = crate::models::config::test_support::test_config(temp_dir.path());
        config.prompt_logs = None;

        let result = create_prompt_stub(&config, "Test prompt", false);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BertError::ConfigError(_)));
    }

    #[test]
    fn test_generate_prompt_log_template_with_custom_template() {
        let temp_dir = TempDir::new().unwrap();
        let bert_root = temp_dir.path().join("bert");
        let config = crate::models::config::test_support::test_config(temp_dir.path());

        // Create custom template in bert_root
        let template_dir = bert_root.join("config/templates");
        fs::create_dir_all(&template_dir).unwrap();
        let custom_template = r#"---
prompt_number: {number}
created: {date}
description: {description}
status: draft
custom_field: test_value
---"#;
        fs::write(template_dir.join("prompt-stub-frontmatter.yml"), custom_template).unwrap();

        let (content, template_used) = generate_prompt_log_template(&config, "042", "Custom Test", false).unwrap();
        assert!(content.contains("prompt_number: 042"));
        assert!(content.contains("description: Custom Test"));
        assert!(content.contains("status: draft"));
        assert!(content.contains("custom_field: test_value"));
        assert!(content.contains("# Prompt Log: Custom Test"));
        assert!(template_used.is_some());
        assert!(template_used.unwrap().contains("config/templates/prompt-stub-frontmatter.yml"));
    }
}
