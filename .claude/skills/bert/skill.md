# Bert - Build, Execute, and Refine Tasks

You are now activating the **Bert** skill for task management and tracking.

## Overview

Bert is a task management system that helps you create, organize, and track hierarchical tasks with arbitrary nesting depth. It provides structured task authoring, smart numbering, and status tracking.

## Command Routing

Bert can be invoked via the `/bert` command with subcommands, or using natural language.

### Subcommand Syntax

**Archive**: `/bert archive <task_number>`
- Example: `/bert archive 3`
- Routes to: archive-task operation (section 5)
- Extracts: task_number from first argument

**List**: `/bert list [filter]`
- Example: `/bert list`
- Example: `/bert list pending`
- Routes to: list-tasks operation (section 3)
- Optional: filter by status (pending, in-progress, completed, blocked)

**Status**: `/bert status <task_number> <new_status>`
- Example: `/bert status 3 completed`
- Routes to: update-status operation (section 4)
- Extracts: task_number (first arg), new_status (second arg)

**Help**: `/bert help` or `/bert`
- Displays usage information and available operations

### Routing Workflow

When invoked via `/bert <subcommand> [args]`:

1. **Parse subcommand**: Extract first word after `/bert`
2. **Validate**: Check subcommand is valid (archive|list|status|help)
3. **Extract arguments**: Get task_number, status, filter, etc.
4. **Route to operation**: Jump to appropriate section below
5. **Execute bash script**: Use embedded bash script for that operation
6. **Format results**: Parse JSON output and display user-friendly message

### Natural Language Support

Bert also activates for natural language patterns:
- "archive task 3" → archive-task
- "show me all tasks" → list-tasks
- "mark task 3 as completed" → update-status

Both approaches (slash command and natural language) use the same embedded bash scripts.

### Error Handling

If subcommand not recognized or invalid arguments:
```
Error: Unknown bert subcommand or invalid arguments

Usage:
  /bert archive <task_number>
  /bert list [filter]
  /bert status <task_number> <new_status>

Examples:
  /bert archive 3
  /bert list pending
  /bert status 3 completed
```

## Configuration

**Tasks Directory**: `docs/bert/tasks/` (configurable in `.claude/skills/bert/skill.yml`)
**Notes Directory**: `docs/bert/notes/` (configurable in `.claude/skills/bert/skill.yml`)
**Archive Tasks Directory**: `docs/bert/archive/tasks/` (configurable in `.claude/skills/bert/skill.yml`)
**Archive Notes Directory**: `docs/bert/archive/notes/` (configurable in `.claude/skills/bert/skill.yml`)

Read the config file to determine the actual directories for all operations.

## Universal Numbering System

**CRITICAL**: Specs and tasks share a unified numbering sequence to prevent collisions.

**Numbering Script**: `.claude/skills/bert/scripts/find-next-number.sh`

**What it scans**:
- Active tasks: `{tasks_directory}/task-{nn}-*.md`
- Archived tasks: `{archive_tasks_directory}/task-{nn}-*.md`
- Active specs: `{specs_directory}/spec-{nn}/`
- Archived specs: `{archive_specs_directory}/spec-{nn}/`

**Returns**: Next available number (2-digit padded)

**Used by**:
- `/bert:task create` - For top-level tasks
- `/bert:spec new` - For new specs

**Example**:
```bash
# Current state:
task-01-gather-steps.md
task-02-exe-web.md
spec-03/

# Next number: 04 (for either task OR spec)
```

**Why this matters**:
- Prevents `task-01.1` from colliding with `task-01`
- Ensures chronological ordering across all work
- Makes numbering predictable and consistent
- Archives are considered to prevent number reuse

## Core Operations

### 1. Task Author (`task-author`)

Generate a parent task file from file analysis or improvement requests.

**Usage Pattern**: User says "create a task for [file/concept]" or "analyze [file] for improvements"

**Workflow**:

1. **Parse Input**:
   - File path → Analyze specific file
   - Abstract concept → Search codebase for relevant files

2. **For File Analysis**:
   - Read the specified file
   - Identify improvement areas: unclear sections, gaps, outdated info
   - Generate specific, actionable suggestions with rationale

3. **For Abstract Requests**:
   - Search codebase using Glob/Grep
   - Present findings for user confirmation
   - Analyze identified files after confirmation

4. **Determine Task Number**:
   - Scan tasks directory for highest task number (e.g., task-04-*.md → 04)
   - Increment by 1 (next: 05)
   - Start at 01 if no tasks exist

5. **Create Parent Task File**:
   - Filename: `task-{nn}-{slug}.md`
   - Structure:
     ```markdown
     ---
     status: pending
     created: YYYY-MM-DD
     ---

     # Task {nn}: {Title}

     ## Description

     [Analysis context and what was analyzed]

     ## Tasks

     - [ ] {nn}.01 [Specific improvement description]
     - [ ] {nn}.02 [Specific improvement description]
     - [ ] {nn}.03 [Specific improvement description]

     ## Rationale

     [WHY these improvements matter - connect to goals, quality, outcomes]
     ```

6. **Output**:
   - Display created file path
   - Summarize findings count
   - Suggest: "Use bert to create subtasks with `-p {nn}`"

**Important**:
- Do NOT auto-generate subtask files - only create parent
- Focus on specific, actionable improvements
- Provide context for WHY each matters
- If file doesn't exist, provide clear error

### 2. Task Create (`task-create`)

Create task files with support for arbitrary nesting depth.

**Usage Patterns**:
- "create top-level task [description]" → New parent task
- "create subtasks for task {nn}" or "expand task {nn}" → Generate subtasks
- "create subtasks for task {nn}.{sub}" → Nested subtasks

**For Top-Level Tasks** (no parent):

1. Get or use provided task description
2. Determine next task number from directory scan
3. Generate kebab-case slug
4. Create file: `task-{nn}-{slug}.md`
5. Add frontmatter with status: pending, created date
6. Confirm creation with file path

**For Subtasks** (with parent):

1. **Parse Parent Number**:
   - Format: `3`, `4.1`, `4.1.2`, `4.1.2.3` (arbitrary depth)
   - Split on dots: `4.1.2` → ['4', '1', '2']

2. **Find Parent File**:
   - Pattern: `task-{parent_number}-*.md`
   - Examples: `task-04-*.md`, `task-04.1-*.md`, `task-04.1.2-*.md`
   - Error if not found

3. **Extract Unchecked Tasks**:
   - Parse `## Tasks` section
   - Find all `- [ ]` items

4. **Determine Subtask Numbers**:
   - Check existing subtasks at target level
   - For parent `4`: check `task-04.1-*.md`, `task-04.2-*.md`
   - For parent `4.1`: check `task-04.1.1-*.md`, `task-04.1.2-*.md`

5. **Calculate Smart Padding**:
   - Scan ALL tasks at target nesting level
   - Find maximum task number
   - Pad to ensure proper sorting
   - Examples:
     - Max 9: pad to 2 digits (01-09)
     - Max 12: pad to 2 digits (01-12)
     - Max 100: pad to 3 digits (001-100)

6. **Create Subtask Files**:
   - For each unchecked task in parent
   - Filename pattern (use dots, NOT dashes):
     - Parent `4` → `task-04.1-{slug}.md`, `task-04.2-{slug}.md`
     - Parent `4.1` → `task-04.1.1-{slug}.md`, `task-04.1.2-{slug}.md`
     - Parent `4.1.2` → `task-04.1.2.1-{slug}.md`
   - Frontmatter includes `parent: {parent_number}`
   - Description from parent task text
   - Include empty `## Tasks` section for future nesting

7. **Update Parent File**:
   - Apply smart padding to checkbox numbers
   - Format: `{major}.{padded_child}.{unpadded_grandchild}`
   - Examples:
     - Parent `4`, max 9: `4.01`, `4.02`, ..., `4.09`
     - Parent `4`, max 12: `4.01`, `4.02`, ..., `4.12`
     - Parent `4.1`, max 3: `4.01.1`, `4.01.2`, `4.01.3`
     - Parent `4.1`, max 10: `4.01.01`, `4.01.02`, ..., `4.01.10`
   - Only pad immediate child level
   - Preserve checkbox state

8. **Confirm**:
   - List created subtask files
   - Confirm parent updated

**Naming Rules**:
- Use dots (.) as separators: `task-04.1.2-slug.md` ✓
- NOT dashes: `task-04-1-2-slug.md` ✗
- Preserve dots in parent references

#### Bash Implementation

```bash
#!/bin/bash
# task-create.sh - Create a new task or subtask
# Usage: bash task-create.sh <config_file> [-p <parent_number>] <description>
# Example: bash task-create.sh .claude/skills/bert/skill.yml "article scorer"
# Example: bash task-create.sh .claude/skills/bert/skill.yml -p 3 "foo"

set -e

CONFIG_FILE="$1"
shift

# Parse optional -p flag
PARENT_NUM=""
if [[ "$1" == "-p" ]]; then
    PARENT_NUM="$2"
    shift 2
fi

DESCRIPTION="$*"

if [[ -z "$CONFIG_FILE" ]] || [[ -z "$DESCRIPTION" ]]; then
    echo '{"error": "Usage: task-create.sh <config_file> [-p <parent_number>] <description>"}' >&2
    exit 1
fi

# Read config
TASKS_DIR=$(grep "tasks_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')
ARCHIVE_TASKS=$(grep "archive_tasks_directory:" "$CONFIG_FILE" | awk '{print $2}')

# Create tasks directory if it doesn't exist
mkdir -p "$TASKS_DIR"

# Function to convert string to kebab-case slug
to_kebab_case() {
    echo "$1" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]/-/g' | sed 's/--*/-/g' | sed 's/^-//' | sed 's/-$//'
}

# If no parent, create top-level task
if [[ -z "$PARENT_NUM" ]]; then
    # Use universal numbering script (scans both tasks AND specs)
    SCRIPT_DIR="$(cd "$(dirname "$CONFIG_FILE")" && pwd)/skills/bert/scripts"
    TASK_NUM=$(bash "$SCRIPT_DIR/find-next-number.sh" "$CONFIG_FILE")

    # Generate slug
    SLUG=$(to_kebab_case "$DESCRIPTION")

    # Create filename
    FILENAME="task-${TASK_NUM}-${SLUG}.md"
    FILEPATH="${TASKS_DIR}/${FILENAME}"

    # Get current date
    CREATED_DATE=$(date +%Y-%m-%d)

    # Create task file
    cat > "$FILEPATH" <<EOF
---
status: pending
created: ${CREATED_DATE}
---

# Task ${TASK_NUM}: ${DESCRIPTION}

## Description

${DESCRIPTION}

## Tasks

- [ ] ${TASK_NUM}.01 [Add subtasks here]

## Rationale

[Why this task matters]
EOF

    # Output JSON result
    echo "{"
    echo "  \"task_number\": \"${TASK_NUM}\","
    echo "  \"description\": \"${DESCRIPTION}\","
    echo "  \"slug\": \"${SLUG}\","
    echo "  \"filename\": \"${FILENAME}\","
    echo "  \"filepath\": \"${FILEPATH}\","
    echo "  \"created\": \"${CREATED_DATE}\","
    echo "  \"type\": \"top-level\""
    echo "}"

else
    # Create subtask with parent.child numbering (e.g., 12.1, 12.2)

    # Pad parent number to match file naming (e.g., 12 -> 12, 3 -> 03)
    PADDED_PARENT=$(echo "$PARENT_NUM" | awk -F. '{printf "%02d", $1; for(i=2; i<=NF; i++) printf ".%s", $i}')

    # Find parent task file
    PARENT_FILE=$(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_PARENT}-*.md" -type f | head -1)

    if [[ -z "$PARENT_FILE" ]]; then
        echo "{\"error\": \"Parent task file not found for task number: $PARENT_NUM\"}" >&2
        exit 1
    fi

    # Find existing subtasks to determine next subtask number
    # Pattern: task-12.1-*.md, task-12.2-*.md, etc.
    MAX_SUBTASK=0
    while IFS= read -r -d '' file; do
        filename=$(basename "$file")
        # Extract subtask number (e.g., from task-12.1-slug.md extract "1")
        if [[ "$filename" =~ ^task-${PADDED_PARENT}\.([0-9]+)-.*\.md$ ]]; then
            subtask_num="${BASH_REMATCH[1]}"
            subtask_num=$((10#$subtask_num))  # Remove leading zeros
            if [[ $subtask_num -gt $MAX_SUBTASK ]]; then
                MAX_SUBTASK=$subtask_num
            fi
        fi
    done < <(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_PARENT}.*-*.md" -print0 2>/dev/null)

    # Next subtask number
    NEXT_SUBTASK=$((MAX_SUBTASK + 1))

    # Full task number (e.g., 12.1)
    FULL_TASK_NUM="${PARENT_NUM}.${NEXT_SUBTASK}"

    # Generate slug
    SLUG=$(to_kebab_case "$DESCRIPTION")

    # Create filename: task-12.1-slug.md
    FILENAME="task-${PADDED_PARENT}.${NEXT_SUBTASK}-${SLUG}.md"
    FILEPATH="${TASKS_DIR}/${FILENAME}"

    # Get current date
    CREATED_DATE=$(date +%Y-%m-%d)

    # Extract parent title
    PARENT_TITLE=$(grep "^# Task" "$PARENT_FILE" | head -1 | sed 's/^# Task [0-9.]*: //')

    # Create subtask file
    cat > "$FILEPATH" <<EOF
---
status: pending
created: ${CREATED_DATE}
parent: ${PARENT_NUM}
---

# Task ${FULL_TASK_NUM}: ${DESCRIPTION}

**Parent**: [Task ${PARENT_NUM}: ${PARENT_TITLE}](./$(basename "$PARENT_FILE"))

## Description

${DESCRIPTION}

## Tasks

- [ ] ${FULL_TASK_NUM}.1 [Add subtasks here]

## Notes

<!-- Add notes here -->
EOF

    # Update parent file to add this subtask to its ## Tasks section
    # Check if parent has ## Tasks section
    if ! grep -q "^## Tasks" "$PARENT_FILE"; then
        echo -e "\n## Tasks\n" >> "$PARENT_FILE"
    fi

    # Add subtask entry to parent's ## Tasks section
    # Use a temporary file for macOS compatibility
    TEMP_FILE="${PARENT_FILE}.tmp"
    awk -v subtask="- [ ] ${FULL_TASK_NUM} ${DESCRIPTION}" '
        /^## Tasks/ {
            print
            print subtask
            next
        }
        {print}
    ' "$PARENT_FILE" > "$TEMP_FILE"
    mv "$TEMP_FILE" "$PARENT_FILE"

    # Output JSON result
    echo "{"
    echo "  \"task_number\": \"${FULL_TASK_NUM}\","
    echo "  \"parent\": \"${PARENT_NUM}\","
    echo "  \"description\": \"${DESCRIPTION}\","
    echo "  \"slug\": \"${SLUG}\","
    echo "  \"filename\": \"${FILENAME}\","
    echo "  \"filepath\": \"${FILEPATH}\","
    echo "  \"created\": \"${CREATED_DATE}\","
    echo "  \"type\": \"subtask\","
    echo "  \"parent_file\": \"$(basename "$PARENT_FILE")\""
    echo "}"
fi
```

#### AI Workflow

When user requests task creation:

1. **Parse arguments**:
   - Check for `-p` flag to determine if creating subtask
   - Extract parent number if present
   - Extract task description

2. **For Top-Level Tasks** (no `-p` flag):
   - Execute bash script
   - Parse JSON output
   - Confirm task creation to user with task number and filename

3. **For Subtasks** (with `-p` flag):
   - Find parent task file
   - Read parent task to extract unchecked tasks
   - For each unchecked task, create a subtask file
   - Update parent task with proper numbering
   - Confirm subtask creation to user

### 3. List Tasks (`list-tasks`)

Display all tasks with their status and hierarchy.

#### Bash Implementation

```bash
#!/bin/bash
# list-tasks.sh - List all tasks with status and hierarchy
# Usage: bash list-tasks.sh <config_file>
# Example: bash list-tasks.sh .claude/skills/bert/skill.yml

set -e

CONFIG_FILE="$1"

if [[ -z "$CONFIG_FILE" ]]; then
    echo '{"error": "Usage: list-tasks.sh <config_file>"}' >&2
    exit 1
fi

# Read config
TASKS_DIR=$(grep "tasks_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')

# Find all task files and sort them
TASK_FILES=($(find "$TASKS_DIR" -maxdepth 1 -name "task-*.md" -type f 2>/dev/null | sort))

# Start JSON output
echo "{"
echo "  \"tasks\": ["

first=true
for task_file in "${TASK_FILES[@]}"; do
    # Extract task number and slug from filename
    # Format: task-03-slug.md or task-03.1-slug.md
    filename=$(basename "$task_file")

    # Extract number (everything between "task-" and the last "-")
    task_num=$(echo "$filename" | sed -E 's/task-([0-9.]+)-.*/\1/')

    # Extract slug (everything between last "-" and ".md")
    slug=$(echo "$filename" | sed -E 's/.*-([^-]+)\.md$/\1/')

    # Extract status from frontmatter (default to "unknown" if not found)
    status=$(grep "^status:" "$task_file" 2>/dev/null | head -1 | awk '{print $2}' || echo "unknown")

    # Extract created date from frontmatter
    created=$(grep "^created:" "$task_file" 2>/dev/null | head -1 | awk '{print $2}' || echo "")

    # Extract title from first # heading
    title=$(grep "^# " "$task_file" | head -1 | sed 's/^# //' || echo "$slug")

    # Calculate nesting level (count dots in task number)
    level=$(echo "$task_num" | tr -cd '.' | wc -c | tr -d ' ')

    # Output JSON object
    if [ "$first" = false ]; then
        echo ","
    fi
    first=false

    echo -n "    {"
    echo -n "\"number\": \"$task_num\", "
    echo -n "\"slug\": \"$slug\", "
    echo -n "\"title\": \"$title\", "
    echo -n "\"status\": \"$status\", "
    echo -n "\"created\": \"$created\", "
    echo -n "\"level\": $level, "
    echo -n "\"filename\": \"$filename\""
    echo -n "}"
done

echo ""
echo "  ],"
echo "  \"total\": ${#TASK_FILES[@]},"
echo "  \"tasks_directory\": \"$TASKS_DIR\""
echo "}"
```

#### AI Workflow

When user requests task listing (e.g., "show me all tasks", "list tasks"):

1. **Extract bash script** from the code block above
2. **Execute script**:
   ```bash
   bash -c '<script>' -- .claude/skills/bert/skill.yml
   ```
3. **Parse JSON output** from script
4. **Format for user** with:
   - Hierarchical indentation based on level
   - Status indicators (emoji or text)
   - Task numbers and titles
   - Optional: filter by status (pending, in-progress, completed)
5. **Example formatted output**:
   ```
   03 yak-news [pending] (2025-10-16)
     03.1 something [in-progress] (2025-10-17)
       03.1.1 detail [pending] (2025-10-17)
   04 sendtoq-urlbank [completed] (2025-10-16)

   Total: 4 tasks
   ```

### 4. Update Status (`update-status`)

Update the status field in a task's frontmatter.

**Valid Statuses**: pending, in-progress, completed, blocked

#### Bash Implementation

```bash
#!/bin/bash
# update-status.sh - Update task status in frontmatter
# Usage: bash update-status.sh <config_file> <task_number> <new_status>
# Example: bash update-status.sh .claude/skills/bert/skill.yml 3 completed

set -e

CONFIG_FILE="$1"
TASK_NUM="$2"
NEW_STATUS="$3"

if [[ -z "$CONFIG_FILE" ]] || [[ -z "$TASK_NUM" ]] || [[ -z "$NEW_STATUS" ]]; then
    echo '{"error": "Usage: update-status.sh <config_file> <task_number> <new_status>"}' >&2
    exit 1
fi

# Validate status
VALID_STATUSES=("pending" "in-progress" "completed" "blocked")
if [[ ! " ${VALID_STATUSES[@]} " =~ " ${NEW_STATUS} " ]]; then
    echo "{\"error\": \"Invalid status: $NEW_STATUS. Valid: ${VALID_STATUSES[*]}\"}" >&2
    exit 1
fi

# Read config
TASKS_DIR=$(grep "tasks_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')

# Pad task number to 2 digits for file matching
PADDED_NUM=$(echo "$TASK_NUM" | awk -F. '{printf "%02d", $1; for(i=2; i<=NF; i++) printf ".%s", $i}')

# Find task file
TASK_FILE=$(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_NUM}-*.md" -type f | head -1)

if [[ -z "$TASK_FILE" ]]; then
    echo "{\"error\": \"Task file not found for task number: $TASK_NUM\"}" >&2
    exit 1
fi

# Get old status
OLD_STATUS=$(grep "^status:" "$TASK_FILE" 2>/dev/null | head -1 | awk '{print $2}' || echo "unknown")

# Update status in frontmatter using sed
# macOS requires '' after -i, Linux doesn't need it
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/^status: .*/status: $NEW_STATUS/" "$TASK_FILE"
else
    sed -i "s/^status: .*/status: $NEW_STATUS/" "$TASK_FILE"
fi

# Output JSON result
echo "{"
echo "  \"task_number\": \"$TASK_NUM\","
echo "  \"filename\": \"$(basename "$TASK_FILE")\","
echo "  \"old_status\": \"$OLD_STATUS\","
echo "  \"new_status\": \"$NEW_STATUS\","
echo "  \"updated\": true"
echo "}"
```

#### AI Workflow

When user requests status update (e.g., "mark task 3 as completed", "update task 4.1 to in-progress"):

1. **Parse task number and status** from user request
2. **Validate status** value (pending, in-progress, completed, blocked)
3. **Extract bash script** from the code block above
4. **Execute script**:
   ```bash
   bash -c '<script>' -- .claude/skills/bert/skill.yml <task_number> <new_status>
   ```
5. **Parse JSON output** from script
6. **Confirm to user**:
   - "Updated task X from [old_status] to [new_status]"
   - Show filename
7. **Handle errors**: Invalid status or task not found

### 5. Archive Task (`archive-task`)

Archive a task and its associated notes by moving them to the archive directories.

**Usage Patterns**:
- "archive task 3" → Archive task-03-*.md and all subtasks (3.1, 3.2, etc.) plus their notes
- "archive task 3.1" → Archive only task-03.1-*.md and its notes (leaves parent task 3 intact)
- "archive task 3.1.2" → Archive only task-03.1.2-*.md and its notes

#### Bash Implementation

```bash
#!/bin/bash
# archive-task.sh - Archive a task and its associated notes
# Usage: bash archive-task.sh <config_file> <task_number>
# Example: bash archive-task.sh .claude/skills/bert/skill.yml 3

set -e

CONFIG_FILE="$1"
TASK_NUM="$2"

if [[ -z "$CONFIG_FILE" ]] || [[ -z "$TASK_NUM" ]]; then
    echo '{"error": "Usage: archive-task.sh <config_file> <task_number>"}' >&2
    exit 1
fi

# Read config values using grep and awk
TASKS_DIR=$(grep "tasks_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')
NOTES_DIR=$(grep "notes_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')
ARCHIVE_TASKS=$(grep "archive_tasks_directory:" "$CONFIG_FILE" | awk '{print $2}')
ARCHIVE_NOTES=$(grep "archive_notes_directory:" "$CONFIG_FILE" | awk '{print $2}')

# Create archive directories
mkdir -p "$ARCHIVE_TASKS" "$ARCHIVE_NOTES"

# Pad task number to 2 digits for file matching (e.g., 3 -> 03, 3.1 -> 03.1)
PADDED_NUM=$(echo "$TASK_NUM" | awk -F. '{printf "%02d", $1; for(i=2; i<=NF; i++) printf ".%s", $i}')

# Determine pattern based on whether it's a parent or subtask
# For parent (e.g., "03"): match "task-03-*.md" AND "task-03.*.md" (parent + all subtasks)
# For subtask (e.g., "03.1"): match "task-03.1-*.md" AND "task-03.1.*.md" (subtask + its children only)

# Find and move task files
TASK_FILES=()

if [[ "$TASK_NUM" =~ \. ]]; then
    # Subtask: match the subtask itself AND its children
    # Example: 03.1 matches task-03.1-*.md and task-03.1.*.md
    # First, find the exact subtask file (task-03.1-slug.md)
    while IFS= read -r -d '' file; do
        TASK_FILES+=("$file")
    done < <(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_NUM}-*.md" -print0 2>/dev/null)

    # Then find all its children (task-03.1.*.md)
    while IFS= read -r -d '' file; do
        TASK_FILES+=("$file")
    done < <(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_NUM}.*.md" -print0 2>/dev/null)
else
    # Parent: match the parent file itself (task-03-slug.md)
    while IFS= read -r -d '' file; do
        TASK_FILES+=("$file")
    done < <(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_NUM}-*.md" -print0 2>/dev/null)

    # Then find all its subtasks (task-03.*.md)
    while IFS= read -r -d '' file; do
        TASK_FILES+=("$file")
    done < <(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_NUM}.*.md" -print0 2>/dev/null)
fi

# Move task files
MOVED_TASKS=()
for task_file in "${TASK_FILES[@]}"; do
    filename=$(basename "$task_file")
    mv "$task_file" "$ARCHIVE_TASKS/"
    MOVED_TASKS+=("$filename")
done

# Find and move associated notes
# Pattern: any file in notes directory containing the task number
NOTE_PATTERN="*task-${PADDED_NUM}*.md"
NOTE_FILES=()
while IFS= read -r -d '' file; do
    NOTE_FILES+=("$file")
done < <(find "$NOTES_DIR" -maxdepth 1 -name "$NOTE_PATTERN" -print0 2>/dev/null)

# Move notes files
MOVED_NOTES=()
for note_file in "${NOTE_FILES[@]}"; do
    filename=$(basename "$note_file")
    mv "$note_file" "$ARCHIVE_NOTES/"
    MOVED_NOTES+=("$filename")
done

# Output JSON result
echo "{"
echo "  \"task_number\": \"$TASK_NUM\","
echo "  \"tasks_archived\": ${#MOVED_TASKS[@]},"
echo "  \"notes_archived\": ${#MOVED_NOTES[@]},"
echo "  \"archived_tasks\": ["
for i in "${!MOVED_TASKS[@]}"; do
    echo -n "    \"${MOVED_TASKS[$i]}\""
    [[ $i -lt $((${#MOVED_TASKS[@]} - 1)) ]] && echo "," || echo
done
echo "  ],"
echo "  \"archived_notes\": ["
for i in "${!MOVED_NOTES[@]}"; do
    echo -n "    \"${MOVED_NOTES[$i]}\""
    [[ $i -lt $((${#MOVED_NOTES[@]} - 1)) ]] && echo "," || echo
done
echo "  ],"
echo "  \"archive_tasks_directory\": \"$ARCHIVE_TASKS\","
echo "  \"archive_notes_directory\": \"$ARCHIVE_NOTES\""
echo "}"
```

#### AI Workflow

When user requests archiving (e.g., "archive task 3"):

1. **Parse task number** from user request (e.g., "3", "3.1", "3.1.2")
2. **Extract bash script** from the code block above
3. **Execute script**:
   ```bash
   bash -c '<script>' -- .claude/skills/bert/skill.yml <task_number>
   ```
4. **Parse JSON output** from script
5. **Confirm to user**:
   - "Archived task X with Y subtasks"
   - List archived files
   - Display archive directory paths
6. **Handle errors**: If script returns error, explain to user

**Pattern Matching Examples**:
- Task `3` → Match: `task-03-*.md`, `task-03.1-*.md`, `task-03.1.1-*.md`, `task-03.2-*.md`
- Task `3.1` → Match: `task-03.1-*.md`, `task-03.1.1-*.md`, `task-03.1.2-*.md` (NOT `task-03-*.md` or `task-03.2-*.md`)
- Task `3.1.2` → Match: `task-03.1.2-*.md`, `task-03.1.2.1-*.md` (NOT `task-03.1-*.md` or `task-03.1.1-*.md`)

### 6. Archive Spec (`archive-spec`)

Archive a spec directory along with all its tasks and review files.

**Usage Patterns**:
- "archive spec 12" → Archive spec-12 directory and all task-12.x files
- "archive spec 12 --tasks-only" → Archive only tasks, keep spec

#### Bash Implementation

```bash
#!/bin/bash
# archive-spec.sh - Archive a spec and optionally its tasks
# Usage: bash archive-spec.sh <config_file> <spec_number> [--tasks-only]
# Example: bash archive-spec.sh .claude/skills/bert/skill.yml 12
# Example: bash archive-spec.sh .claude/skills/bert/skill.yml 12 --tasks-only

set -e

CONFIG_FILE="$1"
SPEC_NUM="$2"
TASKS_ONLY=false

if [[ "$3" == "--tasks-only" ]]; then
    TASKS_ONLY=true
fi

if [[ -z "$CONFIG_FILE" ]] || [[ -z "$SPEC_NUM" ]]; then
    echo '{"error": "Usage: archive-spec.sh <config_file> <spec_number> [--tasks-only]"}' >&2
    exit 1
fi

# Read config
SPECS_DIR=$(grep "specs_directory:" "$CONFIG_FILE" | awk '{print $2}')
TASKS_DIR=$(grep "tasks_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')
NOTES_DIR=$(grep "notes_directory:" "$CONFIG_FILE" | grep -v "archive" | awk '{print $2}')
ARCHIVE_SPECS=$(grep "archive_specs_directory:" "$CONFIG_FILE" | awk '{print $2}')
ARCHIVE_TASKS=$(grep "archive_tasks_directory:" "$CONFIG_FILE" | awk '{print $2}')
ARCHIVE_NOTES=$(grep "archive_notes_directory:" "$CONFIG_FILE" | awk '{print $2}')

# Create archive directories
mkdir -p "$ARCHIVE_SPECS" "$ARCHIVE_TASKS" "$ARCHIVE_NOTES"

# Pad spec number to 2 digits
PADDED_NUM=$(printf "%02d" "$SPEC_NUM")

# Archive spec directory (unless --tasks-only)
SPEC_DIR="${SPECS_DIR}/spec-${PADDED_NUM}"
ARCHIVED_SPEC=false

if [[ "$TASKS_ONLY" == false ]]; then
    if [[ -d "$SPEC_DIR" ]]; then
        mv "$SPEC_DIR" "$ARCHIVE_SPECS/"
        ARCHIVED_SPEC=true
    else
        echo "{\"error\": \"Spec directory not found: $SPEC_DIR\"}" >&2
        exit 1
    fi
fi

# Archive all tasks for this spec (task-{PADDED_NUM}.*.md)
TASK_FILES=()

# Find parent task file (might not exist for spec-based tasks)
while IFS= read -r -d '' file; do
    TASK_FILES+=("$file")
done < <(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_NUM}-*.md" -print0 2>/dev/null)

# Find all subtasks (task-{PADDED_NUM}.*.md)
while IFS= read -r -d '' file; do
    TASK_FILES+=("$file")
done < <(find "$TASKS_DIR" -maxdepth 1 -name "task-${PADDED_NUM}.*.md" -print0 2>/dev/null)

# Move task files
MOVED_TASKS=()
for task_file in "${TASK_FILES[@]}"; do
    filename=$(basename "$task_file")
    mv "$task_file" "$ARCHIVE_TASKS/"
    MOVED_TASKS+=("$filename")
done

# Find and move associated notes
NOTE_PATTERN="*task-${PADDED_NUM}*.md"
NOTE_FILES=()
while IFS= read -r -d '' file; do
    NOTE_FILES+=("$file")
done < <(find "$NOTES_DIR" -maxdepth 1 -name "$NOTE_PATTERN" -print0 2>/dev/null)

# Move notes files
MOVED_NOTES=()
for note_file in "${NOTE_FILES[@]}"; do
    filename=$(basename "$note_file")
    mv "$note_file" "$ARCHIVE_NOTES/"
    MOVED_NOTES+=("$filename")
done

# Output JSON result
echo "{"
echo "  \"spec_number\": \"$SPEC_NUM\","
echo "  \"spec_archived\": $ARCHIVED_SPEC,"
echo "  \"tasks_archived\": ${#MOVED_TASKS[@]},"
echo "  \"notes_archived\": ${#MOVED_NOTES[@]},"
if [[ "$ARCHIVED_SPEC" == true ]]; then
    echo "  \"archived_spec\": \"spec-${PADDED_NUM}\","
fi
echo "  \"archived_tasks\": ["
for i in "${!MOVED_TASKS[@]}"; do
    echo -n "    \"${MOVED_TASKS[$i]}\""
    [[ $i -lt $((${#MOVED_TASKS[@]} - 1)) ]] && echo "," || echo
done
echo "  ],"
echo "  \"archived_notes\": ["
for i in "${!MOVED_NOTES[@]}"; do
    echo -n "    \"${MOVED_NOTES[$i]}\""
    [[ $i -lt $((${#MOVED_NOTES[@]} - 1)) ]] && echo "," || echo
done
echo "  ],"
echo "  \"archive_specs_directory\": \"$ARCHIVE_SPECS\","
echo "  \"archive_tasks_directory\": \"$ARCHIVE_TASKS\","
echo "  \"archive_notes_directory\": \"$ARCHIVE_NOTES\""
echo "}"
```

#### AI Workflow

When user requests spec archiving (e.g., "archive spec 12"):

1. **Parse spec number and options** from user request
2. **Extract bash script** from the code block above
3. **Execute script**:
   ```bash
   bash -c '<script>' -- .claude/skills/bert/skill.yml <spec_number> [--tasks-only]
   ```
4. **Parse JSON output** from script
5. **Confirm to user**:
   - "Archived spec X with Y tasks"
   - List archived files
   - Display archive directory paths
6. **Handle errors**: If script returns error, explain to user

**Archive Behavior**:

- `archive spec 12` → Archives spec-12 directory AND all task-12.x files
- `archive spec 12 --tasks-only` → Archives only task files, keeps spec directory (for living documentation)

## Task Execution and Review Workflow

### After Completing Tasks

When you finish executing a task or set of tasks (e.g., after completing tasks 1.1 through 1.8), you MUST automatically generate a review file:

**Review File Naming Logic**:
- Tasks `1.1, 1.2, 1.3` → `task-01-review.md` (one review for all `1.x` tasks)
- Tasks `1.1.1, 1.1.2` → `task-01.1-review.md` (review for all `1.1.x` tasks)
- Task `2` → `task-02-review.md`

**Review File Structure**:

```markdown
# Task {NN}: {Title} - Review

**Created**: YYYY-MM-DD
**Status**: In Review

---

## Implementation Summary

### Completed Tasks
- [ ] {NN}.1: {Subtask title}
- [ ] {NN}.2: {Subtask title}

### Files Changed
- path/to/file1.ts
- path/to/file2.tsx

### Implementation Notes
{Summary of what was built, key decisions, patterns used}

---

## Testing Checklist

- [ ] Build succeeds without errors
- [ ] Code follows project conventions
- [ ] No console errors or warnings
- [ ] Tested on desktop browser
- [ ] Tested on mobile browser
- [ ] Edge cases handled
- [ ] Error states implemented

---

## Issues Found

<!-- User adds issues here during testing -->

---

## Final Status

- [ ] Ready for production
- [ ] Needs iteration
- [ ] Blocked (explain below)

**Notes**:
{Any final notes, known limitations, future improvements}
```

**Your Workflow After Task Completion**:

1. **Create review file** at `docs/bert/tasks/task-{NN}-review.md`
2. **Fill in Implementation Summary**:
   - List all completed subtasks with checkboxes
   - List all files you created or modified
   - Summarize what was built and key decisions made
3. **Inform user**:
   - "Created review file at: docs/bert/tasks/task-{NN}-review.md"
   - "Please test and add any issues you find to the Issues Found section"
   - "When done, tell me: 'added notes to task-{NN}-review.md'"

**User's Review Process**:

1. User reads the review file to understand what was done
2. User tests the implementation
3. User finds issues and adds them to "Issues Found" section:
   ```markdown
   ### Issue 1: Description (YYYY-MM-DD)
   **Reporter**: User
   **Status**: Open

   Detailed description of the issue...
   ```
4. User tells you: "added notes to task-{NN}-review.md"

**Your Fix Process**:

1. Read the review file
2. See the issues user added
3. Fix each issue
4. Add your fix notes directly below each issue:
   ```markdown
   ### Issue 1: Description (YYYY-MM-DD)
   **Reporter**: User
   **Status**: Fixed

   Detailed description of the issue...

   **Fix**: {Explain what you changed to fix it}
   ```
5. Update issue status from "Open" to "Fixed"
6. Tell user what you fixed

**Iteration**:
- Repeat this process until user checks "Ready for production" in Final Status
- User can add multiple issues at once
- You fix them and document all fixes in the same review file

## User Interaction Patterns

When users say:
- "create a task for [file]" → Use task-author
- "analyze [file] for improvements" → Use task-author
- "create subtasks for task 4" → Use task-create with parent 4
- "expand task 4.1" → Use task-create with parent 4.1
- "show me all tasks" → Use list-tasks
- "mark task 4 as completed" → Use update-status
- "archive task 3" → Use archive-task with task number 3 (archives parent + all subtasks)
- "archive task 3.1" → Use archive-task with task number 3.1 (archives only this subtask)
- "archive spec 12" → Use archive-spec to archive spec-12 directory and all task-12.x files
- "archive spec 12 tasks only" → Use archive-spec with --tasks-only flag (keeps spec, archives tasks)
- "added notes to task-{NN}-review.md" → Read review file, fix issues, update with fix notes

## Key Behaviors

1. **Always read config**: Determine all directories from `.claude/skills/bert/skill.yml` (tasks, notes, archive)
2. **Smart numbering**: Apply padding based on actual task count
3. **Arbitrary depth**: Support unlimited nesting (4.1.2.3.4...)
4. **Preserve structure**: Use dots for hierarchy, maintain frontmatter
5. **Clear errors**: If files missing or invalid format, explain clearly
6. **User confirmation**: For abstract searches, confirm before proceeding
7. **Archive scope**: Parent task archives include all subtasks; subtask archives are isolated

## Session Context

Bert is now **ACTIVE** for this session. You can use natural language to invoke bert operations:
- "Create a task to improve the authentication system"
- "Analyze docs/roadmap.md and create improvement tasks"
- "Break down task 3 into subtasks"
- "Show me all pending tasks"
- "Mark task 4 as completed"
- "Archive task 3" (archives task 3 and all its subtasks with notes)
- "Archive task 3.1" (archives only subtask 3.1 with its notes)

Ready to manage your tasks with bert.

## Archive Directory Structure

When tasks are archived, they maintain their original filenames in the archive directories:

```
docs/bert/
├── tasks/                           # Active tasks
│   ├── task-01-gather-steps.md
│   ├── task-02-exe-web.md
│   └── ...
├── notes/                           # Active notes
│   ├── task-01.md
│   ├── 2025-10-17-task-07.1.md
│   └── ...
└── archive/
    ├── tasks/                       # Archived tasks
    │   ├── task-03-yak-news.md      # Archived parent
    │   ├── task-03.1-something.md   # Archived subtask
    │   └── ...
    └── notes/                       # Archived notes
        ├── task-03.md               # Archived note for task 3
        ├── 2025-10-17-task-03.md    # Archived dated note
        └── ...
```

**Archive Behavior Examples**:

1. **Archive parent task 3**:
   - Moves: `task-03-*.md`, `task-03.1-*.md`, `task-03.1.1-*.md`, `task-03.2-*.md`
   - From: `docs/bert/tasks/`
   - To: `docs/bert/archive/tasks/`
   - Plus all associated notes from `docs/bert/notes/` to `docs/bert/archive/notes/`

2. **Archive subtask 3.1**:
   - Moves: `task-03.1-*.md`, `task-03.1.1-*.md`, `task-03.1.2-*.md` (only children)
   - Does NOT move: `task-03-*.md`, `task-03.2-*.md` (parent and siblings stay active)
   - Plus associated notes for 3.1 and its children

3. **Archive subtask 3.1.2**:
   - Moves: `task-03.1.2-*.md`, `task-03.1.2.1-*.md` (only this subtask and descendants)
   - Does NOT move: `task-03-*.md`, `task-03.1-*.md`, `task-03.1.1-*.md` (ancestors and siblings stay active)
   - Plus associated notes for 3.1.2 and its children

---

## Spec-Driven Development (NEW)

**Adapted from Agent-OS by Brian Casel @ Builder Methods**

Bert now includes intelligent requirements gathering, specification writing, and task decomposition capabilities. These are **optional** features accessed via `/bert:spec` and `/bert:plan` commands that add structured planning workflows for complex features.

### When to Use Specs

**Use `/bert:spec` for**:
- Complex features requiring detailed planning
- Features where you want to think through requirements before coding
- Work that needs visual mockups or design thinking
- Projects that will span multiple tasks
- When you want AI to help break down the work

**Skip `/bert:spec` for**:
- Simple bug fixes or small changes
- Well-understood tasks you can implement directly
- Quick experiments or prototypes
- Ad-hoc tasks (use `/bert:task create` directly)

### Specs vs Tasks

**Specs** are numbered (spec-12, spec-13) and live in `docs/bert/specs/`:
- Each spec has its own directory: `docs/bert/specs/spec-{nn}/`
- Contains: requirements.md, spec.md, tasks-proposal.md, visuals/

**Tasks** created from specs use the spec number as prefix:
- Spec 12 → tasks 12.1, 12.2, 12.3, etc.
- Still live in `docs/bert/tasks/` as `task-12.1-{slug}.md`
- Link back to their parent spec

**Ad-hoc tasks** created without specs continue as normal:
- `/bert:task create "fix login bug"` → task-15-fix-login-bug.md (no spec)

### Bert Commands for Spec Development

See command files in `.claude/commands/bert/` for complete documentation.

**`/bert:plan`** - Initialize product context (optional):
- Creates mission.md, roadmap.md, tech-stack.md templates
- Helps AI understand your project better
- Completely optional

**`/bert:spec`** - Specification development:
- `/bert:spec new <description>` - Start new spec with requirements
- `/bert:spec iterate <number>` - Smart iteration (requirements OR spec feedback)
- `/bert:spec tasks <number>` - Create Bert task files

**`/bert:task`** - Task execution:
- `/bert:task create <description>` - Create task
- `/bert:task execute <number>` - Implement task
- `/bert:task status <number> <status>` - Update status
- `/bert:task list [filter]` - List tasks
- `/bert:task archive <number>` - Archive completed task

### Complete Spec Workflow Example

```bash
# Step 1: (Optional) Setup product context
/bert:plan
# → Creates docs/bert/product/ with templates
[Fill out mission.md, roadmap.md, tech-stack.md]

# Step 2: Start a new spec
/bert:spec new "user authentication system"
# → Creates docs/bert/specs/spec-12/requirements.md

# Step 3: Answer requirements questions
[Open docs/bert/specs/spec-12/requirements.md]
[Fill in your answers over multiple sessions]
[Optionally add mockups to visuals/]

# Step 4: First iteration
/bert:spec iterate 12
# → AI reads answers, may add follow-ups OR generate spec.md

# Step 5: If follow-ups added, answer and iterate again
[Answer follow-ups in requirements.md]
/bert:spec iterate 12
# → AI generates spec.md

# Step 6: Review spec, add feedback if needed
[Open docs/bert/specs/spec-12/spec.md]
[Add feedback in feedback section]
/bert:spec iterate 12
# → AI updates spec based on feedback

# Step 7: Repeat step 6 until satisfied
[Keep iterating until spec looks good]

# Step 8: Create tasks
/bert:spec tasks 12
# → Proposes task breakdown, creates task-12.1.md, task-12.2.md, etc.

# Step 9: (Optional) If spec incomplete after seeing tasks
[Edit spec.md]
/bert:spec iterate 12  # Regenerate spec
/bert:spec tasks 12    # Regenerate tasks

# Step 10: Execute tasks
/bert:task execute 12.1
/bert:task execute 12.2
# [Standard Bert implementation workflow]
```

### Spec Directory Structure

```
docs/bert/
├── specs/
│   └── spec-12/                    # Spec directory
│       ├── requirements.md         # Q&A with user
│       ├── spec.md                 # Technical specification
│       ├── tasks-proposal.md       # Proposed task breakdown
│       └── visuals/                # Optional mockups/wireframes
│           ├── login-mockup.png
│           └── auth-flow.pdf
└── tasks/
    ├── task-12.1-database-schema.md      # Created from spec
    ├── task-12.2-api-endpoints.md        # Created from spec
    ├── task-12.3-frontend-components.md  # Created from spec
    └── task-15-fix-login-bug.md          # Ad-hoc (no spec)
```

### Agents Used

Three specialized agents power the spec workflow:

**requirements-gatherer** (`.claude/skills/bert/agents/requirements-gatherer.md`):
- Invoked by: `/bert:spec new`
- Generates targeted questions about the feature
- Creates file-based Q&A document
- Checks for product context
- Creates visuals/ directory

**spec-iterator** (`.claude/skills/bert/agents/spec-iterator.md`):
- Invoked by: `/bert:spec iterate`
- Smart detection of current state
- Adds follow-up questions to requirements
- Generates spec.md from requirements
- Updates spec.md based on feedback
- Searches codebase for reusable components

**task-proposer** (`.claude/skills/bert/agents/task-proposer.md`):
- Invoked by: `/bert:spec tasks`
- Analyzes specification
- Proposes dependency-aware task breakdown
- Creates Bert task files with spec links

All agents read paths from `.claude/skills/bert/skill.yml` config.

### Product Context (Optional)

You can create product context files to help AI understand your project:

```bash
/bert:plan
```

Creates template files in `docs/bert/product/`:
- `mission.md` - Product mission and goals
- `roadmap.md` - Completed features and future plans
- `tech-stack.md` - Technologies and frameworks

If these files exist, agents will reference them when gathering requirements and generating specs.

### No Lock-In

**Specs are completely optional**:
- Create ad-hoc tasks with `/bert:task create` anytime
- Mix spec-based and ad-hoc tasks in same project
- Start with ad-hoc, add spec later if needed

**You control the process**:
- Review at every step
- Modify files directly
- Provide feedback and iterate as many times as needed
- No "magic black box" automation

### Integration with Standard Bert

**Spec operations work alongside task operations**:
- `/bert:plan` for product context (optional)
- `/bert:spec` for planning (requirements → specification → tasks)
- `/bert:task` for execution (create, execute, status, archive)
- Tasks from specs are standard Bert tasks
- Same `/bert:task execute`, `/bert:task status`, `/bert:task archive` commands

### Acknowledgments

The requirements gathering, spec writing, and task decomposition workflows are adapted from [Agent-OS](https://github.com/builder-methods/agent-os) by Brian Casel @ Builder Methods. Original work licensed under ISC License.

Bert's adaptations include:
- File-based Q&A pattern (vs CLI prompts)
- Simplified workflow (3 commands vs 7)
- Smart iteration command (auto-detects state)
- Integration with Bert task system
- Config-driven paths
- Numbered specs linked to tasks
- Three-command structure (/bert:plan, /bert:spec, /bert:task)