use crate::models::{Board, Comment, Task, TaskDetail};
use serde::Serialize;
use std::sync::OnceLock;
use termimad::crossterm::style::Color;
use termimad::MadSkin;

/// Matches circus's frontend accent color, so the two tools read as one
/// family even though this one lives entirely in a terminal.
const ACCENT: Color = Color::Rgb {
    r: 245,
    g: 166,
    b: 35,
};

fn skin() -> &'static MadSkin {
    static SKIN: OnceLock<MadSkin> = OnceLock::new();
    SKIN.get_or_init(|| {
        let mut skin = MadSkin::default();
        skin.bold.set_fg(ACCENT);
        skin.headers[0].compound_style.set_fg(ACCENT);
        skin.headers[1].compound_style.set_fg(ACCENT);
        skin.italic.set_fg(Color::DarkGrey);
        skin
    })
}

fn render(markdown: &str) {
    skin().print_text(markdown);
}

pub fn print_json<T: Serialize + ?Sized>(value: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(value).expect("serialize")
    );
}

/// User-supplied text can contain markdown special characters (a title with
/// a literal `*`, a body with a `|`, ...); escape it so it renders as literal
/// text rather than being interpreted as formatting.
///
/// Only escape characters minimad's parser actually treats as escapable:
/// backslash, asterisk, tilde, pipe, and backtick. Escaping anything else
/// (underscore, hash, brackets) left a stray visible backslash in testing,
/// since an unrecognized backslash sequence prints both characters instead
/// of consuming the backslash. Hash and `>` are also only special at the
/// start of a line anyway, which user text never lands on here.
fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '\\' | '*' | '~' | '|' | '`') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn fmt_time(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M").to_string()
}

// ---- boards ----------------------------------------------------------

fn board_markdown(board: &Board) -> String {
    let mut s = format!("## {} `#{}`\n", esc(&board.name), board.id);
    if let Some(desc) = &board.description {
        s.push_str(&format!("*{}*\n", esc(desc)));
    }
    s
}

fn boards_markdown(boards: &[Board]) -> String {
    if boards.is_empty() {
        return "*No boards yet. Create one with `mini-circus board create <name>`.*\n".to_string();
    }
    let mut s = String::from("|id|name|description|\n|-:|-|-|\n");
    for b in boards {
        s.push_str(&format!(
            "|{}|**{}**|{}|\n",
            b.id,
            esc(&b.name),
            b.description.as_deref().map(esc).unwrap_or_default()
        ));
    }
    s
}

pub fn show_board(board: &Board, json: bool) {
    if json {
        print_json(board);
        return;
    }
    render(&board_markdown(board));
}

pub fn show_boards(boards: &[Board], json: bool) {
    if json {
        print_json(boards);
        return;
    }
    render(&boards_markdown(boards));
}

// ---- tasks -------------------------------------------------------------

fn task_markdown(task: &Task) -> String {
    let assignee = task
        .assignee
        .as_deref()
        .map(esc)
        .unwrap_or_else(|| "*unassigned*".to_string());
    let mut s = format!(
        "### `#{}` {}\n**Status:** {}  **Priority:** {}  **Assignee:** {}\n",
        task.id,
        esc(&task.title),
        task.status,
        task.priority,
        assignee
    );
    match &task.description {
        Some(desc) => s.push_str(&format!("\n{}\n", esc(desc))),
        None => s.push_str("\n*No description.*\n"),
    }
    s
}

fn tasks_markdown(tasks: &[Task]) -> String {
    if tasks.is_empty() {
        return "*No tasks match.*\n".to_string();
    }
    let mut s = String::from("|id|status|priority|assignee|title|\n|-:|-|-|-|-|\n");
    for t in tasks {
        s.push_str(&format!(
            "|{}|{}|{}|{}|{}|\n",
            t.id,
            t.status,
            t.priority,
            t.assignee
                .as_deref()
                .map(esc)
                .unwrap_or_else(|| "-".to_string()),
            esc(&t.title),
        ));
    }
    s
}

fn comments_section_markdown(comments: &[Comment]) -> String {
    if comments.is_empty() {
        return "\n#### Comments\n\n*No comments.*\n".to_string();
    }
    let mut s = format!("\n#### Comments ({})\n\n", comments.len());
    for c in comments {
        s.push_str(&format!(
            "**{}** · `#{}` · {}\n> {}\n\n",
            esc(&c.author),
            c.id,
            fmt_time(c.created_at),
            esc(&c.body).replace('\n', "\n> ")
        ));
    }
    s
}

fn task_detail_markdown(detail: &TaskDetail) -> String {
    let mut s = task_markdown(&detail.task);
    s.push_str(&comments_section_markdown(&detail.comments));
    s
}

pub fn show_task(task: &Task, json: bool) {
    if json {
        print_json(task);
        return;
    }
    render(&task_markdown(task));
}

pub fn show_tasks(tasks: &[Task], json: bool) {
    if json {
        print_json(tasks);
        return;
    }
    render(&tasks_markdown(tasks));
}

pub fn show_task_detail(detail: &TaskDetail, json: bool) {
    if json {
        print_json(detail);
        return;
    }
    render(&task_detail_markdown(detail));
}

pub fn show_optional_task(task: Option<&Task>, json: bool, empty_message: &str) {
    match task {
        Some(t) => show_task(t, json),
        None if json => println!("null"),
        None => render(&format!("*{}*\n", esc(empty_message))),
    }
}

// ---- comments ------------------------------------------------------------

fn comments_markdown(comments: &[Comment]) -> String {
    if comments.is_empty() {
        return "*No comments.*\n".to_string();
    }
    let mut s = String::new();
    for c in comments {
        s.push_str(&format!(
            "**{}** · `#{}` · {}\n> {}\n\n",
            esc(&c.author),
            c.id,
            fmt_time(c.created_at),
            esc(&c.body).replace('\n', "\n> ")
        ));
    }
    s
}

pub fn show_comment(comment: &Comment, json: bool) {
    if json {
        print_json(comment);
        return;
    }
    render(&comments_markdown(std::slice::from_ref(comment)));
}

pub fn show_comments(comments: &[Comment], json: bool) {
    if json {
        print_json(comments);
        return;
    }
    render(&comments_markdown(comments));
}

// ---- confirmations ---------------------------------------------------

pub fn confirm_deleted(kind: &str, id: impl std::fmt::Display, json: bool) {
    if json {
        print_json(&serde_json::json!({ "deleted": true, "id": id.to_string() }));
        return;
    }
    render(&format!("**Deleted** {kind} `#{id}`\n"));
}
