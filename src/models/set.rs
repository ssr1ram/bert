//! Prompt set data model and serialization

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};

/// A saved prompt set containing a collection of file paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSet {
    /// Name of the set (lowercase-with-dashes format)
    pub name: String,

    /// Creation timestamp
    pub created: DateTime<Utc>,

    /// List of file paths relative to library root
    pub files: Vec<PathBuf>,
}

impl PromptSet {
    /// Create a new prompt set
    pub fn new(name: String, files: Vec<PathBuf>) -> Self {
        Self {
            name,
            created: Utc::now(),
            files,
        }
    }

    /// Validate set name (lowercase, alphanumeric, dashes only)
    pub fn validate_name(name: &str) -> Result<(), String> {
        if name.is_empty() {
            return Err("Set name cannot be empty".to_string());
        }

        if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
            return Err("Set name must be lowercase with dashes (e.g., my-set-name)".to_string());
        }

        if name.starts_with('-') || name.ends_with('-') {
            return Err("Set name cannot start or end with a dash".to_string());
        }

        Ok(())
    }

    /// Load a set from a YAML file
    pub fn from_file(path: &Path) -> crate::errors::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let set: PromptSet = serde_yaml::from_str(&content)
            .map_err(|e| crate::errors::BertError::ConfigError(format!("Failed to parse set: {}", e)))?;
        Ok(set)
    }

    /// Save the set to a YAML file
    pub fn save(&self, sets_dir: &Path) -> crate::errors::Result<PathBuf> {
        // Ensure sets directory exists
        std::fs::create_dir_all(sets_dir)?;

        let file_path = sets_dir.join(format!("{}.yaml", self.name));
        let yaml = serde_yaml::to_string(&self)
            .map_err(|e| crate::errors::BertError::ConfigError(format!("Failed to serialize set: {}", e)))?;

        std::fs::write(&file_path, yaml)?;
        Ok(file_path)
    }

    /// Delete the set file
    pub fn delete(sets_dir: &Path, name: &str) -> crate::errors::Result<()> {
        let file_path = sets_dir.join(format!("{}.yaml", name));
        std::fs::remove_file(file_path)?;
        Ok(())
    }

    /// Rename a set
    pub fn rename(sets_dir: &Path, old_name: &str, new_name: &str) -> crate::errors::Result<()> {
        // Validate new name
        Self::validate_name(new_name)
            .map_err(crate::errors::BertError::ConfigError)?;

        let old_path = sets_dir.join(format!("{}.yaml", old_name));
        let new_path = sets_dir.join(format!("{}.yaml", new_name));

        // Check if new name already exists
        if new_path.exists() {
            return Err(crate::errors::BertError::ConfigError(
                format!("Set '{}' already exists", new_name)
            ));
        }

        // Load, update name, and save
        let mut set = Self::from_file(&old_path)?;
        set.name = new_name.to_string();
        std::fs::remove_file(old_path)?;
        set.save(sets_dir)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_validate_name_valid() {
        assert!(PromptSet::validate_name("my-set").is_ok());
        assert!(PromptSet::validate_name("api-docs").is_ok());
        assert!(PromptSet::validate_name("test-123").is_ok());
        assert!(PromptSet::validate_name("a").is_ok());
    }

    #[test]
    fn test_validate_name_invalid() {
        assert!(PromptSet::validate_name("").is_err());
        assert!(PromptSet::validate_name("My-Set").is_err()); // uppercase
        assert!(PromptSet::validate_name("my_set").is_err()); // underscore
        assert!(PromptSet::validate_name("my set").is_err()); // space
        assert!(PromptSet::validate_name("-myset").is_err()); // starts with dash
        assert!(PromptSet::validate_name("myset-").is_err()); // ends with dash
    }

    #[test]
    fn test_create_set() {
        let files = vec![
            PathBuf::from("path/to/file1.md"),
            PathBuf::from("path/to/file2.md"),
        ];

        let set = PromptSet::new("test-set".to_string(), files.clone());

        assert_eq!(set.name, "test-set");
        assert_eq!(set.files, files);
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        let sets_dir = temp_dir.path().to_path_buf();

        let files = vec![
            PathBuf::from("file1.md"),
            PathBuf::from("folder/file2.md"),
        ];
        let set = PromptSet::new("test-set".to_string(), files.clone());

        let saved_path = set.save(&sets_dir).unwrap();
        assert!(saved_path.exists());

        let loaded_set = PromptSet::from_file(&saved_path).unwrap();
        assert_eq!(loaded_set.name, "test-set");
        assert_eq!(loaded_set.files, files);
    }

    #[test]
    fn test_delete_set() {
        let temp_dir = TempDir::new().unwrap();
        let sets_dir = temp_dir.path().to_path_buf();

        let files = vec![PathBuf::from("file1.md")];
        let set = PromptSet::new("delete-me".to_string(), files);
        let saved_path = set.save(&sets_dir).unwrap();
        assert!(saved_path.exists());

        PromptSet::delete(&sets_dir, "delete-me").unwrap();
        assert!(!saved_path.exists());
    }

    #[test]
    fn test_rename_set() {
        let temp_dir = TempDir::new().unwrap();
        let sets_dir = temp_dir.path().to_path_buf();

        let files = vec![PathBuf::from("file1.md")];
        let set = PromptSet::new("old-name".to_string(), files.clone());
        set.save(&sets_dir).unwrap();

        PromptSet::rename(&sets_dir, "old-name", "new-name").unwrap();

        let old_path = sets_dir.join("old-name.yaml");
        assert!(!old_path.exists());

        let new_path = sets_dir.join("new-name.yaml");
        assert!(new_path.exists());

        let loaded_set = PromptSet::from_file(&new_path).unwrap();
        assert_eq!(loaded_set.name, "new-name");
        assert_eq!(loaded_set.files, files);
    }

    #[test]
    fn test_rename_set_conflict() {
        let temp_dir = TempDir::new().unwrap();
        let sets_dir = temp_dir.path().to_path_buf();

        let files = vec![PathBuf::from("file1.md")];
        PromptSet::new("set1".to_string(), files.clone()).save(&sets_dir).unwrap();
        PromptSet::new("set2".to_string(), files).save(&sets_dir).unwrap();

        // Renaming set1 onto existing set2 must fail
        assert!(PromptSet::rename(&sets_dir, "set1", "set2").is_err());
    }
}
