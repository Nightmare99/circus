# mini-circus

A minimal task board with no accounts and no permissions. Anyone with access
to the database file can create boards and tasks, assign them to any
(free-text) name, and leave comments. It's a single self-contained binary
backed by one SQLite file — no server, no setup step, no config beyond an
optional path override. Human-facing output renders as styled markdown in
the terminal; pass `--json` anywhere you need to parse it instead.

It shares its task-status vocabulary (`TaskStatus`, `Priority`) with
[circus](../README.md), the full multi-tenant issue tracker this repo also
contains, via the `common` crate — but nothing else. No orgs, no users, no
RBAC, no web server. If you want those, use circus. mini-circus is for
coordinating work between multiple independent processes that need a shared,
concurrency-safe list of tasks and don't need anyone to log in.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/Nightmare99/circus/main/mini-circus/install.sh | sh
```

This downloads a prebuilt binary from the repo's
[Releases](https://github.com/Nightmare99/circus/releases) page, verifies its
checksum, and installs it to `~/.local/bin` (add that to your `PATH` if the
installer tells you it isn't already). Prebuilt binaries are published for:

- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`
- `x86_64-apple-darwin`
- `aarch64-apple-darwin`

**Installer environment variables:**

| Variable | Default | Purpose |
|---|---|---|
| `MINI_CIRCUS_VERSION` | `latest` | Install a specific version, e.g. `MINI_CIRCUS_VERSION=0.1.0 curl ... \| sh` |
| `MINI_CIRCUS_INSTALL_DIR` | `$HOME/.local/bin` | Where to put the binary |

**Uninstall:** it's one file — `rm ~/.local/bin/mini-circus` (or wherever you
installed it).

**Windows / unsupported platform:** the installer only handles macOS and
Linux. Build from source instead (below) — everything mini-circus depends on
is cross-platform, only the install script isn't.

### Build from source

Needs a [Rust toolchain](https://rustup.rs).

```bash
# install straight from the repo
cargo install --git https://github.com/Nightmare99/circus mini-circus

# or, from a local clone
cargo build --release -p mini-circus
# binary at target/release/mini-circus
```

## Quickstart

```bash
mini-circus board create backlog
mini-circus task create --board backlog "Write release notes" --priority high
mini-circus task create --board backlog "Update dependencies"
mini-circus task list --board backlog

# hand a task to someone (or something) by name
mini-circus task assign 1 alice

# atomically pull the next unclaimed task off the board
mini-circus task claim --board backlog worker-1

mini-circus task status 1 completed
mini-circus --json task list --board backlog
```

## Concepts

- **Board** — a named container for tasks. Referenced on the command line by
  either its numeric id or its (unique) name, whichever's more convenient.
- **Task** — belongs to exactly one board. Has a title, an optional
  description, a `status`, a `priority`, and an optional free-text
  `assignee`. Referenced by its numeric id (unique across all boards, not
  just within one).
- **Status** — one of `pending`, `in_progress`, `blocked`, `completed`.
  Fixed, not configurable — same four states circus's boards use.
- **Priority** — one of `low`, `medium`, `high`, `urgent`. Defaults to
  `medium`.
- **Assignee** — an arbitrary string. There's no user table to validate it
  against, so typos create a new "assignee" silently. That's the tradeoff
  for having no accounts at all.
- **Comment** — belongs to exactly one task, has a free-text `author` (same
  tradeoff as assignee) and a body. Ordered by creation time. `task show`
  includes a task's comments; nothing else does.

## Command reference

Global flags (valid on every command):

| Flag | Purpose |
|---|---|
| `--db <path>` | Path to the database file. Default: `$MINI_CIRCUS_DB`, or `./.mini-circus/board.db` if that's unset. Created automatically if missing. |
| `--json` | Print machine-readable JSON instead of a formatted table. |

### Boards

```
mini-circus board create <name> [--description <text>]
mini-circus board list
mini-circus board show <board>
mini-circus board delete <board>
```

`<board>` everywhere means "a board's name or numeric id." Deleting a board
deletes all of its tasks too.

### Tasks

```
mini-circus task create --board <board> <title>
    [--description <text>] [--priority low|medium|high|urgent] [--assignee <name>]

mini-circus task list --board <board>
    [--status pending|in_progress|blocked|completed] [--assignee <name>]

mini-circus task show <id>

mini-circus task update <id>
    [--title <text>] [--description <text>] [--priority low|medium|high|urgent]

mini-circus task status <id> <pending|in_progress|blocked|completed>

mini-circus task assign <id> <name>
mini-circus task unassign <id>

mini-circus task claim --board <board> <name>

mini-circus task delete <id>
```

### Comments

```
mini-circus task comment add <task-id> <body> --author <name>
mini-circus task comment list <task-id>
mini-circus task comment delete <id>
```

`task show <id>` already includes a task's comments, so `comment list` is
mainly useful if you want just the comments (e.g. piping `--json` output
into something else) without the rest of the task.

**`task claim`** is the one worth understanding: it atomically finds the
oldest unassigned task on a board with status `pending`, assigns it to
`<name>`, and sets its status to `in_progress` — all in a single SQL
statement (`UPDATE ... WHERE id = (SELECT ...) RETURNING ...`). If nothing is
available it returns nothing (exit 0, prints a message in text mode / `null`
in JSON mode) rather than an error. This is the primitive multiple
independent processes use to pull distinct work off the same board:

```bash
# each of these gets a different task, or nothing once the board is drained -
# never the same task twice, even run concurrently against the same file.
mini-circus task claim --board backlog worker-1
mini-circus task claim --board backlog worker-2
```

Everything else (`assign`, `status`, `update`) is a plain, non-atomic write —
fine for a human or a single controller managing the board, but `claim` is
what you want when several callers are pulling from the same board at once.

## Terminal output

Without `--json`, every command renders through [termimad](https://docs.rs/termimad)
as styled markdown instead of plain aligned columns: lists render as bordered,
auto-sized tables; a task's status/priority/assignee line is bold; comments
render as blockquotes; empty states render in italics. It reads its skin from
your terminal's color support, and reuses circus's accent color for headers
and bold text so the two tools feel like one family. User-supplied text
(titles, descriptions, comment bodies, names) is escaped before being handed
to the markdown renderer, so a title containing `*` or `` ` `` shows up as
that literal character rather than being interpreted as formatting.

This is display-only — it's not meant to be piped anywhere. Use `--json` for
anything that needs to be parsed.

## JSON output

Pass `--json` on any command for machine-readable output. A task:

```json
{
  "id": 1,
  "board_id": 1,
  "title": "Write release notes",
  "description": null,
  "status": "pending",
  "priority": "high",
  "assignee": null,
  "created_at": "2026-08-16T19:13:16.012880Z",
  "updated_at": "2026-08-16T19:13:16.012880Z"
}
```

`task show <id>` returns that same shape with a `comments` array merged in
(a task detail, not a plain task):

```json
{
  "id": 1,
  "board_id": 1,
  "title": "Write release notes",
  "description": null,
  "status": "in_progress",
  "priority": "high",
  "assignee": "alice",
  "created_at": "2026-08-16T19:13:16.012880Z",
  "updated_at": "2026-08-16T19:20:00.000000Z",
  "comments": [
    {
      "id": 1,
      "task_id": 1,
      "author": "alice",
      "body": "Starting on this now.",
      "created_at": "2026-08-16T19:20:00.000000Z"
    }
  ]
}
```

A board:

```json
{
  "id": 1,
  "name": "backlog",
  "description": null,
  "created_at": "2026-08-16T19:13:16.012880Z",
  "updated_at": "2026-08-16T19:13:16.012880Z"
}
```

`task list` / `board list` / `task comment list` print a JSON array of the
above. `task claim` prints a single task object, or `null` if nothing was
available. Timestamps are RFC 3339 UTC. Non-zero exit code on any error, with
the message on stderr — nothing prints to stdout on failure, so `--json`
output is always either a valid JSON value or absent.

## Storage

One SQLite file, WAL mode, `busy_timeout` set to 5s so concurrent writers
from separate processes wait briefly instead of failing immediately on
"database is locked." Default location is `./.mini-circus/board.db` — scoped
to whatever directory you run it from, so it behaves like a per-project board
with zero configuration. Override with `--db <path>` or `$MINI_CIRCUS_DB` if
you want one board shared across directories, or several boards in the same
directory.

## Development

```bash
cargo test -p mini-circus      # store-layer tests (temp SQLite files)
cargo run -p mini-circus -- board create backlog
cargo clippy -p mini-circus --all-targets
```

Code layout:

```
src/
  main.rs     CLI entry point, dispatches to store.rs
  cli.rs      clap argument definitions
  db.rs       SQLite connection + migration setup
  models.rs   Board, Task, Comment, TaskDetail (task + its comments)
  store.rs    all queries, including task-claim atomicity; unit tests live here
  output.rs   builds markdown and renders it via termimad, or prints --json
migrations/   embedded at compile time via sqlx::migrate!
```

### Releasing

Pushing a tag matching `mini-circus-v*` (e.g. `mini-circus-v0.2.0`) triggers
`.github/workflows/mini-circus-release.yml`, which builds the four target
binaries above, packages each as `mini-circus-<target>.tar.gz`, generates a
combined `SHA256SUMS`, and publishes a GitHub Release with all of it attached
— what `install.sh` downloads from. Bump the version in `mini-circus/Cargo.toml`
before tagging.
