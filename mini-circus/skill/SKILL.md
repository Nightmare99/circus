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

**Add work to it:**

```bash
mini-circus --json task create --board <name> "<title>" \
  --description "<details>" --priority low|medium|high|urgent
```

Task ids are global integers, not per-board - `task show`, `task assign`,
`task status`, etc. never need `--board`, the id alone is enough.

**See what's on the board:**

```bash
mini-circus --json task list --board <name>
mini-circus --json task list --board <name> --status pending
mini-circus --json task list --board <name> --assignee <name>
```

**Pick up the next available task** - this is the one to reach for when
picking up work from a shared board rather than working a specific
already-known task id. It atomically finds the oldest unassigned `pending`
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
