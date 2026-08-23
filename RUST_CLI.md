# Bert Rust CLI

Standalone Rust command-line tool for the Bert task management system.

## About

The Bert Rust CLI (`bert` binary) is a standalone tool that complements the Claude Code integration. It handles mechanical operations (creating stubs, archiving tasks) while AI handles intelligence-requiring operations.

## Features (MVP)

- **Task Stub Creation**: Quickly create minimal task stubs with universal numbering
- **Task Listing**: `bert task list` with status/track/priority/tag filters; status synonyms are normalized (`open`/`pending` → todo, `paused`/`deferred` → parked) while files keep their own vocabulary; `--json` for machine consumption
- **Task Archiving**: Archive tasks and associated notes recursively
- **Project Root Detection**: Uses git (`git rev-parse --show-toplevel`); falls back to walking up for a config file, then to the current directory
- **Configuration**: Zero-config by default (tasks at `<repo_root>/docs/tasks`); optional `.bert/config.yml`, legacy `skills.yml`, or `--reporoot` / `--taskdir` flags
- **Self-Contained Footprint**: With zero-config, everything bert creates nests inside the tasks directory — `docs/tasks/{archive,notes,specs,prompts,product}` — claiming no other `docs/` names; explicit `config:` sections keep the classic `{bert_root}` layout
- **Format Tolerance**: Reads any reasonable task-file convention (bare `task-013.md`, slugged `task-01-x.md`, dotted subtasks); new stubs mimic the existing directory's filename shape, padding, frontmatter keys, status vocabulary and H1 style
- **`bert task adopt`**: Persist the detected format as a `format:` section in `.bert/config.yml` so writing is stable even in empty directories

## Prerequisites

- Rust 1.88+ (the repo builds with 1.95)
- Nothing else — bert works in any git repository with no setup; outside a git repo it treats the current directory as the project root

## Installation

### From Source

```bash
# Navigate to the bert project
cd /path/to/bert

# Build and install
cargo install --path .

# The binary will be installed to ~/.cargo/bin/bert
```

### Verify Installation

```bash
bert --help
```

## Usage

### Task Stub

Create a new task stub:

```bash
bert task stub "implement user authentication"
```

Create a subtask:

```bash
bert task stub -p 3 "add validation logic"
```

### Task Archive

Archive a task and all its children:

```bash
bert task archive 08
```

Archive a specific subtask:

```bash
bert task archive 08.2
```

## Development

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

### Run Locally (without installing)

```bash
cargo run -- task stub "test task"
```

## Project Structure

```
src/
├── main.rs           # CLI entry point
├── commands/         # Command implementations
│   ├── task_stub.rs
│   └── task_archive.rs
├── models/           # Data models
│   └── config.rs
├── config.rs         # Configuration parser
├── errors.rs         # Error types
├── numbering.rs      # Universal numbering system
└── project.rs        # Project root detection
```

## Relationship with Claude Code

The Rust CLI and Claude Code AI agents are **independent and complementary**:

- **Rust CLI**: Handles mechanical operations (stubs, archiving) - fast, no AI needed
- **AI Agents**: Handle intelligence operations (task execution, spec generation) - can work without CLI

Both share the same file conventions and work on the same file structure. The CLI discovers its configuration independently: git repo root → `.bert/config.yml` or legacy `skills.yml` at the root → defaults (`<repo_root>/docs/tasks`).

## License

MIT
