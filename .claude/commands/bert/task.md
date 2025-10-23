Activate the bert skill and route the subcommand.

**Arguments**: {{args}}

## Routing Instructions

Parse the arguments to determine which bert operation to execute:

### Subcommands

**execute <task_number> [to <task_number>]**
- Example: `/bert:task execute 1.1` - Execute a single task
- Example: `/bert:task execute 1.1 to 1.14` - Execute a range of tasks
- Example: `/bert:task execute 1.1 1.3 1.5` - Execute multiple specific tasks
- Route to: task-execute operation (implemented below)
- Reads task file, presents objective and scope, executes implementation
- Automatically updates status (pending → in-progress → completed)
- Handles dependencies (warns if dependencies not completed)

**create [-p <parent_number>] <description>**
- Example: `/bert create "article scorer"` - Create new top-level task
- Example: `/bert create -p 3 "foo"` - Create subtask under parent task 3
- Route to: task-create operation
- Extract: optional -p flag with parent number, task description
- Next task number determined by scanning BOTH tasks/ and archive/tasks/ directories

**archive <task_number>**
- Example: `/bert archive 3`
- Route to: archive-task operation
- Extract task_number from first argument after subcommand

**list [filter]**
- Example: `/bert list`
- Example: `/bert list pending`
- Route to: list-tasks operation
- Optional filter parameter for status filtering

**status <task_number> <new_status>**
- Example: `/bert status 3 completed`
- Route to: update-status operation
- Extract task_number (first arg) and new_status (second arg)

**help**
- Example: `/bert help`
- Display available operations and usage

### Execution Flow

1. **Parse subcommand**: Extract first word from arguments
2. **Validate subcommand**: Ensure it matches execute|create|archive|list|status|help
3. **Extract parameters**: Get task_number(s), status, filter, parent flag, description as needed
4. **Route to operation**: Execute the appropriate bert skill operation
5. **Format results**: Parse JSON output and display user-friendly results

### Error Handling

If no arguments provided or subcommand not recognized, display:

```
Bert Task Management

Usage:
  /bert:task execute <task> [to <task>]     Execute one or more tasks
  /bert:task create [-p <parent>] <desc>    Create a new task or subtask
  /bert:task archive <task_number>          Archive a task and its notes
  /bert:task list [filter]                  List all tasks (optionally filter by status)
  /bert:task status <task_number> <status>  Update task status

Examples:
  /bert:task execute 1.1                    Execute single task
  /bert:task execute 1.1 to 1.14            Execute range of tasks
  /bert:task execute 1.1 1.3 1.5            Execute specific tasks
  /bert:task create "article scorer"
  /bert:task create -p 3 "foo"
  /bert:task archive 3
  /bert:task list
  /bert:task list pending
  /bert:task status 3 completed

Valid statuses: pending, in-progress, completed, blocked

Tip: You can also use natural language:
  "execute task 1.1"
  "execute all tasks from 1.1 to 1.14"
  "create a task for article scoring"
  "create subtasks for task 3"
  "archive task 3"
  "show me all tasks"
  "mark task 3 as completed"
```

## Important

You MUST use the embedded bash scripts from the bert skill.md file for all operations.
DO NOT use native tools (Read, Edit, Glob, etc.) - use only the bash script implementations.

---

## Execute Operation Implementation

When the user runs `/bert:task execute <task_number>` or `/bert:task execute <start> to <end>`:

### Step 1: Parse Task Numbers

Parse the arguments to extract task numbers:
- **Single task**: `execute 1.1` → [01.1]
- **Range**: `execute 1.1 to 1.14` → [01.1, 01.2, ..., 01.14]
- **Multiple**: `execute 1.1 1.3 1.5` → [01.1, 01.3, 01.5]

Normalize task numbers to 2-digit format (e.g., "1.1" → "01.1").

### Step 2: Find Task Files

For each task number, find the corresponding task file:
```bash
# Example: Find task file for 01.1
task_file=$(ls docs/bert/tasks/task-01.1-*.md 2>/dev/null | head -1)
```

If task file not found, report error and skip to next task.

### Step 3: Read Task File

For each task file found:
1. Read the full task file using the Read tool
2. Extract from frontmatter:
   - Current status
   - Dependencies array
   - Complexity level
   - Spec reference
3. Extract from body:
   - Objective
   - Scope (deliverables)
   - Technical approach
   - Success criteria
   - Implementation notes

### Step 4: Check Dependencies

Before executing each task:
1. Parse the dependencies array from frontmatter
2. For each dependency (e.g., "01.2"), find and read that task's file
3. Check the status of each dependency
4. If any dependency is not "completed", warn the user:
   ```
   ⚠️  Warning: Task 01.3 depends on:
   - Task 01.1: pending (not completed)

   Recommendation: Complete dependencies first, or proceed with caution.
   ```
5. Ask user if they want to continue despite unmet dependencies

### Step 5: Execute Task

For each task to execute:

1. **Update status to in-progress**:
   - Use Edit tool to change frontmatter `status: pending` → `status: in-progress`

2. **Present task details**:
   ```
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   Executing Task 01.1: Create New Directory Structure
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   Objective: [from task file]

   Scope:
   - [deliverable 1]
   - [deliverable 2]
   - [deliverable 3]

   Complexity: Low
   Dependencies: None

   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   ```

3. **Implement the task**:
   - Follow the Technical Approach section
   - Execute the work described in Scope
   - Reference Reusable Components if applicable
   - Apply Implementation Notes
   - Use TodoWrite to track sub-steps within the task if complex

4. **Verify success criteria**:
   - Check each success criterion from the task file
   - Report what was completed
   - Note any issues or deviations

5. **Update status to completed**:
   - Use Edit tool to change frontmatter `status: in-progress` → `status: completed`
   - Add completion timestamp to frontmatter: `completed: YYYY-MM-DD`

6. **Report completion**:
   ```
   ✅ Task 01.1 completed successfully

   Completed:
   - [deliverable 1] ✓
   - [deliverable 2] ✓
   - [deliverable 3] ✓

   Success criteria verified:
   - [criterion 1] ✓
   - [criterion 2] ✓
   ```

### Step 6: Handle Multiple Tasks

When executing multiple tasks (range or list):

1. **Process in order**: Execute tasks sequentially in the order specified
2. **Stop on error**: If a task fails or cannot be completed, stop and report
3. **Track progress**: Use TodoWrite to show progress across all tasks
4. **Summary report**: After all tasks, show summary:
   ```
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
   Execution Summary
   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

   Completed: 5 tasks
   Failed: 1 task
   Skipped: 0 tasks

   ✅ Task 01.1: Create New Directory Structure
   ✅ Task 01.2: Migrate Exa-Search Source Files
   ✅ Task 01.3: Migrate Prompt-Search/Flib Files
   ✅ Task 01.4: Migrate Shared Resources
   ✅ Task 01.5: Migrate Scripts and Utilities
   ❌ Task 01.6: Update TypeScript Configuration (error: tsconfig.json not found)

   Next steps:
   - Fix issues with Task 01.6
   - Continue with Task 01.7 after 01.6 is resolved
   ```

### Step 7: Range Expansion Algorithm

To expand ranges like "1.1 to 1.14":

```
1. Parse start and end (e.g., "1.1" and "1.14")
2. Normalize to 2-digit format ("01.1" and "01.14")
3. Extract parent and sub numbers:
   - Parent: 01, Sub-start: 1, Sub-end: 14
4. Generate list: 01.1, 01.2, 01.3, ..., 01.14
5. Return array of task numbers
```

For nested tasks (e.g., "1.2.1 to 1.2.5"):
```
1. Parse: Parent=01, Sub=2, Subsub-start=1, Subsub-end=5
2. Generate: 01.2.1, 01.2.2, 01.2.3, 01.2.4, 01.2.5
```

### Key Behaviors

- **Sequential execution**: Tasks execute one at a time, in order
- **Status tracking**: Automatically updates task status throughout
- **Dependency checking**: Warns about unmet dependencies
- **Detailed reporting**: Shows what was done for each task
- **Error handling**: Stops on errors, provides clear error messages
- **Range support**: Handles "X to Y" syntax for task ranges
- **Multiple task support**: Handles space-separated task numbers
