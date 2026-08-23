// Main entry point for bert CLI
use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod config;
mod errors;
mod format;
mod models;
mod numbering;
mod project;
mod tui;
mod utils;

use commands::{adopt, list, prompt_stub, spec_archive, spec_stub, task_archive, task_stub, setdir};
use config::load_config;
use errors::BertError;

/// Bert CLI - Task Management System
#[derive(Parser)]
#[command(name = "bert")]
#[command(version)]
#[command(about = "Rust CLI for Bert task management system", long_about = None)]
struct Cli {
    /// Override the repository root (default: discovered via git from cwd)
    #[arg(long = "reporoot", global = true, value_name = "DIR", visible_alias = "repodir", alias = "repo-root", alias = "bert-dir")]
    repo_root: Option<std::path::PathBuf>,

    /// Override the tasks directory (default: <repo_root>/docs/tasks)
    #[arg(long = "taskdir", global = true, value_name = "DIR", visible_alias = "task-dir", alias = "tasks-dir")]
    task_dir: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Task management operations
    Task {
        #[command(subcommand)]
        operation: TaskOperations,
    },
    /// Spec management operations
    Spec {
        #[command(subcommand)]
        operation: SpecOperations,
    },
    /// Prompt log management operations
    #[command(alias = "pr")]
    Prompt {
        #[command(subcommand)]
        operation: PromptOperations,
    },
    /// Launch TUI interface
    Tui {
        /// Optional: direct command (prompt, spec, task)
        command: Option<String>,
    },
    /// Debug: Print configuration
    Debug,
    /// Initialize a new BERT project directory
    Setdir,
}

#[derive(Subcommand)]
enum TaskOperations {
    /// Create a new task stub
    ///
    /// Creates a minimal task file in the tasks directory (zero-config
    /// default: `<repo_root>/docs/tasks`). The number continues the
    /// directory's existing sequence, and the filename shape, padding,
    /// frontmatter keys and status word mimic what the directory already
    /// uses — bare `task-034.md` in a bare directory, `task-034-slug.md`
    /// in a slugged one.
    ///
    /// With --parent, the new file becomes the next child of that task
    /// (e.g. parent 3 -> task-03.1, then 03.2, ...). The parent must exist.
    ///
    /// Examples:
    ///   bert task stub "implement user authentication"
    ///   bert task stub -p 3 "add validation logic"
    Stub {
        /// Task description (used for the title and, in slugged dirs, the filename)
        description: String,

        /// Parent task number (e.g., "03" or "03.1")
        #[arg(short, long)]
        parent: Option<String>,
    },

    /// Archive a task and its notes
    ///
    /// Moves the task file out of the tasks directory into the archive,
    /// keeping its filename. If a matching notes file exists, it moves too.
    /// Nothing is deleted.
    ///
    /// "Notes" are companion markdown files named `note-{NUMBER}-*.md` living
    /// in the notes directory (e.g. `docs/notes/note-08-findings.md`); they
    /// are optional and only archived when present.
    ///
    /// Destinations (zero-config defaults):
    ///   docs/tasks/task-08-*.md  ->  docs/archive/tasks/
    ///   docs/notes/note-08-*.md  ->  docs/archive/notes/
    ///
    /// Paths follow your configuration: `.bert/config.yml` (or legacy
    /// `skills.yml`) can relocate them, and `--tasks-dir` overrides the source.
    /// Run `bert debug` anywhere to print the resolved directories.
    ///
    /// Examples:
    ///   bert task archive 08              # single task (any padding: 7 == 007)
    ///   bert task archive 08.1            # a subtask
    ///   bert task archive 08 --recursive  # also archive 08.* children + their notes
    Archive {
        /// Task number to archive (padding-insensitive: "7" matches task-07/task-007 files)
        task_number: String,

        /// Recursively archive all child tasks
        #[arg(short, long)]
        recursive: bool,
    },

    /// Adopt an existing tasks directory's conventions
    ///
    /// Scans the tasks directory, detects its filename/frontmatter format,
    /// and persists it as a `format:` section in `.bert/config.yml`.
    ///
    /// Example:
    ///   bert task adopt
    Adopt {},

    /// List tasks with optional filtering
    ///
    /// Status filters accept canonical words (todo, doing, done, blocked,
    /// parked) or the directory's own vocabulary (open, paused, deferred...).
    ///
    /// Examples:
    ///   bert task list
    ///   bert task list --status parked --track newsletter
    ///   bert task list --priority p1 --json
    List {
        /// Filter by status (canonical or raw vocabulary)
        #[arg(short, long)]
        status: Option<String>,

        /// Filter by track value
        #[arg(long)]
        track: Option<String>,

        /// Filter by priority value
        #[arg(long)]
        priority: Option<String>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
enum SpecOperations {
    /// Create a new spec stub directory
    ///
    /// Examples:
    ///   bert spec stub "implement user authentication"
    Stub {
        /// Spec description
        description: String,
    },

    /// Archive a spec and all its related tasks
    ///
    /// Examples:
    ///   bert spec archive 08
    Archive {
        /// Spec number to archive
        spec_number: String,
    },
}

#[derive(Subcommand)]
enum PromptOperations {
    /// Create a new prompt log stub
    ///
    /// Examples:
    ///   bert prompt stub "system creation"
    ///   bert pr stub "testing new feature"
    ///   bert pr st --verbose "quick test"
    #[command(alias = "st")]
    Stub {
        /// Brief description of the prompt log
        description: String,

        /// Enable verbose logging
        #[arg(short, long)]
        verbose: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Task { operation } => match operation {
            TaskOperations::Stub { description, parent } => {
                handle_task_stub(cli.repo_root, cli.task_dir, &description, parent.as_deref())
            }
            TaskOperations::Archive {
                task_number,
                recursive,
            } => handle_task_archive(cli.repo_root, cli.task_dir, &task_number, recursive),
            TaskOperations::Adopt {} => handle_task_adopt(cli.repo_root),
            TaskOperations::List {
                status,
                track,
                priority,
                tag,
                json,
            } => handle_task_list(
                cli.repo_root,
                cli.task_dir,
                list::ListFilter {
                    status,
                    track,
                    priority,
                    tag,
                },
                json,
            ),
        },
        Commands::Spec { operation } => match operation {
            SpecOperations::Stub { description } => handle_spec_stub(cli.repo_root, cli.task_dir, &description),
            SpecOperations::Archive { spec_number } => handle_spec_archive(cli.repo_root, cli.task_dir, &spec_number),
        },
        Commands::Prompt { operation } => match operation {
            PromptOperations::Stub { description, verbose } => handle_prompt_stub(cli.repo_root, cli.task_dir, &description, verbose),
        },
        Commands::Tui { command } => handle_tui(cli.repo_root, cli.task_dir, command.as_deref()),
        Commands::Debug => handle_debug(cli.repo_root, cli.task_dir),
        Commands::Setdir => handle_setdir(cli.repo_root),
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        process::exit(err.exit_code());
    }
}

fn handle_task_stub(repo_root: Option<std::path::PathBuf>, task_dir: Option<std::path::PathBuf>, description: &str, parent: Option<&str>) -> Result<(), BertError> {
    let config = load_config(repo_root, task_dir)?;

    let (task_number, filepath) = task_stub::create_task_stub(&config, description, parent)?;

    println!("✓ Created task {}: {}", task_number, filepath);

    Ok(())
}

fn handle_task_archive(repo_root: Option<std::path::PathBuf>, task_dir: Option<std::path::PathBuf>, task_number: &str, recursive: bool) -> Result<(), BertError> {
    let config = load_config(repo_root, task_dir)?;

    let archived_count = task_archive::archive_task(&config, task_number, recursive)?;

    if recursive {
        println!(
            "✓ Archived {} files (task + children + notes)",
            archived_count
        );
    } else {
        println!("✓ Archived {} file(s)", archived_count);
    }

    Ok(())
}

fn handle_task_adopt(repo_root: Option<std::path::PathBuf>) -> Result<(), BertError> {
    let config = load_config(repo_root, None)?;

    let summary = adopt::adopt(&config)?;
    println!("✓ {}", summary);

    Ok(())
}

fn handle_task_list(
    repo_root: Option<std::path::PathBuf>,
    task_dir: Option<std::path::PathBuf>,
    filter: list::ListFilter,
    json: bool,
) -> Result<(), BertError> {
    let config = load_config(repo_root, task_dir)?;

    list::list_tasks(&config, &filter, json)?;

    Ok(())
}

fn handle_spec_stub(repo_root: Option<std::path::PathBuf>, task_dir: Option<std::path::PathBuf>, description: &str) -> Result<(), BertError> {
    let config = load_config(repo_root, task_dir)?;

    let (spec_number, dirpath) = spec_stub::create_spec_stub(&config, description)?;

    println!("✓ Created spec {}: {}", spec_number, dirpath);

    Ok(())
}

fn handle_spec_archive(repo_root: Option<std::path::PathBuf>, task_dir: Option<std::path::PathBuf>, spec_number: &str) -> Result<(), BertError> {
    let config = load_config(repo_root, task_dir)?;

    let archived_count = spec_archive::archive_spec(&config, spec_number)?;

    println!(
        "✓ Archived {} items (spec + related tasks)",
        archived_count
    );

    Ok(())
}

fn handle_prompt_stub(repo_root: Option<std::path::PathBuf>, task_dir: Option<std::path::PathBuf>, description: &str, verbose: bool) -> Result<(), BertError> {
    let config = load_config(repo_root, task_dir)?;

    if verbose {
        eprintln!("[verbose] Loaded config:");
        eprintln!("[verbose]   project_root: {}", config.project_root.display());
        eprintln!("[verbose]   bert_root: {}", config.bert_root.display());
    }

    let (prompt_number, filepath, template_used) = prompt_stub::create_prompt_stub(&config, description, verbose)?;

    println!("✓ Created prompt log {}: {}", prompt_number, filepath);

    if let Some(template_path) = template_used {
        println!("  ℹ Using custom template: {}", template_path);
    }

    Ok(())
}

fn handle_tui(repo_root: Option<std::path::PathBuf>, task_dir: Option<std::path::PathBuf>, _command: Option<&str>) -> Result<(), BertError> {
    let config = load_config(repo_root, task_dir)?;

    // Single line - TUI is completely isolated
    tui::launch(&config)?;

    Ok(())
}

fn handle_debug(repo_root: Option<std::path::PathBuf>, task_dir: Option<std::path::PathBuf>) -> Result<(), BertError> {
    let config = load_config(repo_root, task_dir)?;

    println!("BERT Configuration Debug");
    println!("========================\n");

    println!("Project Root: {}", config.project_root.display());
    println!("Bert Root: {}", config.bert_root.display());
    println!();

    println!("Directories:");

    if let Some(ref path) = config.library_directory {
        println!("  Library:  {} (exists: {})", path.display(), path.exists());
    } else {
        println!("  Library:  (not configured)");
    }

    if let Some(ref path) = config.sets_directory {
        println!("  Sets:     {} (exists: {})", path.display(), path.exists());
    } else {
        println!("  Sets:     (not configured)");
    }

    println!("  Specs:    {} (exists: {})", config.specs_directory.display(), config.specs_directory.exists());
    println!("  Tasks:    {} (exists: {})", config.tasks_directory.display(), config.tasks_directory.exists());

    if let Some(ref path) = config.archive_directory {
        println!("  Archive:  {} (exists: {})", path.display(), path.exists());
    } else {
        println!("  Archive:  (not configured)");
    }

    if let Some(ref path) = config.notes_directory {
        println!("  Notes:    {} (exists: {})", path.display(), path.exists());
    } else {
        println!("  Notes:    (not configured)");
    }

    if let Some(ref path) = config.product_directory {
        println!("  Product:  {} (exists: {})", path.display(), path.exists());
    } else {
        println!("  Product:  (not configured)");
    }

    if let Some(ref path) = config.prompt_logs {
        println!("  Logs:     {} (exists: {})", path.display(), path.exists());
    } else {
        println!("  Logs:     (not configured)");
    }

    // Show the effective write-format (explicit config > mimicry > defaults)
    let profile = format::apply_overrides(
        format::detect_profile(&config.tasks_directory),
        config.format.as_ref(),
    );
    println!("\nTask Format (effective):");
    println!("  slug: {}", profile.use_slug);
    println!("  padding: {}", profile.number_width);
    println!("  h1_lowercase: {}", profile.h1_lowercase);
    println!("  todo_status_word: \"{}\"", profile.todo_status_word);
    println!(
        "  frontmatter: [{}]",
        profile
            .frontmatter_keys
            .iter()
            .map(|(k, _)| k.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    Ok(())
}

fn handle_setdir(repo_root: Option<std::path::PathBuf>) -> Result<(), BertError> {
    let target_dir = match repo_root {
        Some(path) => path,
        None => std::env::current_dir().map_err(|e| BertError::ConfigError(e.to_string()))?,
    };

    setdir::create_default_config(&target_dir)?;

    println!("✓ Initialized BERT project in {}", target_dir.display());
    println!("  Created skills.yml and base directory structure.");

    Ok(())
}
