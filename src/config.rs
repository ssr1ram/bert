// Configuration parser
use crate::errors::{BertError, Result};
use crate::models::config::{BertConfig, SkillConfig};
use crate::project::{find_project_root, get_skill_yml_path};
use std::fs;
use std::path::Path;

/// Load bert configuration from skill.yml
///
/// This function:
/// 1. Finds the project root
/// 2. Reads the skill.yml file
/// 3. Parses the YAML
/// 4. Resolves all relative paths to absolute paths
///
/// # Errors
///
/// Returns errors if:
/// - Project root cannot be found
/// - skill.yml cannot be read
/// - YAML is invalid
/// - Required fields are missing
///
/// # Example
///
/// ```no_run
/// use bert_cli::config::load_config;
///
/// let config = load_config()?;
/// println!("Tasks directory: {}", config.tasks_directory.display());
/// # Ok::<(), bert_cli::errors::BertError>(())
/// ```
pub fn load_config() -> Result<BertConfig> {
    let project_root = find_project_root()?;
    load_config_from_root(&project_root)
}

/// Load configuration from a specific project root
///
/// This is primarily used for testing.
pub fn load_config_from_root(project_root: &Path) -> Result<BertConfig> {
    let skill_yml_path = get_skill_yml_path(project_root);

    // Read the skill.yml file
    let yaml_content = fs::read_to_string(&skill_yml_path)
        .map_err(|e| BertError::ConfigError(
            format!("Failed to read skill.yml at {}: {}", skill_yml_path.display(), e)
        ))?;

    // Parse YAML
    let skill_config: SkillConfig = serde_yaml::from_str(&yaml_content)
        .map_err(|e| BertError::ConfigError(
            format!("Failed to parse skill.yml: {}", e)
        ))?;

    // Validate required fields
    if skill_config.config.tasks_directory.is_empty() {
        return Err(BertError::ConfigError(
            "tasks_directory is required but was empty".to_string()
        ));
    }

    if skill_config.config.specs_directory.is_empty() {
        return Err(BertError::ConfigError(
            "specs_directory is required but was empty".to_string()
        ));
    }

    // Convert to BertConfig with absolute paths
    Ok(BertConfig::from_skill_config(skill_config, project_root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_project(yaml_content: &str) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create skill.yml
        let skill_dir = root.join(".claude/skills/bert");
        fs::create_dir_all(&skill_dir).unwrap();
        fs::write(skill_dir.join("skill.yml"), yaml_content).unwrap();

        temp_dir
    }

    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
config:
  tasks_directory: docs/bert/tasks
  specs_directory: docs/bert/specs
  notes_directory: docs/bert/notes
"#;
        let temp_dir = create_test_project(yaml);
        let config = load_config_from_root(temp_dir.path()).unwrap();

        assert_eq!(
            config.tasks_directory,
            temp_dir.path().join("docs/bert/tasks")
        );
        assert_eq!(
            config.specs_directory,
            temp_dir.path().join("docs/bert/specs")
        );
        assert_eq!(
            config.notes_directory,
            Some(temp_dir.path().join("docs/bert/notes"))
        );
    }

    #[test]
    fn test_load_config_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create directory structure but no skill.yml
        let skill_dir = root.join(".claude/skills/bert");
        fs::create_dir_all(&skill_dir).unwrap();

        let result = load_config_from_root(root);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BertError::ConfigError(_)));
    }

    #[test]
    fn test_load_config_invalid_yaml() {
        let yaml = "this is not valid yaml: [[[";
        let temp_dir = create_test_project(yaml);

        let result = load_config_from_root(temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BertError::ConfigError(_)));
    }

    #[test]
    fn test_load_config_missing_required_field() {
        let yaml = r#"
config:
  tasks_directory: docs/bert/tasks
  # specs_directory is missing!
"#;
        let temp_dir = create_test_project(yaml);

        let result = load_config_from_root(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_empty_required_field() {
        let yaml = r#"
config:
  tasks_directory: ""
  specs_directory: docs/bert/specs
"#;
        let temp_dir = create_test_project(yaml);

        let result = load_config_from_root(temp_dir.path());
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BertError::ConfigError(_)));
    }

    #[test]
    fn test_load_config_full() {
        let yaml = r#"
config:
  tasks_directory: docs/bert/tasks
  notes_directory: docs/bert/notes
  archive_tasks_directory: docs/bert/archive/tasks
  archive_notes_directory: docs/bert/archive/notes
  specs_directory: docs/bert/specs
  archive_specs_directory: docs/bert/archive/specs
  product_directory: docs/bert/product
"#;
        let temp_dir = create_test_project(yaml);
        let config = load_config_from_root(temp_dir.path()).unwrap();

        assert_eq!(config.project_root, temp_dir.path());
        assert!(config.archive_tasks_directory.is_some());
        assert!(config.archive_specs_directory.is_some());
        assert!(config.product_directory.is_some());
    }
}
