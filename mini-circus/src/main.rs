mod cli;
mod db;
mod models;
mod output;
mod store;

use clap::Parser;
use cli::{BoardCommand, Cli, Command, CommentCommand, TaskCommand};
use models::TaskDetail;
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
            output::show_board(&board, json);
        }
        BoardCommand::List => {
            let boards = store::list_boards(pool).await?;
            output::show_boards(&boards, json);
        }
        BoardCommand::Show { board } => {
            let board = store::resolve_board(pool, &board).await?;
            output::show_board(&board, json);
        }
        BoardCommand::Delete { board } => {
            let board = store::resolve_board(pool, &board).await?;
            store::delete_board(pool, board.id).await?;
            output::confirm_deleted("board", board.id, json);
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
            output::show_task(&task, json);
        }
        TaskCommand::List {
            board,
            status,
            assignee,
        } => {
            let board = store::resolve_board(pool, &board).await?;
            let tasks = store::list_tasks(pool, board.id, &TaskFilter { status, assignee }).await?;
            output::show_tasks(&tasks, json);
        }
        TaskCommand::Show { id } => {
            let task = store::get_task(pool, id).await?;
            let comments = store::list_comments(pool, id).await?;
            output::show_task_detail(&TaskDetail { task, comments }, json);
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
            output::show_task(&task, json);
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
            output::show_task(&task, json);
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
            output::show_task(&task, json);
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
            output::show_task(&task, json);
        }
        TaskCommand::Claim { board, assignee } => {
            let board = store::resolve_board(pool, &board).await?;
            let task = store::claim_next_task(pool, board.id, &assignee).await?;
            output::show_optional_task(
                task.as_ref(),
                json,
                "No unassigned pending tasks on this board.",
            );
        }
        TaskCommand::Delete { id } => {
            store::delete_task(pool, id).await?;
            output::confirm_deleted("task", id, json);
        }
        TaskCommand::Comment { command } => run_comment_command(pool, command, json).await?,
    }
    Ok(())
}

async fn run_comment_command(
    pool: &sqlx::SqlitePool,
    command: CommentCommand,
    json: bool,
) -> anyhow::Result<()> {
    match command {
        CommentCommand::Add { task, body, author } => {
            let comment = store::create_comment(pool, task, &author, &body).await?;
            output::show_comment(&comment, json);
        }
        CommentCommand::List { task } => {
            let comments = store::list_comments(pool, task).await?;
            output::show_comments(&comments, json);
        }
        CommentCommand::Delete { id } => {
            store::delete_comment(pool, id).await?;
            output::confirm_deleted("comment", id, json);
        }
    }
    Ok(())
}
