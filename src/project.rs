// Project root detection
//
// bert is zero-config by default:
// 1. Repo root is discovered with git (`git rev-parse --show-toplevel`)
// 2. Tasks live at `<repo_root>/docs/tasks` unless configured otherwise
// 3. Config, when needed, lives at `<repo_root>/.bert/config.yml`
//    (the historical `skills.yml` at a project root is still honored)
use crate::errors::Result;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Directory holding bert's own configuration, following the dot-directory
/// convention used by AI dev tools (`.claude/`, `.cursor/`, `.gemini/`).
pub const BERT_CONFIG_DIR: &str = ".bert";

/// Config file name inside [`BERT_CONFIG_DIR`].
pub const BERT_CONFIG_FILE: &str = "config.yml";

/// Legacy config file name, honored for backward compatibility.
pub const LEGACY_SKILLS_YML: &str = "skills.yml";

/// Find the bert project root by walking up the directory tree
///
/// Resolution order:
/// 1. If inside a git repository, the nearest `.bert/config.yml` between the
///    start directory and the repo toplevel (inclusive) anchors the project;
///    otherwise the repo toplevel is the project root.
/// 2. Outside a git repository, the nearest ancestor containing a config
///    file (`.bert/config.yml` or legacy `skills.yml`) is the project root.
/// 3. Otherwise the start directory itself is used (pure zero-config mode).
///
/// # Errors
///
/// Returns an I/O error only if the current directory cannot be determined.
/// The absence of git and of any config file is not an error — zero-config
/// mode uses the start directory as the project root.
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
pub fn find_project_root_from(start_dir: PathBuf) -> Result<PathBuf> {
    let current = start_dir.canonicalize()?;

    // 1. Prefer git: repo toplevel anchors the project unless a nearer
    //    `.bert/config.yml` opts out (e.g. bert scoped to a monorepo subtree).
    if let Some(toplevel) = find_git_repo_root(&current) {
        if let Some(anchor) = find_config_ancestor_between(&current, &toplevel) {
            return Ok(anchor);
        }
        return Ok(toplevel);
    }

    // 2. Fall back to walking up for any config marker.
    if let Some(anchor) = find_config_ancestor(&current) {
        return Ok(anchor);
    }

    // 3. Zero-config: the start directory is the project root.
    Ok(current)
}

/// Get the path to the config file for a given project root, if one exists
///
/// Checks the modern `.bert/config.yml` first, then the legacy `skills.yml`.
pub fn get_skill_yml_path(project_root: &Path) -> Option<PathBuf> {
    let candidates = [
        project_root.join(BERT_CONFIG_DIR).join(BERT_CONFIG_FILE),
        project_root.join(LEGACY_SKILLS_YML),
    ];
    candidates.into_iter().find(|p| p.is_file())
}

/// Ask git for the worktree toplevel containing `start_dir`
fn find_git_repo_root(start_dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(start_dir)
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let toplevel = stdout.trim();
        if !toplevel.is_empty() {
            return Some(PathBuf::from(toplevel));
        }
    }
    None
}

/// Walk from `start_dir` up to (and including) `stop_dir` looking for
/// `.bert/config.yml`; returns the directory containing the first hit.
fn find_config_ancestor_between(start_dir: &Path, stop_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();
    loop {
        let marker = current.join(BERT_CONFIG_DIR).join(BERT_CONFIG_FILE);
        if marker.is_file() {
            return Some(current);
        }
        if current == stop_dir {
            return None;
        }
        current = current.parent()?.to_path_buf();
    }
}

/// Walk from `start_dir` up the filesystem looking for any config marker;
/// returns the directory containing the first hit.
fn find_config_ancestor(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();
    loop {
        let modern = current.join(BERT_CONFIG_DIR).join(BERT_CONFIG_FILE);
        let legacy = current.join(LEGACY_SKILLS_YML);
        if modern.is_file() || legacy.is_file() {
            return Some(current);
        }
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Create an isolated git repo (skipped gracefully if git is unavailable)
    fn git_init(dir: &Path) -> bool {
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(dir)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[test]
    fn test_find_project_root_in_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        if !git_init(root) {
            return; // git unavailable in this environment
        }

        let subdir = root.join("a/b/c");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_project_root_from(subdir).unwrap();
        assert_eq!(found.canonicalize().unwrap(), root.canonicalize().unwrap());
    }

    #[test]
    fn test_git_repo_with_bert_config_in_subtree() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        if !git_init(root) {
            return;
        }

        // bert scoped to a monorepo subtree via .bert/config.yml
        let subtree = root.join("services/widget");
        fs::create_dir_all(subtree.join(".bert")).unwrap();
        fs::write(subtree.join(".bert/config.yml"), "config:\n  bert_root: docs\n").unwrap();

        let found = find_project_root_from(subtree.clone()).unwrap();
        assert_eq!(found.canonicalize().unwrap(), subtree.canonicalize().unwrap());

        // A sibling subtree still resolves to the repo toplevel
        let sibling = root.join("services/other");
        fs::create_dir_all(&sibling).unwrap();
        let found = find_project_root_from(sibling).unwrap();
        assert_eq!(found.canonicalize().unwrap(), root.canonicalize().unwrap());
    }

    #[test]
    fn test_find_project_root_falls_back_to_config_walk_without_git() {
        // TempDirs live outside any git repo on macOS/Linux CI
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        fs::write(root.join(LEGACY_SKILLS_YML), "config:\n  bert_root: docs/bert\n").unwrap();

        let subdir = root.join("deeply/nested/path");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_project_root_from(subdir).unwrap();
        assert_eq!(found.canonicalize().unwrap(), root.canonicalize().unwrap());
    }

    #[test]
    fn test_find_project_root_zero_config_defaults_to_start_dir() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("some/deep/path");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_project_root_from(subdir.clone()).unwrap();
        assert_eq!(found.canonicalize().unwrap(), subdir.canonicalize().unwrap());
    }

    #[test]
    fn test_get_skill_yml_path_prefers_modern_config() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        assert!(get_skill_yml_path(root).is_none());

        fs::create_dir_all(root.join(BERT_CONFIG_DIR)).unwrap();
        fs::write(
            root.join(BERT_CONFIG_DIR).join(BERT_CONFIG_FILE),
            "config:\n  bert_root: docs\n",
        )
        .unwrap();
        assert_eq!(
            get_skill_yml_path(root).unwrap(),
            root.join(BERT_CONFIG_DIR).join(BERT_CONFIG_FILE)
        );

        // Modern location wins over legacy when both exist
        fs::write(root.join(LEGACY_SKILLS_YML), "config:\n  bert_root: docs/bert\n").unwrap();
        assert_eq!(
            get_skill_yml_path(root).unwrap(),
            root.join(BERT_CONFIG_DIR).join(BERT_CONFIG_FILE)
        );
    }
}
