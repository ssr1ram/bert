//! Default directory structure for BERT
//!
//! This module defines the standard directory layout under bert_root. When
//! users only specify bert_root in `.bert/config.yml`, these paths are used
//! for anything they don't explicitly override.
//!
//! Everything nests under `{bert_root}/tasks/`, mirroring the zero-config
//! layout (`BertConfig::from_skill_config`'s config-less branch) exactly —
//! so having *any* `config:` section (even just a custom `bert_root`) never
//! by itself scatters companion directories as siblings of `tasks/` under
//! `bert_root`.

use std::path::{Path, PathBuf};

/// Default directory structure relative to bert_root
pub struct BertDirectoryLayout;

impl BertDirectoryLayout {
    /// Tasks directory: {bert_root}/tasks
    pub fn tasks_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("tasks")
    }

    /// Notes directory: {bert_root}/tasks/notes
    pub fn notes_dir(bert_root: &Path) -> PathBuf {
        Self::tasks_dir(bert_root).join("notes")
    }

    /// Specs directory: {bert_root}/tasks/specs
    pub fn specs_dir(bert_root: &Path) -> PathBuf {
        Self::tasks_dir(bert_root).join("specs")
    }

    /// Product directory: {bert_root}/tasks/product
    pub fn product_dir(bert_root: &Path) -> PathBuf {
        Self::tasks_dir(bert_root).join("product")
    }

    /// Archive root directory: {bert_root}/tasks/archive
    pub fn archive_dir(bert_root: &Path) -> PathBuf {
        Self::tasks_dir(bert_root).join("archive")
    }

    /// Archive tasks directory: {bert_root}/tasks/archive (completed task
    /// files land directly in the archive root, same as the zero-config
    /// layout — no extra `tasks/` segment inside it).
    pub fn archive_tasks_dir(bert_root: &Path) -> PathBuf {
        Self::archive_dir(bert_root)
    }

    /// Archive notes directory: {bert_root}/tasks/archive/notes
    pub fn archive_notes_dir(bert_root: &Path) -> PathBuf {
        Self::archive_dir(bert_root).join("notes")
    }

    /// Archive specs directory: {bert_root}/tasks/archive/specs
    pub fn archive_specs_dir(bert_root: &Path) -> PathBuf {
        Self::archive_dir(bert_root).join("specs")
    }

    /// Prompts library directory: {bert_root}/tasks/prompts/library
    pub fn prompts_library_dir(bert_root: &Path) -> PathBuf {
        Self::tasks_dir(bert_root).join("prompts").join("library")
    }

    /// Prompts sets directory: {bert_root}/tasks/prompts/sets
    pub fn prompts_sets_dir(bert_root: &Path) -> PathBuf {
        Self::tasks_dir(bert_root).join("prompts").join("sets")
    }

    /// Prompt logs directory: {bert_root}/tasks/prompts/logs
    pub fn prompt_logs_dir(bert_root: &Path) -> PathBuf {
        Self::tasks_dir(bert_root).join("prompts").join("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_directory_layout_nests_under_tasks() {
        let bert_root = PathBuf::from("x-bert");

        assert_eq!(
            BertDirectoryLayout::tasks_dir(&bert_root),
            PathBuf::from("x-bert/tasks")
        );

        assert_eq!(
            BertDirectoryLayout::notes_dir(&bert_root),
            PathBuf::from("x-bert/tasks/notes")
        );

        assert_eq!(
            BertDirectoryLayout::prompts_library_dir(&bert_root),
            PathBuf::from("x-bert/tasks/prompts/library")
        );

        // Archive tasks land directly in the archive root, not a nested
        // "tasks" subfolder of it — same shape as the zero-config layout.
        assert_eq!(
            BertDirectoryLayout::archive_dir(&bert_root),
            PathBuf::from("x-bert/tasks/archive")
        );
        assert_eq!(
            BertDirectoryLayout::archive_tasks_dir(&bert_root),
            PathBuf::from("x-bert/tasks/archive")
        );
        assert_eq!(
            BertDirectoryLayout::archive_notes_dir(&bert_root),
            PathBuf::from("x-bert/tasks/archive/notes")
        );
    }

    #[test]
    fn test_directory_layout_with_docs_bert() {
        let bert_root = PathBuf::from("docs/bert");

        assert_eq!(
            BertDirectoryLayout::tasks_dir(&bert_root),
            PathBuf::from("docs/bert/tasks")
        );

        assert_eq!(
            BertDirectoryLayout::prompts_sets_dir(&bert_root),
            PathBuf::from("docs/bert/tasks/prompts/sets")
        );

        assert_eq!(
            BertDirectoryLayout::product_dir(&bert_root),
            PathBuf::from("docs/bert/tasks/product")
        );
    }
}
