//! Shared directory-tree scanning for the file viewer and prompt builder

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::prompt_builder::{TreeItem, TreeItemType};
use crate::errors::Result;

/// Recursively build the display tree for `root`.
///
/// Entries are ordered folders-first, then alphabetically; only files
/// accepted by `keep_file` are included. Folder paths are relative to
/// `root`, matching the keys used in `expanded_folders`.
pub(crate) fn scan_directory_tree(
    root: &Path,
    expanded_folders: &HashSet<PathBuf>,
    keep_file: impl Fn(&Path) -> bool,
) -> Result<Vec<TreeItem>> {
    let mut items = Vec::new();
    scan_recursive(root, &PathBuf::new(), 0, expanded_folders, &keep_file, &mut items)?;
    Ok(items)
}

fn scan_recursive(
    root: &Path,
    relative_path: &Path,
    depth: usize,
    expanded_folders: &HashSet<PathBuf>,
    keep_file: &impl Fn(&Path) -> bool,
    items: &mut Vec<TreeItem>,
) -> Result<()> {
    let full_path = root.join(relative_path);

    let mut entries: Vec<_> = fs::read_dir(&full_path)?
        .filter_map(|e| e.ok())
        .collect();

    // Sort: folders first, then files, alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.path().is_dir();
        let b_is_dir = b.path().is_dir();

        match (a_is_dir, b_is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.file_name().cmp(&b.file_name()),
        }
    });

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let item_relative_path = relative_path.join(&name);

        if path.is_dir() {
            let is_expanded = expanded_folders.contains(&item_relative_path);

            items.push(TreeItem {
                path: item_relative_path.clone(),
                display_name: name.clone(),
                depth,
                item_type: TreeItemType::Folder,
                is_expanded,
            });

            // If expanded, recursively scan children
            if is_expanded {
                scan_recursive(root, &item_relative_path, depth + 1, expanded_folders, keep_file, items)?;
            }
        } else if keep_file(&path) {
            items.push(TreeItem {
                path: item_relative_path,
                display_name: name,
                depth,
                item_type: TreeItemType::File,
                is_expanded: false,
            });
        }
    }

    Ok(())
}
