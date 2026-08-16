mod cli;
mod db;
mod models;
mod output;
mod store;

use clap::Parser;
use cli::{BoardCommand, Cli, Command, TaskCommand};
use store::{NewTask, TaskFilter, TaskPatch};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let db_path = cli.db.unwrap_or_else(db::default_db_path);
    let pool = db::connect(&db_path).await?;
    let json = cli.json;

    match cli.command {
        Command::Board { command } => run_board_command(&pool, command, json).await,
        Command::Task { command } => run_task_command(&pool, command, json).await,
    }
}

async fn run_board_command(
    pool: &sqlx::SqlitePool,
    command: BoardCommand,
    json: bool,
) -> anyhow::Result<()> {
    match command {
        BoardCommand::Create { name, description } => {
            let board = store::create_board(pool, &name, description.as_deref()).await?;
            output::print_board(&board, json);
        }
        BoardCommand::List => {
            let boards = store::list_boards(pool).await?;
            output::print_boards(&boards, json);
        }
        BoardCommand::Show { board } => {
            let board = store::resolve_board(pool, &board).await?;
            output::print_board(&board, json);
        }
        BoardCommand::Delete { board } => {
            let board = store::resolve_board(pool, &board).await?;
            store::delete_board(pool, board.id).await?;
            if json {
                output::print_json(&serde_json::json!({ "deleted": true, "id": board.id }));
            } else {
                println!("Deleted board #{} ({})", board.id, board.name);
            }
        }
    }
    Ok(())
}

async fn run_task_command(
    pool: &sqlx::SqlitePool,
    command: TaskCommand,
    json: bool,
) -> anyhow::Result<()> {
    match command {
        TaskCommand::Create {
            board,
            title,
            description,
            priority,
            assignee,
        } => {
            let board = store::resolve_board(pool, &board).await?;
            let task = store::create_task(
                pool,
                NewTask {
                    board_id: board.id,
                    title: &title,
                    description: description.as_deref(),
                    priority,
                    assignee: assignee.as_deref(),
                },
            )
            .await?;
            output::print_task(&task, json);
        }
        TaskCommand::List {
            board,
            status,
            assignee,
        } => {
            let board = store::resolve_board(pool, &board).await?;
            let tasks = store::list_tasks(pool, board.id, &TaskFilter { status, assignee }).await?;
            output::print_tasks(&tasks, json);
        }
        TaskCommand::Show { id } => {
            let task = store::get_task(pool, id).await?;
            output::print_task(&task, json);
        }
        TaskCommand::Update {
            id,
            title,
            description,
            priority,
        } => {
            let task = store::update_task(
                pool,
                id,
                TaskPatch {
                    title: title.as_deref(),
                    description: description.as_deref().map(Some),
                    priority,
                    ..Default::default()
                },
            )
            .await?;
            output::print_task(&task, json);
        }
        TaskCommand::Status { id, status } => {
            let task = store::update_task(
                pool,
                id,
                TaskPatch {
                    status: Some(status),
                    ..Default::default()
                },
            )
            .await?;
            output::print_task(&task, json);
        }
        TaskCommand::Assign { id, assignee } => {
            let task = store::update_task(
                pool,
                id,
                TaskPatch {
                    assignee: Some(Some(&assignee)),
                    ..Default::default()
                },
            )
            .await?;
            output::print_task(&task, json);
        }
        TaskCommand::Unassign { id } => {
            let task = store::update_task(
                pool,
                id,
                TaskPatch {
                    assignee: Some(None),
                    ..Default::default()
                },
            )
            .await?;
            output::print_task(&task, json);
        }
        TaskCommand::Claim { board, assignee } => {
            let board = store::resolve_board(pool, &board).await?;
            let task = store::claim_next_task(pool, board.id, &assignee).await?;
            output::print_optional_task(
                task.as_ref(),
                json,
                "No unassigned pending tasks on this board.",
            );
        }
        TaskCommand::Delete { id } => {
            store::delete_task(pool, id).await?;
            if json {
                output::print_json(&serde_json::json!({ "deleted": true, "id": id }));
            } else {
                println!("Deleted task #{id}");
            }
        }
    }
    Ok(())
}
