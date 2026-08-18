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

## Writing task descriptions

See [SKILL.md](SKILL.md) for the rule (write for a claimant with zero
shared context: exact scope, the interface/contract, file locations,
acceptance criteria, constraints) and a backend/API example. Two more
worked examples, same principle applied to a different kind of interface:

**Frontend/component task - too terse:**
```bash
mini-circus task create --board web "Add loading spinner"
```

**Frontend/component task - detailed enough to hand off cold:**
```bash
mini-circus --json task create --board web "Add loading state to TaskList" \
  --assignee frontend-worker \
  --description "TaskList component (frontend/src/components/TaskList.tsx)
currently renders nothing while its data is fetching, so the screen is
blank for ~1s on load.
Contract:
  New prop: isLoading: boolean (already returned by the useTasks() hook
  in frontend/src/hooks/useTasks.ts as .isLoading - just wire it through,
  don't add new fetch logic).
  When isLoading is true, render the existing <Spinner size=\"md\" />
  component (frontend/src/components/Spinner.tsx) centered in place of
  the task list, and render nothing else in that state.
  When isLoading is false, current rendering is unchanged.
Acceptance: throttle network in devtools, load the page, see the spinner
until data arrives, then the task list - no layout shift, no console
errors."
```

**Bug-fix task - too terse:**
```bash
mini-circus task create --board api "Fix login bug"
```

**Bug-fix task - detailed enough to hand off cold:**
```bash
mini-circus --json task create --board api "Login returns 500 for emails with a +" \
  --priority urgent --assignee backend-worker \
  --description "Repro: POST /api/auth/login with email
  \"user+test@example.com\" and a correct password returns 500. Same
  password with the plain email (no +) works fine.
Expected: 200 with the normal session response shape (see
  POST /api/auth/login in backend/crates/api/src/auth/handlers.rs) for any
  RFC 5321-valid email, + included.
Suspected cause (unconfirmed): the email normalization step in
  backend/crates/api/src/auth/handlers.rs likely mishandles + in
  to_lowercase()/trim() chain - check there first, but verify against the
  actual code rather than assuming.
Acceptance: the repro request above returns 200; add a test case for a
  +-containing email alongside the existing login tests."
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
