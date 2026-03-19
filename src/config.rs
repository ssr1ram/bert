// Configuration parser
use crate::errors::{BertError, Result};
use crate::models::config::{BertConfig, SkillConfig};
use crate::project::{find_project_root, get_skill_yml_path};
use std::fs;
use std::path::{Path, PathBuf};

/// Load bert configuration from skill.yml
///
/// This function:
/// 1. Finds the project root (or uses the provided override)
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
/// use std::path::PathBuf;
///
/// let config = load_config(None)?;
/// println!("Tasks directory: {}", config.tasks_directory.display());
/// # Ok::<(), bert_cli::errors::BertError>(())
/// ```
pub fn load_config(bert_dir: Option<PathBuf>) -> Result<BertConfig> {
    let project_root = match bert_dir {
        Some(path) => path.canonicalize()?,
        None => find_project_root()?,
    };
    load_config_from_root(&project_root)
}

/// Load configuration from a specific project root
///
/// This is primarily used for testing.
pub fn load_config_from_root(project_root: &Path) -> Result<BertConfig> {
    let skill_yml_path = get_skill_yml_path(project_root);

    // Read the skills.yml file
    let yaml_content = fs::read_to_string(&skill_yml_path)
        .map_err(|e| BertError::ConfigError(
            format!("Failed to read skills.yml at {}: {}", skill_yml_path.display(), e)
        ))?;

    // Parse YAML
    let skill_config: SkillConfig = serde_yaml::from_str(&yaml_content)
        .map_err(|e| BertError::ConfigError(
            format!("Failed to parse skills.yml: {}", e)
        ))?;

    // Validate required field: bert_root
    if skill_config.config.bert_root.is_empty() {
        return Err(BertError::ConfigError(
            "bert_root is required but was empty".to_string()
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

        // Create skills.yml
        fs::write(root.join("skills.yml"), yaml_content).unwrap();

        temp_dir
    }

    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
config:
  bert_root: docs/bert
"#;
        let temp_dir = create_test_project(yaml);
        let config = load_config_from_root(temp_dir.path()).unwrap();

        // Test that default paths are used
        assert_eq!(
            config.bert_root,
            temp_dir.path().join("docs/bert")
        );
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
        assert_eq!(
            config.library_directory,
            Some(temp_dir.path().join("docs/bert/prompts/library"))
        );
    }

    #[test]
    fn test_load_config_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create directory but no skills.yml
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
  # bert_root is missing!
"#;
        let temp_dir = create_test_project(yaml);

        let result = load_config_from_root(temp_dir.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_config_empty_required_field() {
        let yaml = r#"
config:
  bert_root: ""
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
  bert_root: docs/bert
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
        assert_eq!(config.bert_root, temp_dir.path().join("docs/bert"));
        assert!(config.archive_tasks_directory.is_some());
        assert!(config.archive_specs_directory.is_some());
        assert!(config.product_directory.is_some());
    }

    #[test]
    fn test_load_config_with_override() {
        let yaml = r#"
config:
  bert_root: docs/bert
"#;
        let temp_dir = create_test_project(yaml);
        let override_path = temp_dir.path().to_path_buf();

        // Create the expected bert_root directory so canonicalize() works
        fs::create_dir_all(override_path.join("docs/bert")).unwrap();

        // Should use the provided override path
        let config = load_config(Some(override_path.clone())).unwrap();

        assert_eq!(config.project_root.canonicalize().unwrap(), override_path.canonicalize().unwrap());
        assert_eq!(config.bert_root.canonicalize().unwrap(), override_path.join("docs/bert").canonicalize().unwrap());
    }
}
