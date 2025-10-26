// Main entry point for bert CLI
use clap::{Parser, Subcommand};
use std::process;

mod commands;
mod config;
mod errors;
mod models;
mod numbering;
mod project;
mod utils;

use commands::{spec_archive, spec_stub, task_archive, task_stub};
use config::load_config;
use errors::BertError;

/// Bert CLI - Task Management System
#[derive(Parser)]
#[command(name = "bert")]
#[command(version)]
#[command(about = "Rust CLI for Bert task management system", long_about = None)]
struct Cli {
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

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Task { operation } => match operation {
            TaskOperations::Stub { description, parent } => {
                handle_task_stub(&description, parent.as_deref())
            }
            TaskOperations::Archive {
                task_number,
                recursive,
            } => handle_task_archive(&task_number, recursive),
        },
        Commands::Spec { operation } => match operation {
            SpecOperations::Stub { description } => handle_spec_stub(&description),
            SpecOperations::Archive { spec_number } => handle_spec_archive(&spec_number),
        },
    };

    if let Err(err) = result {
        eprintln!("Error: {}", err);
        process::exit(err.exit_code());
    }
}

fn handle_task_stub(description: &str, parent: Option<&str>) -> Result<(), BertError> {
    let config = load_config()?;

    let (task_number, filepath) = task_stub::create_task_stub(&config, description, parent)?;

    println!("✓ Created task {}: {}", task_number, filepath);

    Ok(())
}

fn handle_task_archive(task_number: &str, recursive: bool) -> Result<(), BertError> {
    let config = load_config()?;

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

fn handle_spec_stub(description: &str) -> Result<(), BertError> {
    let config = load_config()?;

    let (spec_number, dirpath) = spec_stub::create_spec_stub(&config, description)?;

    println!("✓ Created spec {}: {}", spec_number, dirpath);

    Ok(())
}

fn handle_spec_archive(spec_number: &str) -> Result<(), BertError> {
    let config = load_config()?;

    let archived_count = spec_archive::archive_spec(&config, spec_number)?;

    println!(
        "✓ Archived {} items (spec + related tasks)",
        archived_count
    );

    Ok(())
}
