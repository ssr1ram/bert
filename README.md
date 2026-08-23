# Bert - Build, Execute, and Refine Tasks

## version 0.3.3

Task management for Claude Code with optional spec-driven development, built-in review workflow, and Rust CLI.

## Quick Start

```bash
# Install: Copy files to your Claude Code project (see Installation)

# Basic task workflow
/bert:task create "add feature"
/bert:task execute 15
# → AI creates task-15-review.md
# → You test, add issues to review file
# → Tell AI: "added notes to task-15-review.md"
# → AI fixes issues, documents fixes

# Spec-driven workflow (complex features)
/bert:spec new "user authentication"
/bert:spec iterate 12
/bert:spec tasks 12
/bert:task execute 12.1
# → Same review workflow
```

## What is Bert?

- **Task management** - Create, track, archive hierarchical tasks
- **Review workflow** - AI auto-generates review files, you add issues, AI fixes and documents
- **Spec development** - Optional requirements + spec writing for complex features
- **Product context** - Optional mission/roadmap/tech-stack for better AI assistance
- **Rust CLI** - Optional standalone CLI for task operations (complements Claude Code integration)

## Installation

### Quick Install

Run this one-line command from your target repository:

```bash
curl -fsSL https://raw.githubusercontent.com/ssr1ram/bert/main/scripts/base-install.sh | bash
```

This will download and install all Bert files to your `.claude/` directory.

### Manual Installation

Copy these files to your Claude Code project:

#### File Locations

```
your-project/
├── .claude/
│   ├── commands/bert/
│   │   ├── plan.md          # /bert:plan command
│   │   ├── spec.md          # /bert:spec command
│   │   └── task.md          # /bert:task command
│   └── skills/bert/
│       ├── agents/
│       │   ├── requirements-gatherer.md
│       │   ├── spec-iterator.md
│       │   └── task-proposer.md
│       ├── skill.md         # Main skill definition
│       └── skill.yml        # Configuration
```

#### Directories (auto-created)

```
your-project/
└── docs/bert/
    ├── specs/              # Specs (auto-created)
    ├── tasks/              # Tasks (auto-created)
    ├── notes/              # Notes (auto-created)
    ├── product/            # Product context (optional)
    └── archive/            # Archives (auto-created)
```

#### Configuration

Edit `.claude/skills/bert/skill.yml` if needed:

```yaml
config:
  tasks_directory: docs/bert/tasks
  specs_directory: docs/bert/specs
  product_directory: docs/bert/product
  # ... archive paths
```

#### Universal Numbering

**Tasks and specs share a unified number sequence** to prevent collisions. The system scans both active and archived tasks/specs to determine the next available number.

Example: If you have `task-01`, `task-02`, `spec-03`, the next task or spec will be numbered `04`. This prevents conflicts where `spec-01` would create `task-01.1` colliding with existing `task-01`.

### Rust CLI (Optional)

A standalone Rust CLI is available for faster task operations. It complements (doesn't replace) the Claude Code integration.

**Installation:**

```bash
# From the bert directory
cargo install --path .
```

This installs the `bert` binary to `~/.cargo/bin/bert`.

**Usage:**

```bash
# Create task stubs
bert task stub "add new feature"
bert task stub -p 3 "subtask under task 3"

# Archive tasks
bert task archive 08              # Archive single task
bert task archive 08 -r           # Archive with all children

# List tasks (filters understand status synonyms: open≈todo, paused/deferred≈parked)
bert task list                    # All tasks, aligned table + summary
bert task list --status parked --track newsletter
bert task list --priority p1 --json

# Adopt an existing tasks directory's format into .bert/config.yml
bert task adopt

# Help
bert --help
bert task --help
```

**When to use:**
- **Rust CLI**: Quick task stub creation and archiving from command line
- **Claude Code** (`/bert:task`): Full workflow with AI execution, review generation, status updates

The CLI is zero-config: it finds the repo root via git and puts tasks in `<repo_root>/docs/tasks`. Override via `.bert/config.yml` (same schema as the legacy `skills.yml`), the `--reporoot` / `--taskdir` flags, or a legacy `skills.yml` at the project root — all still honored.

The CLI also tolerates existing task files in other conventions (e.g. bare `task-013.md` with rich frontmatter): it reads them all, and new stubs mimic whatever the directory already uses. `bert task adopt` pins the detected format into `.bert/config.yml`.

## Commands

### `/bert:task` - Task Management

```bash
/bert:task create "description"           # New task
/bert:task create -p 3 "subtask"          # Subtask under task 3
/bert:task execute 12                     # Execute single task
/bert:task execute 1.1 to 1.14            # Execute range of tasks
/bert:task execute 1.1 1.3 1.5            # Execute multiple specific tasks
/bert:task status 12 completed            # Update status
/bert:task list [pending|completed]       # List tasks
/bert:task archive 12                     # Archive
```

### `/bert:spec` - Spec Development

```bash
/bert:spec new "description"              # Create requirements.md
/bert:spec iterate 12                     # Smart iteration
/bert:spec tasks 12                       # Create task files
/bert:spec archive 12                     # Archive spec + tasks
/bert:spec archive 12 --tasks-only        # Archive tasks, keep spec
```

**Smart iteration** auto-detects:
- Add follow-up questions to requirements
- Generate spec.md from requirements
- Update spec from feedback

### `/bert:plan` - Product Context

```bash
/bert:plan                                # Create templates
```

Creates: `mission.md`, `roadmap.md`, `tech-stack.md`

## Task Execution Features

The `/bert:task execute` command provides automated task execution with intelligent features:

### Execution Modes

- **Single task**: `/bert:task execute 1.1` - Execute one task
- **Range**: `/bert:task execute 1.1 to 1.14` - Execute sequential tasks automatically
- **Multiple**: `/bert:task execute 1.1 1.3 1.5` - Execute specific non-sequential tasks

### Automated Workflow

When executing a task, Bert automatically:
1. **Reads task file** - Loads objective, scope, and technical approach
2. **Checks dependencies** - Warns if dependent tasks are not completed
3. **Updates status** - Changes from pending → in-progress → completed
4. **Implements work** - Follows the technical approach and deliverables
5. **Verifies success** - Checks all success criteria from the task file
6. **Generates review** - Creates review file for testing and feedback

### Dependency Checking

Bert checks task dependencies before execution:
```
⚠️  Warning: Task 01.3 depends on:
- Task 01.1: pending (not completed)

Recommendation: Complete dependencies first, or proceed with caution.
```

### Range Execution Summary

When executing multiple tasks, Bert provides a summary report:
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Execution Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Completed: 5 tasks
Failed: 1 task

✅ Task 01.1: Create New Directory Structure
✅ Task 01.2: Migrate Exa-Search Source Files
...
❌ Task 01.6: Update TypeScript Configuration (error: ...)

Next steps: Fix issues with Task 01.6
```

## Workflows

For a detailed user walkthrough see [docs/walkthrough.md](./docs/walkthrough.md)

### Ad-hoc Task

```bash
# 1. Create and execute
/bert:task create "fix login bug"
/bert:task execute 15

# 2. AI auto-generates task-15-review.md
# - Lists files changed
# - Documents implementation
# - Provides testing checklist

# 3. You test and add issues
# Edit docs/bert/tasks/task-15-review.md:
### Issue 1: Button not visible (2025-01-21)
**Reporter**: User
**Status**: Open

Button doesn't show on mobile...

# 4. Notify AI
"added notes to task-15-review.md"

# 5. AI fixes and documents
# - Fixes each issue
# - Adds **Fix** notes below each issue
# - Updates status to Fixed

# 6. Iterate until ready
# Repeat steps 3-5 until you check "Ready for production"
```

### Spec-Driven Feature

```bash
# 1. Requirements
/bert:spec new "user authentication"
# → Edit docs/bert/specs/spec-12/requirements.md

# 2. Iterate
/bert:spec iterate 12
# → Adds follow-ups or generates spec.md
# → Edit spec.md, add feedback
/bert:spec iterate 12
# → Repeat until satisfied

# 3. Tasks
/bert:spec tasks 12
# → Creates task-12.1.md, task-12.2.md, etc.

# 4. Execute with review
/bert:task execute 12.1
# → AI creates task-12-review.md (covers all 12.x tasks)
# → Same review workflow as ad-hoc tasks

# 5. Archive when done
/bert:spec archive 12                # Archive spec + tasks
/bert:spec archive 12 --tasks-only   # Keep spec as documentation
```

## File Structure

### Specs

```
docs/bert/specs/spec-12/
├── requirements.md       # Q&A
├── spec.md              # Technical spec
└── visuals/             # Optional mockups
```

### Tasks

```
docs/bert/tasks/
├── task-12.1-database.md      # From spec 12
├── task-12.2-api.md           # From spec 12
├── task-12-review.md          # Review file (auto-generated)
├── task-15-bugfix.md          # Ad-hoc
├── task-15-review.md          # Review file (auto-generated)
└── task-03.1-subtask.md       # Subtask
```

**Review files**:
- Auto-generated after task execution
- `task-01-review.md` covers all `01.x` subtasks
- `task-01.1-review.md` covers all `01.1.x` subtasks
- Contains: implementation summary, files changed, testing checklist, issues section

## Features

- **Hierarchical tasks** - Unlimited nesting (3.1.2.4)
- **Auto-review generation** - AI creates review files after task completion
- **Async issue tracking** - Add issues to review file anytime, AI fixes and documents
- **No lock-in** - Mix spec + ad-hoc tasks
- **File-based Q&A** - Answer requirements async
- **Smart iteration** - One command for all refinement
- **Config-driven** - Customize all paths

## Credits

Spec workflows adapted from [Agent-OS](https://github.com/builder-methods/agent-os) by Brian Casel @ Builder Methods (ISC License).

Bert adaptations:
- File-based Q&A
- Simplified workflow 
- Smart iteration
- Three-tier commands
- Task integration
