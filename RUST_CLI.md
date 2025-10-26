# Bert Rust CLI

Standalone Rust command-line tool for the Bert task management system.

## About

The Bert Rust CLI (`bert` binary) is a standalone tool that complements the Claude Code integration. It handles mechanical operations (creating stubs, archiving tasks) while AI handles intelligence-requiring operations.

## Features (MVP)

- **Task Stub Creation**: Quickly create minimal task stubs with universal numbering
- **Task Archiving**: Archive tasks and associated notes recursively
- **Project Root Detection**: Automatically finds bert projects by walking up directories
- **Configuration**: Reads from `.claude/skills/bert/skill.yml`

## Prerequisites

- Rust 1.70+ (stable)
- A bert project (contains `.claude/skills/bert/skill.yml`)

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

Both read from the same `skill.yml` configuration and work on the same file structure.

## License

MIT
