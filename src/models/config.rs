// Configuration data models
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Per-file format conventions, typically written by `bert task adopt` as a
/// top-level `format:` section. Every field is optional; anything absent
/// keeps the detected or bert-native default.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct FileFormatConfig {
    /// Emit `task-NN-slug.md` (true) or bare `task-NNN.md` (false)
    #[serde(default)]
    pub slug: Option<bool>,

    /// Zero-padding width for task numbers (minimum 2 on apply)
    #[serde(default)]
    pub padding: Option<usize>,

    /// H1 style: `# task-01-slug` (true) vs `# Task 01: Slug` (false)
    #[serde(default)]
    pub h1_lowercase: Option<bool>,

    /// The directory's word for a fresh task's status
    #[serde(default)]
    pub todo_status_word: Option<String>,

    /// Frontmatter keys to emit on new stubs, in order
    #[serde(default)]
    pub frontmatter: Option<Vec<String>>,
}

/// Main skill.yml / `.bert/config.yml` structure.
///
/// Both sections are optional: a file with only a `format:` section is valid,
/// and a missing `config:` section selects the self-contained default layout.
#[derive(Debug, Deserialize, Clone)]
pub struct SkillConfig {
    /// Configuration section (directory overrides)
    #[serde(default)]
    pub config: Option<DirectoryConfig>,

    /// File format conventions
    #[serde(default)]
    pub format: Option<FileFormatConfig>,
}

/// Directory configuration from skill.yml
#[derive(Debug, Deserialize, Clone)]
pub struct DirectoryConfig {
    /// Bert root directory (required when a config section is present - defines
    /// the base for all BERT directories). When only bert_root is specified,
    /// all other paths use the default layout:
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

    /// File format conventions (explicit config beats directory mimicry)
    pub format: Option<FileFormatConfig>,
}

/// If `tasks_dir/done` already exists on disk, prefer it as the archive
/// destination for completed tasks. This mimics an established `done/`
/// convention the same way bert already mimics filename/frontmatter shape
/// from an existing directory, so a project that already archives into
/// `done/` doesn't need a `.bert/config.yml` just to keep `bert task done`
/// (and `archive`) from creating a second, competing `archive/` folder.
fn existing_done_dir(tasks_dir: &Path) -> Option<PathBuf> {
    let done_dir = tasks_dir.join("done");
    done_dir.is_dir().then_some(done_dir)
}

impl BertConfig {
    /// Create BertConfig from SkillConfig by resolving relative paths
    ///
    /// With a `config:` section, relative paths resolve against the project
    /// root and unspecified directories use the classic layout under
    /// `bert_root` (defined in models::defaults::BertDirectoryLayout).
    ///
    /// Without one, everything nests self-contained under `<root>/docs/tasks`.
    pub fn from_skill_config(skill_config: SkillConfig, project_root: &Path) -> Self {
        use super::defaults::BertDirectoryLayout;
        use crate::config::DEFAULT_BERT_ROOT;

        // Self-contained zero-config layout: everything lives inside the
        // tasks directory itself, keeping the docs/ namespace clean.
        let Some(config) = skill_config.config else {
            let docs = project_root.join(DEFAULT_BERT_ROOT);
            let tasks = docs.join("tasks");
            let archive_tasks_directory = existing_done_dir(&tasks).unwrap_or_else(|| tasks.join("archive"));
            return BertConfig {
                project_root: project_root.to_path_buf(),
                bert_root: docs,
                tasks_directory: tasks.clone(),
                notes_directory: Some(tasks.join("notes")),
                archive_tasks_directory: Some(archive_tasks_directory),
                archive_notes_directory: Some(tasks.join("archive/notes")),
                archive_specs_directory: Some(tasks.join("archive/specs")),
                specs_directory: tasks.join("specs"),
                archive_directory: Some(tasks.join("archive")),
                product_directory: Some(tasks.join("product")),
                prompt_logs: Some(tasks.join("prompts/logs")),
                library_directory: Some(tasks.join("prompts/library")),
                sets_directory: Some(tasks.join("prompts/sets")),
                tui: TuiConfig {
                    pane_widths: Some(PaneWidths::default()),
                },
                format: skill_config.format,
            };
        };

        // Resolve bert_root to absolute path
        let bert_root_abs = project_root.join(&config.bert_root);
        let bert_root_rel = PathBuf::from(&config.bert_root);

        // Resolve an optional override against the project root, falling back
        // to the layout default relative to bert_root.
        let resolve_opt = |override_path: &Option<String>, layout_default: fn(&Path) -> PathBuf| {
            Some(
                override_path
                    .as_deref()
                    .map(|p| project_root.join(p))
                    .unwrap_or_else(|| project_root.join(layout_default(&bert_root_rel))),
            )
        };
        // Same as `resolve_opt` for required (non-Option) paths.
        let resolve_req = |override_path: Option<&String>, layout_default: fn(&Path) -> PathBuf| {
            override_path
                .map(|p| project_root.join(p))
                .unwrap_or_else(|| project_root.join(layout_default(&bert_root_rel)))
        };

        let tasks_directory = resolve_req(
            config.tasks_directory.as_ref(),
            BertDirectoryLayout::tasks_dir,
        );

        // An explicit override always wins; otherwise prefer an existing
        // `done/` subdirectory (same mimicry as the zero-config layout)
        // before falling back to bert's own `archive/tasks` layout default.
        let archive_tasks_directory = Some(match config.archive_tasks_directory.as_deref() {
            Some(p) => project_root.join(p),
            None => existing_done_dir(&tasks_directory)
                .unwrap_or_else(|| project_root.join(BertDirectoryLayout::archive_tasks_dir(&bert_root_rel))),
        });

        BertConfig {
            project_root: project_root.to_path_buf(),
            bert_root: bert_root_abs.clone(),

            tasks_directory,
            notes_directory: resolve_opt(
                &config.notes_directory,
                BertDirectoryLayout::notes_dir,
            ),
            archive_tasks_directory,
            archive_notes_directory: resolve_opt(
                &config.archive_notes_directory,
                BertDirectoryLayout::archive_notes_dir,
            ),
            specs_directory: resolve_req(
                config.specs_directory.as_ref(),
                BertDirectoryLayout::specs_dir,
            ),
            archive_specs_directory: resolve_opt(
                &config.archive_specs_directory,
                BertDirectoryLayout::archive_specs_dir,
            ),
            archive_directory: resolve_opt(
                &config.archive_directory,
                BertDirectoryLayout::archive_dir,
            ),
            product_directory: resolve_opt(
                &config.product_directory,
                BertDirectoryLayout::product_dir,
            ),
            prompt_logs: resolve_opt(&config.prompt_logs, BertDirectoryLayout::prompt_logs_dir),
            library_directory: resolve_opt(
                &config.library_directory,
                BertDirectoryLayout::prompts_library_dir,
            ),
            sets_directory: resolve_opt(
                &config.sets_directory,
                BertDirectoryLayout::prompts_sets_dir,
            ),

            tui: config.tui.unwrap_or_else(|| TuiConfig {
                pane_widths: Some(PaneWidths::default()),
            }),
            format: skill_config.format,
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;
    use std::path::Path;

    /// Fully-populated `BertConfig` rooted at `root`, for tests.
    /// Tests override the individual fields they vary.
    pub(crate) fn test_config(root: &Path) -> BertConfig {
        BertConfig {
            project_root: root.to_path_buf(),
            bert_root: root.join("bert"),
            tasks_directory: root.join("tasks"),
            notes_directory: Some(root.join("notes")),
            archive_tasks_directory: Some(root.join("archive/tasks")),
            archive_notes_directory: Some(root.join("archive/notes")),
            specs_directory: root.join("specs"),
            archive_specs_directory: Some(root.join("archive/specs")),
            archive_directory: Some(root.join("archive")),
            product_directory: Some(root.join("product")),
            prompt_logs: Some(root.join("prompt-logs")),
            library_directory: Some(root.join("library")),
            sets_directory: Some(root.join("sets")),
            tui: TuiConfig {
                pane_widths: Some(PaneWidths::default()),
            },
            format: None,
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
        assert_eq!(skill_config.config.as_ref().unwrap().bert_root, "docs/bert");
        assert!(skill_config.format.is_none());
    }

    #[test]
    fn test_resolve_paths_classic_layout() {
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
        // Companion directories nest under tasks/, not sibling to it —
        // same shape as the zero-config layout (see existing_done_dir).
        assert_eq!(bert_config.specs_directory, PathBuf::from("/project/root/docs/bert/tasks/specs"));
        assert_eq!(bert_config.notes_directory, Some(PathBuf::from("/project/root/docs/bert/tasks/notes")));
        assert_eq!(bert_config.library_directory, Some(PathBuf::from("/project/root/docs/bert/tasks/prompts/library")));
        assert_eq!(bert_config.sets_directory, Some(PathBuf::from("/project/root/docs/bert/tasks/prompts/sets")));
        assert!(bert_config.format.is_none());
    }

    #[test]
    fn test_zero_config_nests_under_tasks() {
        let skill_config: SkillConfig = serde_yaml::from_str("format:\n  slug: false\n").unwrap();
        assert!(skill_config.config.is_none());

        let bert_config = BertConfig::from_skill_config(skill_config, Path::new("/proj"));

        assert_eq!(bert_config.tasks_directory, PathBuf::from("/proj/docs/tasks"));
        assert_eq!(
            bert_config.notes_directory,
            Some(PathBuf::from("/proj/docs/tasks/notes"))
        );
        assert_eq!(
            bert_config.archive_tasks_directory,
            Some(PathBuf::from("/proj/docs/tasks/archive"))
        );
        assert_eq!(
            bert_config.specs_directory,
            PathBuf::from("/proj/docs/tasks/specs")
        );
        assert_eq!(bert_config.format.as_ref().unwrap().slug, Some(false));
    }

    #[test]
    fn test_zero_config_prefers_existing_done_dir_over_archive() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp_dir.path().join("docs/tasks/done")).unwrap();

        let skill_config = SkillConfig { config: None, format: None };
        let bert_config = BertConfig::from_skill_config(skill_config, temp_dir.path());

        assert_eq!(
            bert_config.archive_tasks_directory,
            Some(temp_dir.path().join("docs/tasks/done"))
        );
    }

    #[test]
    fn test_zero_config_falls_back_to_archive_when_no_done_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();

        let skill_config = SkillConfig { config: None, format: None };
        let bert_config = BertConfig::from_skill_config(skill_config, temp_dir.path());

        assert_eq!(
            bert_config.archive_tasks_directory,
            Some(temp_dir.path().join("docs/tasks/archive"))
        );
    }

    #[test]
    fn test_configured_layout_prefers_existing_done_dir_when_unset() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp_dir.path().join("docs/tasks/done")).unwrap();

        let yaml = "config:\n  bert_root: docs\n";
        let skill_config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        let bert_config = BertConfig::from_skill_config(skill_config, temp_dir.path());

        assert_eq!(bert_config.tasks_directory, temp_dir.path().join("docs/tasks"));
        assert_eq!(
            bert_config.archive_tasks_directory,
            Some(temp_dir.path().join("docs/tasks/done"))
        );
    }

    #[test]
    fn test_configured_layout_explicit_override_wins_over_done_dir() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(temp_dir.path().join("docs/tasks/done")).unwrap();
        std::fs::create_dir_all(temp_dir.path().join("elsewhere")).unwrap();

        let yaml = "config:\n  bert_root: docs\n  archive_tasks_directory: elsewhere\n";
        let skill_config: SkillConfig = serde_yaml::from_str(yaml).unwrap();
        let bert_config = BertConfig::from_skill_config(skill_config, temp_dir.path());

        assert_eq!(
            bert_config.archive_tasks_directory,
            Some(temp_dir.path().join("elsewhere"))
        );
    }
}
