CREATE TABLE boards (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

-- status/priority are plain TEXT (see common::TaskStatus / common::Priority)
-- with a CHECK constraint standing in for the enum, same as circus's schema.
CREATE TABLE tasks (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    board_id    INTEGER NOT NULL REFERENCES boards(id) ON DELETE CASCADE,
    title       TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'pending'
                    CHECK (status IN ('pending', 'in_progress', 'blocked', 'completed')),
    priority    TEXT NOT NULL DEFAULT 'medium'
                    CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    -- Arbitrary free-text name. No user/auth system, so nothing validates it
    -- beyond "non-empty" - that's intentional.
    assignee    TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX idx_tasks_board ON tasks(board_id);
CREATE INDEX idx_tasks_board_status ON tasks(board_id, status);
CREATE INDEX idx_tasks_assignee ON tasks(assignee);
