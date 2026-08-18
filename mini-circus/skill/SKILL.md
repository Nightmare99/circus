---
description: Use the mini-circus CLI to track work on a lightweight local task board - create boards, create/list/update tasks, atomically claim work, add comments. Reach for this whenever the user asks to set up or use a task board, track work items in a project, hand off / pick up tasks, report progress on something, or coordinate work that spans multiple sessions or processes against the same board. Also load it whenever a `.mini-circus/board.db` file or a `mini-circus` invocation shows up in this project, since that means a board already exists here.
allowed-tools: Bash(mini-circus *)
---

# mini-circus

A single-binary, single-SQLite-file task board with no accounts and no
permissions - anyone with access to the database file can create boards and
tasks and assign them to any free-text name. There is no server and no
config beyond an optional path override. Full reference for exact command
syntax and JSON shapes: [reference.md](reference.md).

## Is it installed?

```bash
command -v mini-circus >/dev/null 2>&1 || echo "not installed"
```

If missing, install it (one file, no sudo, no build step needed):

```bash
curl -fsSL https://raw.githubusercontent.com/Nightmare99/circus/main/mini-circus/install.sh | sh
```

That puts the binary at `~/.local/bin/mini-circus`. If it's not immediately
on `PATH` after installing, the installer says so and prints the export line
- run that, or just call the binary by its full path for the rest of this
session.

## Is there already a board here?

```bash
ls .mini-circus/board.db 2>/dev/null && mini-circus board list
```

mini-circus scopes its database to the current working directory by default
(`./.mini-circus/board.db`, auto-created on first write). If one already
exists in this project, work off it rather than creating a second board that
fragments the task list - `mini-circus board list` shows what's there.

## Always use `--json`

Everything below assumes `--json`. Without it, mini-circus renders styled
markdown for a human terminal (colors, box-drawn tables) - readable for a
person, extra work to parse reliably as an agent. `--json` gives clean,
stable-shaped output on every command; parse it, don't scrape the plain-text
form. Errors always go to stderr with a non-zero exit code regardless of
`--json`, so check the exit code, not stdout, to detect failure.

## Core workflow

**Set up a board once per project** (skip if one already exists - see above):

```bash
mini-circus board create <name> --description "<what this board is for>"
```

**Add work to it - always with `--assignee`, and write the description
like a spec, not a label.**

Pass `--assignee <name>` on every `task create` - decide who's doing the
work at creation time rather than leaving it to be picked up later. An
unassigned task is invisible to `task list --assignee <name>` and easy to
lose track of; a named owner from the start is not. Only omit `--assignee`
when you're deliberately building a shared pool of interchangeable work for
`task claim` to hand out (see below) - that's the one intentional exception,
not the default.

Whoever picks this up is a *separate process or session with none of your
context*. The description is the only information transfer that happens -
if it's not in there, the claimant doesn't know it. `"Fix the auth bug"` or
`"Add user endpoint"` is not a task description; it's a title that happens
to be longer. Write every task as if handing it to someone who will start
cold:

- **Exactly what to do** - which function/endpoint/component, current
  behavior vs. required behavior, not just the symptom or the goal.
- **The contract, for anything with an interface.** For a backend/API
  task: method + path, full request shape, full response shape (every
  field, its type, which are optional), status codes and error response
  shapes, and which existing types/models it touches or must reuse rather
  than duplicate. For a frontend/component task: props/inputs, the states
  it must handle, exact behavior per interaction. For a CLI/library task:
  function signature, inputs/outputs, error cases. No interface, no task -
  underspecifying it here is where rework comes from.
- **Where** - file paths or module names, if already known, so the
  claimant isn't searching from zero.
- **Acceptance criteria** - concretely: a request/response that should
  round-trip, a test that should pass, an observable behavior. Not "should
  work."
- **Constraints** - things to leave alone, conventions to follow,
  dependencies on other tasks.

```bash
mini-circus --json task create --board api "Add GET /users/:id endpoint" \
  --priority high --assignee backend-worker \
  --description "Add to backend/crates/api/src/users/handlers.rs.
Contract:
  Request: path param id (uuid)
  200: { \"id\": uuid, \"email\": string, \"display_name\": string, \"created_at\": rfc3339 }
  404: { \"error\": \"user not found\" } if id doesn't exist
  400: { \"error\": \"invalid id\" } if id isn't a valid uuid
Reuse the existing UserRow struct in backend/crates/db/src/models.rs - don't
add fields to the response beyond what's listed above.
Acceptance: GET with an existing user id returns 200 with the shape above;
a random uuid returns 404; a non-uuid string returns 400."
```

That's the depth to aim for regardless of task type - a frontend, docs, or
infra task needs the same rigor for whatever *its* interface is (props,
config keys, CLI flags, file formats), not just backend work. More worked
examples (frontend/component, bug-fix) are in
[reference.md](reference.md#writing-task-descriptions).

Task ids are global integers, not per-board - `task show`, `task assign`,
`task status`, etc. never need `--board`, the id alone is enough.

**See what's on the board:**

```bash
mini-circus --json task list --board <name>
mini-circus --json task list --board <name> --status pending
mini-circus --json task list --board <name> --assignee <name>
```

**Pick up the next available task** - only relevant for tasks that were
deliberately left unassigned (the one intentional exception to "always set
`--assignee`" above: a shared pool of interchangeable work, created without
`--assignee` on purpose because you don't know or don't care in advance who
picks each one up). It atomically finds the oldest unassigned `pending`
task, assigns it to `<name>`, and marks it `in_progress`, in one SQL
statement - safe to call from several processes against the same board at
once; each call gets a different task, or `null` if the board is drained:

```bash
mini-circus --json task claim --board <name> <your-name>
```

Pick a stable, identifying `<name>` for yourself and reuse it consistently
within a session (so `task list --assignee <name>` reliably shows your own
work) - don't invent a new name per command.

**Report progress and hand off work:**

```bash
mini-circus --json task comment add <id> "<progress note>" --author <name>
mini-circus --json task status <id> completed|blocked|in_progress|pending
mini-circus --json task assign <id> <name>     # hand off explicitly
mini-circus --json task unassign <id>          # put back for someone else to claim
```

**Check a specific task's full state, including its comment history:**

```bash
mini-circus --json task show <id>
```

## Things worth knowing

- Default to `--assignee` on every `task create`. Leave it off only when
  you're intentionally creating unclaimed pool work for `task claim`.
- Statuses are fixed: `pending`, `in_progress`, `blocked`, `completed`. No
  custom columns.
- Assignees and comment authors are arbitrary strings - there's no account
  system validating them, so a typo silently creates a new "identity"
  rather than erroring. Reuse exact names.
- Boards can be referenced by name or numeric id anywhere a board is asked
  for; tasks and comments only by numeric id.
- `task claim` is the only mutation that's meaningfully different from a
  plain write (`assign`/`status`/`update`) - it's the one built for
  multiple concurrent callers pulling distinct work off the same board.
  Prefer it over "list, then assign the first pending one" when you might
  be racing another process, since that pattern isn't atomic and can
  double-assign.
- See [reference.md](reference.md) for the complete command list, every
  flag, and the exact JSON shape of a board/task/comment.
