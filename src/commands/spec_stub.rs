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

    // Construct directory path
    let dirname = format!("spec-{}", spec_number);
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

    // Create visuals subdirectory
    fs::create_dir_all(dirpath.join("visuals"))?;

    // Generate requirements.md content
    let requirements_content = generate_requirements_template(&spec_number, description);
    fs::write(dirpath.join("requirements.md"), requirements_content)?;

    // Generate spec.md content
    let spec_content = generate_spec_template(&spec_number, description);
    fs::write(dirpath.join("spec.md"), spec_content)?;

    Ok((spec_number.clone(), dirpath.display().to_string()))
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

/// Generate spec.md template
fn generate_spec_template(spec_number: &str, description: &str) -> String {
    let today = Local::now().format("%Y-%m-%d");

    format!(
        r#"---
status: draft
created: {date}
updated: {date}
iteration: 1
spec_number: {number}
---

# Spec {number}: {title}

**Requirements**: [Spec {number} Requirements](./requirements.md)

## Goal

<!-- High-level goal of this specification -->

## Overview

<!-- Brief summary of the approach -->

## Design

<!-- Detailed design section -->

### Architecture

<!-- System architecture and components -->

### Implementation

<!-- Implementation details -->

## Technical Considerations

<!-- Important technical notes -->

## Testing Strategy

<!-- How to test this implementation -->

## Migration Plan

<!-- If applicable, how to migrate from current state -->

## Open Issues

<!-- Known issues or areas needing further work -->
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
    fn test_generate_spec_template() {
        let content = generate_spec_template("08", "Test Spec");
        assert!(content.contains("status: draft"));
        assert!(content.contains("spec_number: 08"));
        assert!(content.contains("# Spec 08: Test Spec"));
        assert!(content.contains("## Goal"));
        assert!(content.contains("## Design"));
    }
}
