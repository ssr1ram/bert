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

use commands::{task_archive, task_stub};
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

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Task { operation } => match operation {
            TaskOperations::Stub { description, parent } => {
                handle_task_stub(&description, parent.as_deref())
            }
            TaskOperations::Archive { task_number, recursive } => {
                handle_task_archive(&task_number, recursive)
            }
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
        println!("✓ Archived {} files (task + children + notes)", archived_count);
    } else {
        println!("✓ Archived {} file(s)", archived_count);
    }

    Ok(())
}
