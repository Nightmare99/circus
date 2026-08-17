# mini-circus command reference

Full human-facing docs: [mini-circus/README.md](https://github.com/Nightmare99/circus/blob/main/mini-circus/README.md).
This file is the terse version for quick lookup.

Every command accepts `--json` (recommended for programmatic use) and
`--db <path>` (default: `$MINI_CIRCUS_DB`, else `./.mini-circus/board.db`).
Non-zero exit code + message on stderr on any error.

## Boards

`<board>` = a board's name or numeric id, interchangeably, everywhere.

```
mini-circus board create <name> [--description <text>]
mini-circus board list
mini-circus board show <board>
mini-circus board delete <board>      # also deletes all of its tasks
```

## Tasks

Task ids are global integers (unique across every board, not per-board), so
every id-only command below never needs `--board`.

```
mini-circus task create --board <board> <title>
    [--description <text>] [--priority low|medium|high|urgent] [--assignee <name>]

mini-circus task list --board <board>
    [--status pending|in_progress|blocked|completed] [--assignee <name>]

mini-circus task show <id>            # includes this task's comments

mini-circus task update <id>
    [--title <text>] [--description <text>] [--priority low|medium|high|urgent]

mini-circus task status <id> <pending|in_progress|blocked|completed>

mini-circus task assign <id> <name>
mini-circus task unassign <id>

mini-circus task claim --board <board> <name>
    # atomic: assigns + sets in_progress on the oldest unassigned pending
    # task; prints null / nothing if none available. Safe under concurrency.

mini-circus task delete <id>
```

## Comments

```
mini-circus task comment add <task-id> <body> --author <name>
mini-circus task comment list <task-id>
mini-circus task comment delete <id>
```

## JSON shapes

**Board:**
```json
{
  "id": 1,
  "name": "backlog",
  "description": null,
  "created_at": "2026-08-16T19:13:16.012880Z",
  "updated_at": "2026-08-16T19:13:16.012880Z"
}
```

**Task** (from `create`, `list`, `update`, `status`, `assign`, `unassign`, `claim`):
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

**Task detail** (from `task show <id>` only - same shape plus `comments`):
```json
{
  "id": 1, "board_id": 1, "title": "Write release notes", "description": null,
  "status": "in_progress", "priority": "high", "assignee": "alice",
  "created_at": "...", "updated_at": "...",
  "comments": [
    { "id": 1, "task_id": 1, "author": "alice", "body": "Starting on this now.", "created_at": "..." }
  ]
}
```

**Comment** (from `comment add`, `comment list`):
```json
{ "id": 1, "task_id": 1, "author": "alice", "body": "Starting on this now.", "created_at": "..." }
```

`task list` / `board list` / `comment list` return a JSON array of the
matching shape above. `task claim` returns a single task object or `null`.
Timestamps are RFC 3339 UTC.

## Errors

Common failure modes and what they mean:

| Message | Cause |
|---|---|
| `board "<x>" not found` | no board with that name or id |
| `task <n> not found` | no task with that id |
| `comment <n> not found` | no comment with that id |
| `a board named "<x>" already exists` | board names are unique |
| `invalid value '<x>' for '--priority <PRIORITY>'` | must be one of low/medium/high/urgent |
| `invalid value '<x>' for '<STATUS>'` | must be one of pending/in_progress/blocked/completed |

## Concurrency

SQLite in WAL mode with a 5s busy-timeout, so concurrent callers from
separate processes wait briefly rather than erroring immediately on
"database is locked." `task claim` is a single atomic `UPDATE ... WHERE id =
(SELECT ...) RETURNING ...` statement - the only command safe to call from
multiple processes racing for the same work without a double-assignment.
Every other mutation (`assign`, `status`, `update`) is a plain write: fine
for one caller managing the board, not for several callers contending over
the same task.
