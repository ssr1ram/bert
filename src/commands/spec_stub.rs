// Spec stub command implementation
use crate::errors::{BertError, Result};
use crate::models::config::BertConfig;
use crate::numbering::find_next_number;
use chrono::Local;
use std::fs;

/// Create a new spec stub directory with minimal template files
///
/// # Arguments
///
/// * `config` - Bert configuration
/// * `description` - Spec description
///
/// # Returns
///
/// Returns tuple of (spec_number, directory_path) on success
pub fn create_spec_stub(config: &BertConfig, description: &str) -> Result<(String, String)> {
    if description.trim().is_empty() {
        return Err(BertError::InvalidInput(
            "Description cannot be empty".to_string(),
        ));
    }

    // Get next spec number using universal numbering
    let spec_number = find_next_number(config)?;

    // Generate short slug from description (5-6 chars max)
    let slug = generate_short_slug(description);

    // Construct directory path with slug
    let dirname = format!("spec-{}-{}", spec_number, slug);
    let dirpath = config.specs_directory.join(&dirname);

    // Ensure specs directory exists
    fs::create_dir_all(&config.specs_directory)?;

    // Check if spec directory already exists
    if dirpath.exists() {
        return Err(BertError::AlreadyExists(format!(
            "Spec directory already exists: {}",
            dirpath.display()
        )));
    }

    // Create spec directory
    fs::create_dir_all(&dirpath)?;

    // Generate requirements.md content (stub only contains requirements)
    let requirements_content = generate_requirements_template(&spec_number, description);
    fs::write(dirpath.join("requirements.md"), requirements_content)?;

    Ok((spec_number, dirpath.display().to_string()))
}

/// Generate a short slug (5-6 chars max) from description
///
/// Strategy:
/// - Split description into words
/// - Take first letter of each significant word (skip common words like "the", "a", "an")
/// - If too short, take more letters from first word
/// - Lowercase and alphanumeric only
/// - Max 6 characters
fn generate_short_slug(description: &str) -> String {
    // Common words to skip
    let skip_words = ["the", "a", "an", "and", "or", "for", "to", "of", "in", "on"];

    // Get words and filter
    let lowercase = description.to_lowercase();
    let words: Vec<&str> = lowercase
        .split_whitespace()
        .filter(|w| !skip_words.contains(w))
        .collect();

    if words.is_empty() {
        return "spec".to_string();
    }

    // Strategy 1: Try acronym from first letters
    let acronym: String = words
        .iter()
        .take(6)
        .filter_map(|w| {
            w.chars()
                .find(|c| c.is_alphanumeric())
                .map(|c| c.to_ascii_lowercase())
        })
        .collect();

    // If acronym is good length (3-6 chars), use it
    if acronym.len() >= 3 && acronym.len() <= 6 {
        return acronym;
    }

    // Strategy 2: Take first word, remove vowels if needed, truncate
    let first_word: String = words[0]
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect();

    if first_word.len() <= 6 {
        return first_word;
    }

    // Remove vowels to shorten
    let no_vowels: String = first_word
        .chars()
        .filter(|c| !matches!(c, 'a' | 'e' | 'i' | 'o' | 'u'))
        .take(6)
        .collect();

    if !no_vowels.is_empty() && no_vowels.len() <= 6 {
        return no_vowels;
    }

    // Fallback: just truncate first word
    first_word.chars().take(6).collect()
}

/// Generate requirements.md template
fn generate_requirements_template(spec_number: &str, description: &str) -> String {
    let today = Local::now().format("%Y-%m-%d");

    format!(
        r#"---
status: draft
created: {date}
updated: {date}
spec_number: {number}
---

# Requirements: {title}

## Problem Statement

<!-- Describe the problem this spec aims to solve -->

## Goals

<!-- What are we trying to achieve? -->

## Non-Goals

<!-- What is explicitly out of scope? -->

## User Stories

<!-- Describe user scenarios and use cases -->

## Requirements

### Functional Requirements

<!-- What the system must do -->

### Non-Functional Requirements

<!-- Performance, security, usability, etc. -->

## Open Questions

<!-- Unresolved issues that need discussion -->
"#,
        date = today,
        number = spec_number,
        title = description
    )
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_requirements_template() {
        let content = generate_requirements_template("08", "Test Spec");
        assert!(content.contains("status: draft"));
        assert!(content.contains("spec_number: 08"));
        assert!(content.contains("# Requirements: Test Spec"));
        assert!(content.contains("## Problem Statement"));
    }

    #[test]
    fn test_generate_short_slug_acronym() {
        // Multi-word: should create acronym
        assert_eq!(generate_short_slug("Rust CLI Tool"), "rct");
        assert_eq!(generate_short_slug("User Authentication System"), "uas");
    }

    #[test]
    fn test_generate_short_slug_short_word() {
        // Short single word
        assert_eq!(generate_short_slug("auth"), "auth");
        assert_eq!(generate_short_slug("api"), "api");
    }

    #[test]
    fn test_generate_short_slug_long_word() {
        // Long word: remove vowels, take up to 6 chars
        assert_eq!(generate_short_slug("authentication"), "thntct");
    }

    #[test]
    fn test_generate_short_slug_with_stop_words() {
        // Should skip "the", "and" -> ["user", "admin"]
        // Acronym "ua" is only 2 chars (< 3), so falls back to first word
        assert_eq!(generate_short_slug("the user and admin"), "user");
    }

    #[test]
    fn test_generate_short_slug_max_length() {
        // Should be max 6 chars
        let slug = generate_short_slug("one two three four five six seven");
        assert!(slug.len() <= 6);
        assert_eq!(slug, "ottffs"); // o-t-t-f-f-s from first 6 words
    }

    #[test]
    fn test_generate_short_slug_special_chars() {
        // Should handle special characters
        let slug = generate_short_slug("API v2.0 (beta)");
        assert!(slug.chars().all(|c| c.is_alphanumeric()));
        assert_eq!(slug, "avb");
    }

    #[test]
    fn test_generate_short_slug_empty() {
        // Empty or only stop words
        assert_eq!(generate_short_slug("the a an"), "spec");
    }
}
