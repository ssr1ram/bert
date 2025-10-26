// Configuration data models
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Main skill.yml configuration structure
#[derive(Debug, Deserialize, Clone)]
pub struct SkillConfig {
    /// Configuration section
    pub config: DirectoryConfig,
}

/// Directory configuration from skill.yml
#[derive(Debug, Deserialize, Clone)]
pub struct DirectoryConfig {
    /// Tasks directory (required)
    pub tasks_directory: String,

    /// Notes directory (optional)
    #[serde(default)]
    pub notes_directory: Option<String>,

    /// Archive tasks directory (optional)
    #[serde(default)]
    pub archive_tasks_directory: Option<String>,

    /// Archive notes directory (optional)
    #[serde(default)]
    pub archive_notes_directory: Option<String>,

    /// Specs directory (required)
    pub specs_directory: String,

    /// Archive specs directory (optional)
    #[serde(default)]
    pub archive_specs_directory: Option<String>,

    /// Product context directory (optional)
    #[serde(default)]
    pub product_directory: Option<String>,
}

/// Resolved configuration with absolute paths
#[derive(Debug, Clone)]
pub struct BertConfig {
    /// Project root directory
    #[allow(dead_code)]
    pub project_root: PathBuf,

    /// Tasks directory (absolute path)
    pub tasks_directory: PathBuf,

    /// Notes directory (absolute path, optional)
    pub notes_directory: Option<PathBuf>,

    /// Archive tasks directory (absolute path, optional)
    pub archive_tasks_directory: Option<PathBuf>,

    /// Archive notes directory (absolute path, optional)
    pub archive_notes_directory: Option<PathBuf>,

    /// Specs directory (absolute path)
    pub specs_directory: PathBuf,

    /// Archive specs directory (absolute path, optional)
    pub archive_specs_directory: Option<PathBuf>,

    /// Product context directory (absolute path, optional)
    #[allow(dead_code)]
    pub product_directory: Option<PathBuf>,
}

impl BertConfig {
    /// Create BertConfig from SkillConfig by resolving relative paths
    pub fn from_skill_config(skill_config: SkillConfig, project_root: &Path) -> Self {
        let config = skill_config.config;

        BertConfig {
            project_root: project_root.to_path_buf(),
            tasks_directory: project_root.join(&config.tasks_directory),
            notes_directory: config.notes_directory
                .map(|p| project_root.join(p)),
            archive_tasks_directory: config.archive_tasks_directory
                .map(|p| project_root.join(p)),
            archive_notes_directory: config.archive_notes_directory
                .map(|p| project_root.join(p)),
            specs_directory: project_root.join(&config.specs_directory),
            archive_specs_directory: config.archive_specs_directory
                .map(|p| project_root.join(p)),
            product_directory: config.product_directory
                .map(|p| project_root.join(p)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_minimal_config() {
        let yaml = r#"
config:
  tasks_directory: docs/bert/tasks
  specs_directory: docs/bert/specs
"#;

        let skill_config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(skill_config.config.tasks_directory, "docs/bert/tasks");
        assert_eq!(skill_config.config.specs_directory, "docs/bert/specs");
        assert!(skill_config.config.notes_directory.is_none());
    }

    #[test]
    fn test_deserialize_full_config() {
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

        let skill_config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(skill_config.config.tasks_directory, "docs/bert/tasks");
        assert_eq!(skill_config.config.notes_directory, Some("docs/bert/notes".to_string()));
        assert_eq!(skill_config.config.specs_directory, "docs/bert/specs");
    }

    #[test]
    fn test_resolve_paths() {
        let yaml = r#"
config:
  tasks_directory: docs/bert/tasks
  specs_directory: docs/bert/specs
  notes_directory: docs/bert/notes
"#;

        let skill_config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        let project_root = PathBuf::from("/project/root");
        let bert_config = BertConfig::from_skill_config(skill_config, &project_root);

        assert_eq!(bert_config.project_root, PathBuf::from("/project/root"));
        assert_eq!(bert_config.tasks_directory, PathBuf::from("/project/root/docs/bert/tasks"));
        assert_eq!(bert_config.specs_directory, PathBuf::from("/project/root/docs/bert/specs"));
        assert_eq!(bert_config.notes_directory, Some(PathBuf::from("/project/root/docs/bert/notes")));
    }
}
