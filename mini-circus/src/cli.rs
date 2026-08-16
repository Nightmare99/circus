use clap::{Parser, Subcommand};
use common::{ParseEnumError, Priority, TaskStatus};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "mini-circus",
    version,
    about = "A minimal, single-file task board.",
    long_about = "A minimal task board with no accounts and no permissions: \
        anyone with access to the database file can create boards and tasks \
        and assign them to any name. Meant for coordinating work across \
        multiple independent processes on the same machine or repo."
)]
pub struct Cli {
    /// Path to the board database file. Defaults to $MINI_CIRCUS_DB, or
    /// ./.mini-circus/board.db if unset - created automatically if missing.
    #[arg(long, global = true)]
    pub db: Option<PathBuf>,

    /// Print machine-readable JSON instead of a formatted table.
    #[arg(long, global = true)]
    pub json: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Create, list, and inspect boards
    Board {
        #[command(subcommand)]
        command: BoardCommand,
    },
    /// Create, list, and update tasks on a board
    Task {
        #[command(subcommand)]
        command: TaskCommand,
    },
}

#[derive(Subcommand)]
pub enum BoardCommand {
    /// Create a new board
    Create {
        name: String,
        #[arg(long)]
        description: Option<String>,
    },
    /// List all boards
    List,
    /// Show a single board
    Show {
        /// Board name or id
        board: String,
    },
    /// Delete a board and all of its tasks
    Delete {
        /// Board name or id
        board: String,
    },
}

#[derive(Subcommand)]
pub enum TaskCommand {
    /// Create a task on a board
    Create {
        /// Board name or id
        #[arg(long)]
        board: String,
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, default_value = "medium", value_parser = parse_priority)]
        priority: Priority,
        /// Free-text name to assign the task to right away
        #[arg(long)]
        assignee: Option<String>,
    },
    /// List tasks on a board, optionally filtered
    List {
        /// Board name or id
        #[arg(long)]
        board: String,
        #[arg(long, value_parser = parse_status)]
        status: Option<TaskStatus>,
        #[arg(long)]
        assignee: Option<String>,
    },
    /// Show a single task
    Show { id: i64 },
    /// Update a task's title, description, or priority
    Update {
        id: i64,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long, value_parser = parse_priority)]
        priority: Option<Priority>,
    },
    /// Set a task's status directly
    Status {
        id: i64,
        #[arg(value_parser = parse_status)]
        status: TaskStatus,
    },
    /// Assign a task to a name
    Assign { id: i64, assignee: String },
    /// Clear a task's assignee
    Unassign { id: i64 },
    /// Atomically claim the next unassigned, pending task on a board and
    /// mark it in progress. Safe to call concurrently from multiple
    /// processes polling the same board - each call gets a different task,
    /// or nothing if none are available.
    Claim {
        /// Board name or id
        #[arg(long)]
        board: String,
        /// Free-text name to assign the claimed task to
        assignee: String,
    },
    /// Delete a task
    Delete { id: i64 },
}

fn parse_status(s: &str) -> Result<TaskStatus, ParseEnumError> {
    s.parse()
}

fn parse_priority(s: &str) -> Result<Priority, ParseEnumError> {
    s.parse()
}
