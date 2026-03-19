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
    /// Bert root directory (required - defines the base for all BERT directories)
    /// When only bert_root is specified, all other paths use the default layout:
    /// {bert_root}/tasks, {bert_root}/specs, {bert_root}/prompts/library, etc.
    pub bert_root: String,

    /// Tasks directory (optional - defaults to {bert_root}/tasks)
    #[serde(default)]
    pub tasks_directory: Option<String>,

    /// Notes directory (optional)
    #[serde(default)]
    pub notes_directory: Option<String>,

    /// Archive tasks directory (optional)
    #[serde(default)]
    pub archive_tasks_directory: Option<String>,

    /// Archive notes directory (optional)
    #[serde(default)]
    pub archive_notes_directory: Option<String>,

    /// Specs directory (optional - defaults to {bert_root}/specs)
    #[serde(default)]
    pub specs_directory: Option<String>,

    /// Archive specs directory (optional)
    #[serde(default)]
    pub archive_specs_directory: Option<String>,

    /// Archive root directory (optional) - parent of archive/{tasks,specs,notes}
    #[serde(default)]
    pub archive_directory: Option<String>,

    /// Product context directory (optional)
    #[serde(default)]
    pub product_directory: Option<String>,

    /// Prompt logs directory (optional)
    #[serde(default)]
    pub prompt_logs: Option<String>,

    /// Prompt library directory (optional)
    #[serde(default)]
    pub library_directory: Option<String>,

    /// Prompt sets directory (optional)
    #[serde(default)]
    pub sets_directory: Option<String>,

    /// TUI configuration (optional)
    #[serde(default)]
    pub tui: Option<TuiConfig>,
}

/// TUI configuration
#[derive(Debug, Deserialize, Clone)]
pub struct TuiConfig {
    /// Pane width configuration
    #[serde(default)]
    pub pane_widths: Option<PaneWidths>,
}

/// Pane width percentages
#[derive(Debug, Deserialize, Clone)]
pub struct PaneWidths {
    /// Explorer pane width (left)
    #[serde(default = "default_explorer_width")]
    pub explorer: u16,

    /// Buffer/queue pane width (middle)
    #[serde(default = "default_buffer_width")]
    pub buffer: u16,

    /// Preview pane width (right)
    #[serde(default = "default_preview_width")]
    pub preview: u16,
}

fn default_explorer_width() -> u16 { 30 }
fn default_buffer_width() -> u16 { 30 }
fn default_preview_width() -> u16 { 40 }

impl Default for PaneWidths {
    fn default() -> Self {
        Self {
            explorer: 30,
            buffer: 30,
            preview: 40,
        }
    }
}

/// Resolved configuration with absolute paths
#[derive(Debug, Clone)]
pub struct BertConfig {
    /// Project root directory
    #[allow(dead_code)]
    pub project_root: PathBuf,

    /// Bert root directory (absolute path)
    pub bert_root: PathBuf,

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

    /// Archive root directory (absolute path, optional)
    pub archive_directory: Option<PathBuf>,

    /// Product context directory (absolute path, optional)
    #[allow(dead_code)]
    pub product_directory: Option<PathBuf>,

    /// Prompt logs directory (absolute path, optional)
    pub prompt_logs: Option<PathBuf>,

    /// Prompt library directory (absolute path, optional)
    pub library_directory: Option<PathBuf>,

    /// Prompt sets directory (absolute path, optional)
    #[allow(dead_code)]
    pub sets_directory: Option<PathBuf>,
    /// TUI configuration
    pub tui: TuiConfig,
}

impl BertConfig {
    /// Create BertConfig from SkillConfig by resolving relative paths
    ///
    /// If bert_root is specified but individual directories are not, the default
    /// directory layout is used (defined in models::defaults::BertDirectoryLayout).
    pub fn from_skill_config(skill_config: SkillConfig, project_root: &Path) -> Self {
        use super::defaults::BertDirectoryLayout;

        let config = skill_config.config;

        // Resolve bert_root to absolute path
        let bert_root_abs = project_root.join(&config.bert_root);

        BertConfig {
            project_root: project_root.to_path_buf(),
            bert_root: bert_root_abs.clone(),

            // Use provided path or default to {bert_root}/tasks
            tasks_directory: config.tasks_directory
                .map(|p| project_root.join(p))
                .unwrap_or_else(|| project_root.join(BertDirectoryLayout::tasks_dir(&PathBuf::from(&config.bert_root)))),

            // Use provided path or default to {bert_root}/notes
            notes_directory: config.notes_directory
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::notes_dir(&PathBuf::from(&config.bert_root))))),

            // Use provided path or default to {bert_root}/archive/tasks
            archive_tasks_directory: config.archive_tasks_directory
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::archive_tasks_dir(&PathBuf::from(&config.bert_root))))),

            // Use provided path or default to {bert_root}/archive/notes
            archive_notes_directory: config.archive_notes_directory
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::archive_notes_dir(&PathBuf::from(&config.bert_root))))),

            // Use provided path or default to {bert_root}/specs
            specs_directory: config.specs_directory
                .map(|p| project_root.join(p))
                .unwrap_or_else(|| project_root.join(BertDirectoryLayout::specs_dir(&PathBuf::from(&config.bert_root)))),

            // Use provided path or default to {bert_root}/archive/specs
            archive_specs_directory: config.archive_specs_directory
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::archive_specs_dir(&PathBuf::from(&config.bert_root))))),

            // Use provided path or default to {bert_root}/archive
            archive_directory: config.archive_directory
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::archive_dir(&PathBuf::from(&config.bert_root))))),

            // Use provided path or default to {bert_root}/product
            product_directory: config.product_directory
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::product_dir(&PathBuf::from(&config.bert_root))))),

            // Use provided path or default to {bert_root}/prompts/logs
            prompt_logs: config.prompt_logs
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::prompt_logs_dir(&PathBuf::from(&config.bert_root))))),

            // Use provided path or default to {bert_root}/prompts/library
            library_directory: config.library_directory
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::prompts_library_dir(&PathBuf::from(&config.bert_root))))),

            // Use provided path or default to {bert_root}/prompts/sets
            sets_directory: config.sets_directory
                .map(|p| project_root.join(p))
                .or_else(|| Some(project_root.join(BertDirectoryLayout::prompts_sets_dir(&PathBuf::from(&config.bert_root))))),

            tui: config.tui.unwrap_or_else(|| TuiConfig {
                pane_widths: Some(PaneWidths::default()),
            }),
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
  bert_root: docs/bert
"#;

        let skill_config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(skill_config.config.bert_root, "docs/bert");
        assert!(skill_config.config.tasks_directory.is_none());
        assert!(skill_config.config.specs_directory.is_none());
        assert!(skill_config.config.notes_directory.is_none());
    }

    #[test]
    fn test_deserialize_full_config() {
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

        let skill_config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(skill_config.config.bert_root, "docs/bert");
        assert_eq!(skill_config.config.tasks_directory, Some("docs/bert/tasks".to_string()));
        assert_eq!(skill_config.config.notes_directory, Some("docs/bert/notes".to_string()));
        assert_eq!(skill_config.config.specs_directory, Some("docs/bert/specs".to_string()));
    }

    #[test]
    fn test_resolve_paths() {
        let yaml = r#"
config:
  bert_root: docs/bert
"#;

        let skill_config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        let project_root = PathBuf::from("/project/root");
        let bert_config = BertConfig::from_skill_config(skill_config, &project_root);

        assert_eq!(bert_config.project_root, PathBuf::from("/project/root"));
        assert_eq!(bert_config.bert_root, PathBuf::from("/project/root/docs/bert"));
        assert_eq!(bert_config.tasks_directory, PathBuf::from("/project/root/docs/bert/tasks"));
        assert_eq!(bert_config.specs_directory, PathBuf::from("/project/root/docs/bert/specs"));
        assert_eq!(bert_config.notes_directory, Some(PathBuf::from("/project/root/docs/bert/notes")));
        assert_eq!(bert_config.library_directory, Some(PathBuf::from("/project/root/docs/bert/prompts/library")));
        assert_eq!(bert_config.sets_directory, Some(PathBuf::from("/project/root/docs/bert/prompts/sets")));
    }
}
