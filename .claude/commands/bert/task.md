Activate the bert skill and route the subcommand.

**Arguments**: {{args}}

## Routing Instructions

Parse the arguments to determine which bert operation to execute:

### Subcommands

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
2. **Validate subcommand**: Ensure it matches create|archive|list|status|help
3. **Extract parameters**: Get task_number, status, filter, parent flag, description as needed
4. **Route to operation**: Execute the appropriate bert skill operation
5. **Format results**: Parse JSON output and display user-friendly results

### Error Handling

If no arguments provided or subcommand not recognized, display:

```
Bert Task Management

Usage:
  /bert create [-p <parent>] <description>  Create a new task or subtask
  /bert archive <task_number>               Archive a task and its notes
  /bert list [filter]                       List all tasks (optionally filter by status)
  /bert status <task_number> <status>       Update task status

Examples:
  /bert create "article scorer"
  /bert create -p 3 "foo"
  /bert archive 3
  /bert list
  /bert list pending
  /bert status 3 completed

Valid statuses: pending, in-progress, completed, blocked

Tip: You can also use natural language:
  "create a task for article scoring"
  "create subtasks for task 3"
  "archive task 3"
  "show me all tasks"
  "mark task 3 as completed"
```

## Important

You MUST use the embedded bash scripts from the bert skill.md file for all operations.
DO NOT use native tools (Read, Edit, Glob, etc.) - use only the bash script implementations.
