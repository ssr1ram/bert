// Main entry point for bert CLI
use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod config;
mod errors;
mod models;
mod numbering;
mod project;
mod tui;
mod utils;

use commands::{prompt_stub, spec_archive, spec_stub, task_archive, task_stub, setdir};
use config::load_config;
use errors::BertError;

/// Bert CLI - Task Management System
#[derive(Parser)]
#[command(name = "bert")]
#[command(version)]
#[command(about = "Rust CLI for Bert task management system", long_about = None)]
struct Cli {
    /// Override BERT project directory
    #[arg(long, global = true)]
    bert_dir: Option<std::path::PathBuf>,

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
    /// Examples:
    ///   bert task stub "implement user authentication"
    ///   bert task stub -p 3 "add validation logic"
    Stub {
        /// Task description
        description: String,

        /// Parent task number (e.g., "03" or "03.1")
        #[arg(short, long)]
        parent: Option<String>,
    },

    /// Archive a task and its notes
    ///
    /// Examples:
    ///   bert task archive 08
    ///   bert task archive 08.1
    Archive {
        /// Task number to archive
        task_number: String,

        /// Recursively archive all child tasks
        #[arg(short, long)]
        recursive: bool,
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
                handle_task_stub(cli.bert_dir, &description, parent.as_deref())
            }
            TaskOperations::Archive {
                task_number,
                recursive,
            } => handle_task_archive(cli.bert_dir, &task_number, recursive),
        },
        Commands::Spec { operation } => match operation {
            SpecOperations::Stub { description } => handle_spec_stub(cli.bert_dir, &description),
            SpecOperations::Archive { spec_number } => handle_spec_archive(cli.bert_dir, &spec_number),
        },
        Commands::Prompt { operation } => match operation {
            PromptOperations::Stub { description, verbose } => handle_prompt_stub(cli.bert_dir, &description, verbose),
        },
        Commands::Tui { command } => handle_tui(cli.bert_dir, command.as_deref()),
        Commands::Debug => handle_debug(cli.bert_dir),
        Commands::Setdir => handle_setdir(cli.bert_dir),
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        process::exit(err.exit_code());
    }
}

fn handle_task_stub(bert_dir: Option<std::path::PathBuf>, description: &str, parent: Option<&str>) -> Result<(), BertError> {
    let config = load_config(bert_dir)?;

    let (task_number, filepath) = task_stub::create_task_stub(&config, description, parent)?;

    println!("✓ Created task {}: {}", task_number, filepath);

    Ok(())
}

fn handle_task_archive(bert_dir: Option<std::path::PathBuf>, task_number: &str, recursive: bool) -> Result<(), BertError> {
    let config = load_config(bert_dir)?;

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

fn handle_spec_stub(bert_dir: Option<std::path::PathBuf>, description: &str) -> Result<(), BertError> {
    let config = load_config(bert_dir)?;

    let (spec_number, dirpath) = spec_stub::create_spec_stub(&config, description)?;

    println!("✓ Created spec {}: {}", spec_number, dirpath);

    Ok(())
}

fn handle_spec_archive(bert_dir: Option<std::path::PathBuf>, spec_number: &str) -> Result<(), BertError> {
    let config = load_config(bert_dir)?;

    let archived_count = spec_archive::archive_spec(&config, spec_number)?;

    println!(
        "✓ Archived {} items (spec + related tasks)",
        archived_count
    );

    Ok(())
}

fn handle_prompt_stub(bert_dir: Option<std::path::PathBuf>, description: &str, verbose: bool) -> Result<(), BertError> {
    let config = load_config(bert_dir)?;

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

fn handle_tui(bert_dir: Option<std::path::PathBuf>, command: Option<&str>) -> Result<(), BertError> {
    let config = load_config(bert_dir)?;

    // Single line - TUI is completely isolated
    tui::launch(&config, command)?;

    Ok(())
}

fn handle_debug(bert_dir: Option<std::path::PathBuf>) -> Result<(), BertError> {
    let config = load_config(bert_dir)?;

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

    Ok(())
}

fn handle_setdir(bert_dir: Option<std::path::PathBuf>) -> Result<(), BertError> {
    let target_dir = match bert_dir {
        Some(path) => path,
        None => std::env::current_dir().map_err(|e| BertError::ConfigError(e.to_string()))?,
    };

    setdir::create_default_config(&target_dir)?;

    println!("✓ Initialized BERT project in {}", target_dir.display());
    println!("  Created skills.yml and base directory structure.");

    Ok(())
}
