// Configuration parser
use crate::errors::{BertError, Result};
use crate::models::config::{BertConfig, SkillConfig};
use crate::project::{find_project_root, get_skill_yml_path};
use std::fs;
use std::path::{Path, PathBuf};

/// Default bert root relative to the project root, used in zero-config mode.
///
/// With this default the tasks directory resolves to `<repo_root>/docs/tasks`.
pub const DEFAULT_BERT_ROOT: &str = "docs";

/// Load bert configuration
///
/// Resolution order:
/// 1. Explicit project directory override (`repo_root`)
/// 2. Config discovered at the project root: `.bert/config.yml`, then legacy
///    `skills.yml`
/// 3. Zero-config defaults: `bert_root = docs`, i.e. tasks at
///    `<repo_root>/docs/tasks` (the repo root is found via git)
///
/// # Errors
///
/// Returns errors if a config file exists but cannot be read/parsed, or is
/// missing required fields. Absence of any config file is *not* an error.
pub fn load_config(repo_root: Option<PathBuf>, task_dir: Option<PathBuf>) -> Result<BertConfig> {
    let project_root = match repo_root {
        Some(path) => path.canonicalize()?,
        None => find_project_root()?,
    };
    let mut config = load_config_from_root(&project_root)?;

    // CLI flag wins over everything ("unless otherwise told")
    if let Some(task_dir) = task_dir {
        config.tasks_directory = task_dir;
    }

    Ok(config)
}

/// Load configuration from a specific project root
///
/// This is primarily used for testing.
pub fn load_config_from_root(project_root: &Path) -> Result<BertConfig> {
    let config = match get_skill_yml_path(project_root) {
        Some(config_path) => {
            let yaml_content = fs::read_to_string(&config_path).map_err(|e| {
                BertError::ConfigError(format!(
                    "Failed to read config at {}: {}",
                    config_path.display(),
                    e
                ))
            })?;

            let skill_config: SkillConfig = serde_yaml::from_str(&yaml_content)
                .map_err(|e| BertError::ConfigError(format!("Failed to parse config: {}", e)))?;

            // Required only when a config section is present
            if let Some(cfg) = &skill_config.config {
                if cfg.bert_root.is_empty() {
                    return Err(BertError::ConfigError(
                        "bert_root is required but was empty".to_string(),
                    ));
                }
            }

            skill_config
        }
        // No config file at all — pure zero-config defaults
        None => SkillConfig {
            config: None,
            format: None,
        },
    };

    // Convert to BertConfig with absolute paths
    Ok(BertConfig::from_skill_config(config, project_root))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_legacy_config(root: &Path, yaml: &str) {
        fs::write(root.join("skills.yml"), yaml).unwrap();
    }

    fn write_modern_config(root: &Path, yaml: &str) {
        fs::create_dir_all(root.join(".bert")).unwrap();
        fs::write(root.join(".bert/config.yml"), yaml).unwrap();
    }

    #[test]
    fn test_zero_config_defaults_to_docs_tasks() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        let config = load_config_from_root(root).unwrap();

        assert_eq!(config.bert_root, root.join("docs"));
        assert_eq!(
            config.tasks_directory,
            root.join("docs").join("tasks"),
            "tasks must default to <repo_root>/docs/tasks"
        );
        // Self-contained layout: all machinery nests inside docs/tasks,
        // claiming no other top-level docs/ names.
        assert_eq!(config.specs_directory, root.join("docs").join("tasks").join("specs"));
        assert_eq!(config.notes_directory, Some(root.join("docs").join("tasks").join("notes")));
        assert_eq!(
            config.archive_tasks_directory,
            Some(root.join("docs").join("tasks").join("archive"))
        );
        assert_eq!(
            config.archive_notes_directory,
            Some(root.join("docs").join("tasks").join("archive").join("notes"))
        );
        assert_eq!(
            config.library_directory,
            Some(root.join("docs").join("tasks").join("prompts").join("library"))
        );
    }

    #[test]
    fn test_legacy_config_section_keeps_bert_root_layout() {
        // An explicit config: section preserves the classic layout
        let temp_dir = TempDir::new().unwrap();
        write_legacy_config(temp_dir.path(), "config:\n  bert_root: docs/bert\n");

        let config = load_config_from_root(temp_dir.path()).unwrap();
        assert_eq!(
            config.tasks_directory,
            temp_dir.path().join("docs/bert/tasks")
        );
        assert_eq!(
            config.archive_tasks_directory,
            Some(temp_dir.path().join("docs/bert/archive/tasks"))
        );
        assert_eq!(
            config.notes_directory,
            Some(temp_dir.path().join("docs/bert/notes"))
        );
    }

    #[test]
    fn test_load_valid_legacy_config() {
        let temp_dir = TempDir::new().unwrap();
        write_legacy_config(
            temp_dir.path(),
            "config:\n  bert_root: docs/bert\n",
        );

        let config = load_config_from_root(temp_dir.path()).unwrap();
        assert_eq!(config.bert_root, temp_dir.path().join("docs/bert"));
        assert_eq!(
            config.tasks_directory,
            temp_dir.path().join("docs/bert/tasks")
        );
    }

    #[test]
    fn test_modern_config_preferred_and_explicit_tasks_directory() {
        let temp_dir = TempDir::new().unwrap();
        write_modern_config(
            temp_dir.path(),
            "config:\n  bert_root: docs\n  tasks_directory: docs/tasks\n",
        );
        // Legacy file would win if checked first; modern must take precedence
        write_legacy_config(temp_dir.path(), "config:\n  bert_root: other\n");

        let config = load_config_from_root(temp_dir.path()).unwrap();
        assert_eq!(config.bert_root, temp_dir.path().join("docs"));
        assert_eq!(
            config.tasks_directory,
            temp_dir.path().join("docs/tasks")
        );
    }

    #[test]
    fn test_format_only_config_loads_without_config_section() {
        // `bert task adopt` writes exactly this shape
        let temp_dir = TempDir::new().unwrap();
        write_modern_config(
            temp_dir.path(),
            "format:\n  slug: false\n  padding: 3\n  h1_lowercase: true\n  todo_status_word: open\n  frontmatter:\n  - id\n  - title\n  - status\n",
        );

        let config = load_config_from_root(temp_dir.path()).unwrap();
        // No `config:` section → default directory layout still applies
        assert_eq!(config.tasks_directory, temp_dir.path().join("docs").join("tasks"));

        let fmt = config.format.unwrap();
        assert_eq!(fmt.slug, Some(false));
        assert_eq!(fmt.padding, Some(3));
        assert_eq!(fmt.todo_status_word.as_deref(), Some("open"));
    }

    #[test]
    fn test_invalid_yaml_is_an_error_even_in_zero_config_world() {
        let temp_dir = TempDir::new().unwrap();
        write_modern_config(temp_dir.path(), "this is not valid yaml: [[[");
        assert!(load_config_from_root(temp_dir.path()).is_err());
    }

    #[test]
    fn test_missing_required_field_in_present_config() {
        let temp_dir = TempDir::new().unwrap();
        // DirectoryConfig requires bert_root; deserialization fails without it
        write_modern_config(temp_dir.path(), "config:\n  tasks_directory: docs/tasks\n");
        assert!(load_config_from_root(temp_dir.path()).is_err());

        write_modern_config(temp_dir.path(), "config:\n  bert_root: \"\"\n");
        assert!(load_config_from_root(temp_dir.path()).is_err());
    }

    #[test]
    fn test_full_override_config() {
        let temp_dir = TempDir::new().unwrap();
        write_legacy_config(
            temp_dir.path(),
            r#"
config:
  bert_root: docs/bert
  tasks_directory: docs/bert/tasks
  notes_directory: docs/bert/notes
  archive_tasks_directory: docs/bert/archive/tasks
  archive_notes_directory: docs/bert/archive/notes
  specs_directory: docs/bert/specs
  archive_specs_directory: docs/bert/archive/specs
  product_directory: docs/bert/product
"#,
        );

        let config = load_config_from_root(temp_dir.path()).unwrap();
        assert_eq!(config.project_root, temp_dir.path());
        assert_eq!(config.bert_root, temp_dir.path().join("docs/bert"));
        assert!(config.archive_tasks_directory.is_some());
        assert!(config.archive_specs_directory.is_some());
    }

    #[test]
    fn test_cli_tasks_dir_override_wins_over_everything() {
        let temp_dir = TempDir::new().unwrap();
        write_modern_config(
            temp_dir.path(),
            "config:\n  bert_root: docs\n  tasks_directory: docs/custom-tasks\n",
        );

        let override_dir = temp_dir.path().join("elsewhere/tasks");
        let config =
            load_config(None, Some(override_dir.clone())).unwrap();
        assert_eq!(config.tasks_directory, override_dir);
    }
}
