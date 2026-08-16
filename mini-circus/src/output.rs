use crate::models::{Board, Task};
use serde::Serialize;

pub fn print_json<T: Serialize + ?Sized>(value: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("serialize")
    );
}

pub fn print_board(board: &Board, json: bool) {
    if json {
        print_json(board);
        return;
    }
    println!("#{} {}", board.id, board.name);
    if let Some(desc) = &board.description {
        println!("  {desc}");
    }
}

pub fn print_boards(boards: &[Board], json: bool) {
    if json {
        print_json(boards);
        return;
    }
    if boards.is_empty() {
        println!("No boards yet. Create one with `mini-circus board create <name>`.");
        return;
    }
    for board in boards {
        print_board(board, false);
    }
}

pub fn print_task(task: &Task, json: bool) {
    if json {
        print_json(task);
        return;
    }
    let assignee = task.assignee.as_deref().unwrap_or("-");
    let status = format!("[{}]", task.status);
    println!(
        "#{:<4} {:<13} {:<7} {:<15} {}",
        task.id, status, task.priority, assignee, task.title
    );
    if let Some(desc) = &task.description {
        println!("      {desc}");
    }
}

pub fn print_tasks(tasks: &[Task], json: bool) {
    if json {
        print_json(tasks);
        return;
    }
    if tasks.is_empty() {
        println!("No tasks match.");
        return;
    }
    for task in tasks {
        print_task(task, false);
    }
}

pub fn print_optional_task(task: Option<&Task>, json: bool, empty_message: &str) {
    match task {
        Some(t) => print_task(t, json),
        None if json => println!("null"),
        None => println!("{empty_message}"),
    }
}
