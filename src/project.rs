// Project root detection
use crate::errors::{BertError, Result};
use std::env;
use std::path::{Path, PathBuf};

/// The relative path to the skill.yml file from project root
/// The relative path to the skill.yml file from project root
const SKILL_YML_PATH: &str = "skills.yml";

/// Find the bert project root by walking up the directory tree
///
/// Starts from the current directory and walks up, looking for
/// `skills.yml`. Returns the directory containing
/// the `skills.yml` file.
///
/// # Errors
///
/// Returns `BertError::ProjectNotFound` if no project root is found
/// before reaching the filesystem root.
///
/// # Example
///
/// ```no_run
/// use bert_cli::project::find_project_root;
///
/// let root = find_project_root()?;
/// println!("Project root: {}", root.display());
/// # Ok::<(), bert_cli::errors::BertError>(())
/// ```
pub fn find_project_root() -> Result<PathBuf> {
    find_project_root_from(env::current_dir()?)
}

/// Find the bert project root starting from a specific directory
///
/// This is primarily used for testing, allowing tests to specify
/// a starting directory.
///
/// # Errors
///
/// Returns `BertError::ProjectNotFound` if no project root is found.
pub fn find_project_root_from(start_dir: PathBuf) -> Result<PathBuf> {
    let mut current = start_dir.canonicalize()?;

    loop {
        // Check if skills.yml exists in current directory
        let skill_path = current.join(SKILL_YML_PATH);

        if skill_path.exists() && skill_path.is_file() {
            return Ok(current);
        }

        // Try to move up to parent directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => {
                // Reached filesystem root without finding project
                return Err(BertError::ProjectNotFound(
                    env::current_dir().unwrap_or_else(|_| PathBuf::from("unknown"))
                ));
            }
        }
    }
}

/// Get the path to skills.yml for a given project root
pub fn get_skill_yml_path(project_root: &Path) -> PathBuf {
    project_root.join(SKILL_YML_PATH)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_project_root_from_root() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create skills.yml
        fs::write(root.join("skills.yml"), "test: true").unwrap();

        // Should find root from root directory
        let found = find_project_root_from(root.to_path_buf()).unwrap();
        assert_eq!(found.canonicalize().unwrap(), root.canonicalize().unwrap());
    }

    #[test]
    fn test_find_project_root_from_subdirectory() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create skills.yml
        fs::write(root.join("skills.yml"), "test: true").unwrap();

        // Create nested subdirectory
        let subdir = root.join("docs/bert/tasks");
        fs::create_dir_all(&subdir).unwrap();

        // Should find root from subdirectory
        let found = find_project_root_from(subdir).unwrap();
        assert_eq!(found.canonicalize().unwrap(), root.canonicalize().unwrap());
    }

    #[test]
    fn test_find_project_root_not_found() {
        // Create temporary directory without skills.yml
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path().join("some/deep/path");
        fs::create_dir_all(&root).unwrap();

        // Should return ProjectNotFound error
        let result = find_project_root_from(root);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BertError::ProjectNotFound(_)));
    }

    #[test]
    fn test_get_skill_yml_path() {
        let root = PathBuf::from("/project/root");
        let skill_path = get_skill_yml_path(&root);
        assert_eq!(
            skill_path,
            PathBuf::from("/project/root/skills.yml")
        );
    }

    #[test]
    fn test_find_project_root_with_multiple_levels() {
        // Create temporary directory structure
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create skills.yml
        fs::write(root.join("skills.yml"), "test: true").unwrap();

        // Create deeply nested subdirectory
        let deep_subdir = root.join("a/b/c/d/e/f");
        fs::create_dir_all(&deep_subdir).unwrap();

        // Should still find root
        let found = find_project_root_from(deep_subdir).unwrap();
        assert_eq!(found.canonicalize().unwrap(), root.canonicalize().unwrap());
    }
}
