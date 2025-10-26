// Integration tests for bert CLI
use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Rust CLI for Bert"));
}

#[test]
fn test_version_flag() {
    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("bert"));
}

#[test]
fn test_task_help() {
    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.arg("task").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Task management operations"));
}

#[test]
fn test_task_stub_help() {
    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.arg("task").arg("stub").arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Create a new task stub"));
}

#[test]
fn test_project_not_found_error() {
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd.arg("task").arg("stub").arg("test");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Not in a bert project"));
}

#[test]
fn test_create_task_stub_integration() {
    // Create a temporary bert project
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Set up project structure
    let skill_dir = root.join(".claude/skills/bert");
    fs::create_dir_all(&skill_dir).unwrap();

    let config = r#"
config:
  tasks_directory: docs/bert/tasks
  specs_directory: docs/bert/specs
  archive_tasks_directory: docs/bert/archive/tasks
  archive_specs_directory: docs/bert/archive/specs
"#;
    fs::write(skill_dir.join("skill.yml"), config).unwrap();

    // Create tasks directory
    fs::create_dir_all(root.join("docs/bert/tasks")).unwrap();

    // Run bert task stub
    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.current_dir(root);
    cmd.arg("task").arg("stub").arg("test integration");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Created task 01"));

    // Verify task file was created
    let task_file = root.join("docs/bert/tasks/task-01-test-integration.md");
    assert!(task_file.exists());

    // Verify file content
    let content = fs::read_to_string(task_file).unwrap();
    assert!(content.contains("status: pending"));
    assert!(content.contains("# Task 01: test integration"));
}

#[test]
fn test_archive_task_integration() {
    // Create a temporary bert project
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    // Set up project structure
    let skill_dir = root.join(".claude/skills/bert");
    fs::create_dir_all(&skill_dir).unwrap();

    let config = r#"
config:
  tasks_directory: docs/bert/tasks
  specs_directory: docs/bert/specs
  archive_tasks_directory: docs/bert/archive/tasks
  archive_specs_directory: docs/bert/archive/specs
"#;
    fs::write(skill_dir.join("skill.yml"), config).unwrap();

    // Create tasks directory and a task file
    let tasks_dir = root.join("docs/bert/tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    fs::write(tasks_dir.join("task-01-test.md"), "test content").unwrap();

    // Run bert task archive
    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.current_dir(root);
    cmd.arg("task").arg("archive").arg("01");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Archived"));

    // Verify task was moved
    assert!(!tasks_dir.join("task-01-test.md").exists());
    assert!(root.join("docs/bert/archive/tasks/task-01-test.md").exists());
}
