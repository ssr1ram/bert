# Sets Feature Testing Guide

## Automated Verification

All automated checks have passed! ✅

Run the verification scripts:
```bash
./test_sets.sh      # Basic smoke test
./verify_sets.sh    # Comprehensive code verification
```

## Manual Testing Checklist

### 1. Launch the TUI
```bash
cargo run -- tui
# or
./target/debug/bert tui
```

### 2. Test Library/Sets View Toggle

**What to test:**
- [ ] Press `` ` `` (backtick) to toggle between Library and Sets
- [ ] Explorer title shows: `Explorer  Library Sets`
- [ ] Active view is **white and bold**
- [ ] Inactive view is **dark gray**

**Expected behavior:**
- Toggle works smoothly
- No visual glitches
- Tabs update immediately

---

### 3. Test Library View (File Selection)

**What to test:**
- [ ] Navigate with `j`/`k` (or arrow keys)
- [ ] Press `l` to expand folders
- [ ] Press `h` to collapse folders
- [ ] Press `Space` or `Enter` on a file to add to buffer
- [ ] Files appear in Working Buffer pane with full paths

**Expected behavior:**
- Navigation is smooth
- Folders expand/collapse correctly
- Files show relative paths in buffer (e.g., `1. style/code-formatting.md`)
- Duplicate files are not added twice

---

### 4. Test Sets View (Loading Sets)

**What to test:**
- [ ] Press `` ` `` to switch to Sets view
- [ ] See existing set "foo" listed
- [ ] Press `Space` or `Enter` on "foo"
- [ ] Files from set appear in Working Buffer
- [ ] **Click** on "foo" with mouse
- [ ] Files load correctly via mouse click

**Expected behavior:**
- Sets view shows all `.yaml` files from sets directory
- Loading a set adds all its files to buffer
- Duplicate files are silently skipped
- Both keyboard and mouse work

---

### 5. Test Set Preview

**What to test:**
- [ ] In Sets view, select a set
- [ ] Preview pane shows:
  - Set name
  - Created date
  - File count
  - List of files in the set

**Expected behavior:**
- Preview updates when selecting different sets
- All files in the set are listed
- Formatting is clean and readable

---

### 6. Test Saving a Set

**What to test:**
- [ ] Add 2-3 files to Working Buffer from Library view
- [ ] Press `Tab` to focus Queue pane
- [ ] Press `s` to save
- [ ] Input box appears at bottom
- [ ] Type a name (e.g., `my-test-set`)
- [ ] Press `Enter`
- [ ] Switch to Sets view
- [ ] New set appears in the list

**Expected behavior:**
- Input box has yellow border
- Typing works smoothly
- `Backspace` removes characters
- `Esc` cancels
- `Enter` saves the set
- Set file created in `docs/bert/prompts/sets/`

**Try invalid names:**
- [ ] Empty name (should fail)
- [ ] Name with spaces (should fail)
- [ ] Name with uppercase (should fail)
- [ ] Name with underscores (should fail)

---

### 7. Test Deleting a Set

**What to test:**
- [ ] In Sets view, select a set
- [ ] Press `d` to delete
- [ ] Set disappears from list
- [ ] File removed from `docs/bert/prompts/sets/`

**Expected behavior:**
- Deletion is immediate
- No confirmation prompt (careful!)
- Cursor adjusts if needed

---

### 8. Test Renaming a Set

**What to test:**
- [ ] In Sets view, select a set
- [ ] Press `r` to rename
- [ ] Input box shows: `Rename 'old-name' to: ___`
- [ ] Type new name (e.g., `renamed-set`)
- [ ] Press `Enter`
- [ ] Set appears with new name

**Expected behavior:**
- Old file is deleted
- New file is created
- Set name inside YAML is updated
- Files list remains unchanged

---

### 9. Test Help Modal

**What to test:**
- [ ] Press `?` to show help
- [ ] Modal appears centered
- [ ] Border matches theme (not yellow/black)
- [ ] Background matches theme
- [ ] All keybindings are listed
- [ ] Press `?` or `Esc` to close
- [ ] Returns to exact same state

**Expected behavior:**
- Help modal is properly centered
- Fits within terminal bounds
- Colors match main UI theme
- Closing preserves all state (cursor, buffer, expanded folders)

---

### 10. Test Mixed Workflow (Files + Sets)

**What to test:**
- [ ] Add 1-2 individual files from Library
- [ ] Switch to Sets view
- [ ] Load a set (Space/Enter)
- [ ] Switch back to Library
- [ ] Add more individual files
- [ ] Buffer contains mix of manually added files and set files
- [ ] Press `c` to copy
- [ ] Paste in a text editor to verify content

**Expected behavior:**
- Files and sets can be freely mixed
- No duplicates in buffer
- Copy includes all files with headers
- Order is preserved

---

### 11. Test Tab Navigation

**What to test:**
- [ ] Press `Tab` to cycle: Explorer → Queue → Preview → Explorer
- [ ] Press `Shift+Tab` to cycle backwards
- [ ] Press `←` to move left (no wrap)
- [ ] Press `→` to move right (no wrap)
- [ ] Active pane has bold border

**Expected behavior:**
- Pane switching is smooth
- Border highlights change correctly
- Keybindings work only in appropriate panes

---

### 12. Test Edge Cases

**Empty buffer:**
- [ ] Try to save empty buffer with `s`
- [ ] Should fail gracefully (no set created)

**Empty sets directory:**
- [ ] Delete all sets
- [ ] Switch to Sets view
- [ ] Shows "(empty)" or empty list

**Long filenames:**
- [ ] Add files with long paths
- [ ] Buffer shows full paths correctly
- [ ] No text overflow

**Small terminal:**
- [ ] Resize terminal to ~80x24
- [ ] UI still usable
- [ ] Help modal fits

---

## Verification Checklist

After testing, verify the following files:

**Check sets directory:**
```bash
ls -la docs/bert/prompts/sets/
cat docs/bert/prompts/sets/*.yaml
```

**Check YAML format:**
- Name field matches filename
- Created timestamp is valid
- Files list contains relative paths from library root

**Check no crashes:**
- No panics or error messages
- Graceful handling of invalid input
- Smooth operation throughout

---

## Known Limitations

1. **No undo** - Deleting a set is permanent
2. **No confirmation** - Delete happens immediately
3. **Case sensitive** - Set names are case-sensitive in the file system
4. **ASCII only** - Use basic characters in set names

---

## Success Criteria

✅ All manual tests pass without crashes
✅ UI is responsive and smooth
✅ Sets can be created, loaded, renamed, and deleted
✅ Files and sets can be mixed freely
✅ Help modal is properly styled and positioned
✅ Keyboard and mouse both work correctly

---

## Reporting Issues

If you find issues:

1. Note which test failed
2. Describe expected vs actual behavior
3. Check terminal size and color support
4. Share any error messages

---

**Last Updated:** October 29, 2025
**Version:** 0.2.1
