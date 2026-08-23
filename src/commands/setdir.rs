use crate::errors::{BertError, Result};
use std::fs;
use std::path::Path;

/// Create a default skills.yml and directory structure in the target directory
pub fn create_default_config(bert_dir: &Path) -> Result<()> {
    // 1. Ensure bert_dir exists (create_dir_all is idempotent)
    fs::create_dir_all(bert_dir).map_err(|e| {
        BertError::ConfigError(format!("Failed to create directory {}: {}", bert_dir.display(), e))
    })?;

    let skills_yml_path = bert_dir.join(crate::project::LEGACY_SKILLS_YML);
    
    // 2. Check if skills.yml already exists
    if skills_yml_path.exists() {
        return Err(BertError::ConfigError(format!(
            "skills.yml already exists at {}",
            skills_yml_path.display()
        )));
    }

    // 3. Write default skills.yml
    let default_yaml = r#"config:
  bert_root: docs/bert
"#;
    fs::write(&skills_yml_path, default_yaml).map_err(|e| {
        BertError::ConfigError(format!("Failed to write skills.yml: {}", e))
    })?;

    // 4. Create default directory structure
    let bert_root = bert_dir.join("docs/bert");
    let subdirs = ["tasks", "specs", "prompts/library", "prompts/sets", "notes"];
    
    for subdir in &subdirs {
        let path = bert_root.join(subdir);
        fs::create_dir_all(&path).map_err(|e| {
            BertError::ConfigError(format!("Failed to create directory {}: {}", path.display(), e))
        })?;
    }

    Ok(())
}
