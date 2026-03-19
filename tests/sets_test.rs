// Note: This test file requires the library to be properly set up
// For now, we'll create a simpler validation test

use std::fs;
use std::path::PathBuf;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_create_and_save_set() {
    let temp_dir = TempDir::new().unwrap();
    let sets_dir = temp_dir.path().to_path_buf();

    let files = vec![
        PathBuf::from("file1.md"),
        PathBuf::from("folder/file2.md"),
    ];

    let set = PromptSet::new("test-set".to_string(), files.clone());

    // Save the set
    let saved_path = set.save(&sets_dir).unwrap();
    assert!(saved_path.exists());

    // Load it back
    let loaded_set = PromptSet::from_file(&saved_path).unwrap();
    assert_eq!(loaded_set.name, "test-set");
    assert_eq!(loaded_set.files, files);
}

#[test]
fn test_validate_set_name() {
    // Valid names
    assert!(PromptSet::validate_name("my-set").is_ok());
    assert!(PromptSet::validate_name("api-docs").is_ok());
    assert!(PromptSet::validate_name("test-123").is_ok());

    // Invalid names
    assert!(PromptSet::validate_name("").is_err());
    assert!(PromptSet::validate_name("My-Set").is_err()); // uppercase
    assert!(PromptSet::validate_name("my_set").is_err()); // underscore
    assert!(PromptSet::validate_name("my set").is_err()); // space
    assert!(PromptSet::validate_name("-myset").is_err()); // starts with dash
    assert!(PromptSet::validate_name("myset-").is_err()); // ends with dash
}

#[test]
fn test_delete_set() {
    let temp_dir = TempDir::new().unwrap();
    let sets_dir = temp_dir.path().to_path_buf();

    let files = vec![PathBuf::from("file1.md")];
    let set = PromptSet::new("delete-me".to_string(), files);
    let saved_path = set.save(&sets_dir).unwrap();

    assert!(saved_path.exists());

    // Delete the set
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

    // Rename the set
    PromptSet::rename(&sets_dir, "old-name", "new-name").unwrap();

    // Old file should not exist
    let old_path = sets_dir.join("old-name.yaml");
    assert!(!old_path.exists());

    // New file should exist with updated name
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

    // Create two sets
    let set1 = PromptSet::new("set1".to_string(), files.clone());
    set1.save(&sets_dir).unwrap();

    let set2 = PromptSet::new("set2".to_string(), files);
    set2.save(&sets_dir).unwrap();

    // Try to rename set1 to set2 (should fail)
    let result = PromptSet::rename(&sets_dir, "set1", "set2");
    assert!(result.is_err());
}
