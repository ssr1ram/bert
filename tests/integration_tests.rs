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
fn test_zero_config_creates_default_tasks_dir() {
    // No config anywhere: bert treats cwd as project root and defaults
    // to <root>/docs/tasks (self-contained layout).
    let temp_dir = TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.current_dir(temp_dir.path());
    cmd.arg("task").arg("stub").arg("test");
    cmd.assert().success();

    assert!(temp_dir.path().join("docs/tasks/task-01-test.md").exists());
}

#[test]
fn test_create_task_stub_integration() {
    // Legacy layout via a root-level skills.yml is still honored.
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::write(
        root.join("skills.yml"),
        "config:\n  bert_root: docs/bert\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("docs/bert/tasks")).unwrap();

    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.current_dir(root);
    cmd.arg("task").arg("stub").arg("test integration");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Created task 01"));

    let task_file = root.join("docs/bert/tasks/task-01-test-integration.md");
    assert!(task_file.exists());

    // Empty directory -> bert-native defaults: stub status, unzeroed H1 number
    let content = fs::read_to_string(task_file).unwrap();
    assert!(content.contains("status: stub"));
    assert!(content.contains("# Task 1: test integration"));
}

#[test]
fn test_archive_task_integration() {
    let temp_dir = TempDir::new().unwrap();
    let root = temp_dir.path();

    fs::write(
        root.join("skills.yml"),
        "config:\n  bert_root: docs/bert\n",
    )
    .unwrap();

    let tasks_dir = root.join("docs/bert/tasks");
    fs::create_dir_all(&tasks_dir).unwrap();
    fs::write(tasks_dir.join("task-01-test.md"), "test content").unwrap();

    let mut cmd = Command::cargo_bin("bert").unwrap();
    cmd.current_dir(root);
    cmd.arg("task").arg("archive").arg("01");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Archived"));

    assert!(!tasks_dir.join("task-01-test.md").exists());
    // Archive nests under tasks/ now, same shape as the zero-config layout.
    assert!(root.join("docs/bert/tasks/archive/task-01-test.md").exists());
}
