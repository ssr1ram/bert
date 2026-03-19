//! Default directory structure for BERT
//!
//! This module defines the standard directory layout under bert_root.
//! When users only specify bert_root in skill.yml, these paths are used.

use std::path::{Path, PathBuf};

/// Default directory structure relative to bert_root
pub struct BertDirectoryLayout;

impl BertDirectoryLayout {
    /// Tasks directory: {bert_root}/tasks
    pub fn tasks_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("tasks")
    }

    /// Notes directory: {bert_root}/notes
    pub fn notes_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("notes")
    }

    /// Specs directory: {bert_root}/specs
    pub fn specs_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("specs")
    }

    /// Product directory: {bert_root}/product
    pub fn product_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("product")
    }

    /// Archive root directory: {bert_root}/archive
    pub fn archive_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("archive")
    }

    /// Archive tasks directory: {bert_root}/archive/tasks
    pub fn archive_tasks_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("archive").join("tasks")
    }

    /// Archive notes directory: {bert_root}/archive/notes
    pub fn archive_notes_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("archive").join("notes")
    }

    /// Archive specs directory: {bert_root}/archive/specs
    pub fn archive_specs_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("archive").join("specs")
    }

    /// Prompts library directory: {bert_root}/prompts/library
    pub fn prompts_library_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("prompts").join("library")
    }

    /// Prompts sets directory: {bert_root}/prompts/sets
    pub fn prompts_sets_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("prompts").join("sets")
    }

    /// Prompt logs directory: {bert_root}/prompts/logs
    pub fn prompt_logs_dir(bert_root: &Path) -> PathBuf {
        bert_root.join("prompts").join("logs")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_directory_layout() {
        let bert_root = PathBuf::from("x-bert");

        assert_eq!(
            BertDirectoryLayout::tasks_dir(&bert_root),
            PathBuf::from("x-bert/tasks")
        );

        assert_eq!(
            BertDirectoryLayout::prompts_library_dir(&bert_root),
            PathBuf::from("x-bert/prompts/library")
        );

        assert_eq!(
            BertDirectoryLayout::archive_tasks_dir(&bert_root),
            PathBuf::from("x-bert/archive/tasks")
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
            PathBuf::from("docs/bert/prompts/sets")
        );
    }
}
