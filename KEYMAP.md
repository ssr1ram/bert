# BERT TUI Keymap Reference

## Global Keys (Work Everywhere)

| Key | Action | Context |
|-----|--------|---------|
| `q`, `Esc` | Quit application | All modes |
| `1`, `F1` | Switch to Prompt Builder | All modes |
| `2`, `F2` | Switch to Spec Viewer | All modes |
| `3`, `F3` | Switch to Task Viewer | All modes |
| `4`, `F4` | Switch to Archive Viewer | All modes |
| `5`, `F5` | Switch to Settings | All modes |
| `p` | Toggle preview mode (Raw ↔ Rendered) | All modes |
| `Ctrl+C` | Force quit | All modes |

---

## Prompt Builder Mode

### Global (Any Pane)

| Key | Action | Notes |
|-----|--------|-------|
| `` ` `` (backtick) | Toggle Library ↔ Sets view | Switches explorer content |
| `Tab` | Next pane (→) | Explorer → Queue → Preview → Explorer |
| `Shift+Tab` | Previous pane (←) | Explorer ← Queue ← Preview ← Explorer |
| `←` (Left Arrow) | Move pane left | No wrap (stops at leftmost) |
| `→` (Right Arrow) | Move pane right | No wrap (stops at rightmost) |

### Explorer Pane

| Key | Action | Applies To | Notes |
|-----|--------|------------|-------|
| `j`, `Down` | Move cursor down | All items | |
| `k`, `Up` | Move cursor up | All items | |
| `l`, `Enter` | Expand/collapse folder OR add file | Context-sensitive | Folders: toggle expand; Files: add to queue |
| `Space` | Add file to queue | Files only | Same as Enter for files |
| `h` | Collapse folder OR move to parent | Context-sensitive | Folders: collapse; Files: jump to parent |
| `E` | Expand all folders | All folders | Recursive expand |
| `C` | Collapse all folders | All folders | |

**Mouse**: Click on item to select and expand/add

### Queue Pane (Working Buffer)

| Key | Action | Notes |
|-----|--------|-------|
| `j`, `Down` | Move cursor down | Select item in queue |
| `k`, `Up` | Move cursor up | Select item in queue |
| `d`, `Delete` | Remove selected item | Removes from buffer |
| `c` | Copy buffer to clipboard | Copies all file contents with headers |

**Mouse**:
- Click on title to copy
- Click on item to select

### Preview Pane

| Key | Action | Notes |
|-----|--------|-------|
| (read-only) | View selected file | Shows raw or rendered markdown |

**Mouse**: Click to focus pane

---

## Spec Viewer Mode

| Key | Action | Notes |
|-----|--------|-------|
| `j`, `Down` | Move cursor down | |
| `k`, `Up` | Move cursor up | |
| `l`, `Enter` | Expand/collapse folder OR view file | Context-sensitive |
| `h` | Collapse folder OR move to parent | Context-sensitive |
| `E` | Expand all folders | Recursive expand |
| `C` | Collapse all folders | |
| `a` | Archive selected spec | Archives spec folder (folders only) |

---

## Task Viewer Mode

| Key | Action | Notes |
|-----|--------|-------|
| `j`, `Down` | Move cursor down | |
| `k`, `Up` | Move cursor up | |
| `l`, `Enter` | Expand/collapse folder OR view file | Context-sensitive |
| `h` | Collapse folder OR move to parent | Context-sensitive |
| `E` | Expand all folders | Recursive expand |
| `C` | Collapse all folders | |

---

## Archive Viewer Mode

| Key | Action | Notes |
|-----|--------|-------|
| `j`, `Down` | Move cursor down | |
| `k`, `Up` | Move cursor up | |
| `l`, `Enter` | Expand/collapse folder OR view file | Context-sensitive |
| `h` | Collapse folder OR move to parent | Context-sensitive |
| `E` | Expand all folders | Recursive expand |
| `C` | Collapse all folders | |

---

## Settings Mode

*(To be implemented)*

---

## Available Keys for New Features

### Lowercase Letters
- `a` - Used in Spec Viewer (archive)
- `b` - **AVAILABLE**
- `c` - Used in Queue (copy)
- `d` - Used in Queue (delete)
- `e` - **AVAILABLE**
- `f` - **AVAILABLE**
- `g` - **AVAILABLE**
- `h` - Used (collapse/parent)
- `i` - **AVAILABLE**
- `j` - Used (down)
- `k` - Used (up)
- `l` - Used (expand/add)
- `m` - **AVAILABLE**
- `n` - **AVAILABLE**
- `o` - **AVAILABLE**
- `p` - Used (toggle preview)
- `q` - Used (quit)
- `r` - **AVAILABLE**
- `s` - **AVAILABLE** ← Suggested for "Save set"
- `t` - **AVAILABLE**
- `u` - **AVAILABLE**
- `v` - **AVAILABLE** ← Alternative to backtick for view toggle
- `w` - **AVAILABLE**
- `x` - **AVAILABLE**
- `y` - **AVAILABLE**
- `z` - **AVAILABLE**

### Uppercase Letters
- `A-B` - **AVAILABLE**
- `C` - Used (collapse all)
- `D-N` - **AVAILABLE**
- `E` - Used (expand all)
- `F-Z` - **AVAILABLE**

### Special Characters
- `` ` `` - Used (toggle Library/Sets)
- `Space` - Used (add to queue)
- `Enter` - Used (expand/add)
- `Esc` - Used (quit)
- `Tab` - Used (next pane)
- `Shift+Tab` - Used (previous pane)
- `Delete` - Used (remove from queue)
- `?` - **AVAILABLE** ← Suggested for help

### Function Keys
- `F1-F5` - Used (menu navigation)
- `F6-F12` - **AVAILABLE**

### Number Keys
- `1-5` - Used (menu navigation)
- `6-9, 0` - **AVAILABLE**

---

## Recommended Keys for New Features

Based on common TUI conventions:

- `?` - Show help/keybindings
- `s` - Save current buffer as set
- `/` - Search (future feature)
- `n`/`N` - Next/previous search result (future feature)
- `g`/`G` - Jump to top/bottom (future feature)
- `i` - Insert/edit mode (if needed)
- `r` - Rename (for sets)
- `v` - Alternative to backtick for view toggle (more intuitive)
- `x` - Quick delete (alternative to `d`)
- `y` - Yank/copy (vim-style alternative to `c`)
- `u` - Undo (future feature)

---

## Key Design Principles

1. **Vim-style navigation**: `hjkl` for movement
2. **Context-sensitive actions**: Same key does different things based on item type
3. **Lowercase for common**: Common actions use lowercase letters
4. **Uppercase for bulk**: Bulk operations (E=expand all, C=collapse all) use uppercase
5. **No conflicts**: Keys behave consistently across modes where possible
6. **Mouse support**: Most actions have mouse equivalents for accessibility
