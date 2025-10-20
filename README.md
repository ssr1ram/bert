# Bert - Build, Execute, and Refine Tasks

Task management for Claude Code with optional spec-driven development.

## Quick Start

```bash
# Install: Copy files to your Claude Code project (see Installation)

# Basic task workflow
/bert:task create "add feature"
/bert:task execute 15

# Spec-driven workflow (complex features)
/bert:spec new "user authentication"
/bert:spec iterate 12
/bert:spec tasks 12
/bert:task execute 12.1
```

## What is Bert?

- **Task management** - Create, track, archive hierarchical tasks
- **Spec development** - Optional requirements + spec writing for complex features
- **Product context** - Optional mission/roadmap/tech-stack for better AI assistance

## Installation

Copy these files to your Claude Code project:

### File Locations

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

### Directories (auto-created)

```
your-project/
└── docs/bert/
    ├── specs/              # Specs (auto-created)
    ├── tasks/              # Tasks (auto-created)
    ├── notes/              # Notes (auto-created)
    ├── product/            # Product context (optional)
    └── archive/            # Archives (auto-created)
```

### Configuration

Edit `.claude/skills/bert/skill.yml` if needed:

```yaml
config:
  tasks_directory: docs/bert/tasks
  specs_directory: docs/bert/specs
  product_directory: docs/bert/product
  # ... archive paths
```

## Commands

### `/bert:task` - Task Management

```bash
/bert:task create "description"           # New task
/bert:task create -p 3 "subtask"          # Subtask under task 3
/bert:task execute 12                     # Work on task
/bert:task status 12 completed            # Update status
/bert:task list [pending|completed]       # List tasks
/bert:task archive 12                     # Archive
```

### `/bert:spec` - Spec Development

```bash
/bert:spec new "description"              # Create requirements.md
/bert:spec iterate 12                     # Smart iteration
/bert:spec tasks 12                       # Create task files
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

## Workflows

### Ad-hoc Task

```bash
/bert:task create "fix login bug"
/bert:task execute 15
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

# 4. Execute
/bert:task execute 12.1
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
├── task-15-bugfix.md          # Ad-hoc
└── task-03.1-subtask.md       # Subtask
```

Spec-based tasks use spec number prefix: `spec-12` → `task-12.1`, `12.2`, etc.

## Features

- **Hierarchical tasks** - Unlimited nesting (3.1.2.4)
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
