-- Enums
--
-- task_status/task_priority are intentionally plain TEXT with a CHECK
-- constraint rather than native Postgres enum types: the Rust types for both
-- columns live in the `common` crate and are shared with mini-circus (which
-- runs on SQLite, with no equivalent to native Postgres enums), so both
-- backends bind them as text.

CREATE TYPE instance_role AS ENUM ('user', 'superadmin');
CREATE TYPE org_role AS ENUM ('member', 'admin', 'owner');
CREATE TYPE project_role AS ENUM ('viewer', 'contributor', 'lead');

-- Identity

CREATE TABLE users (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email           TEXT NOT NULL UNIQUE,
    password_hash   TEXT NOT NULL,
    display_name    TEXT NOT NULL,
    instance_role   instance_role NOT NULL DEFAULT 'user',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Tenancy

CREATE TABLE orgs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            TEXT NOT NULL,
    slug            TEXT NOT NULL UNIQUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE org_memberships (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role            org_role NOT NULL DEFAULT 'member',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, user_id)
);

CREATE INDEX idx_org_memberships_user ON org_memberships(user_id);

CREATE TABLE invites (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    email           TEXT NOT NULL,
    role            org_role NOT NULL DEFAULT 'member',
    token           TEXT NOT NULL UNIQUE,
    invited_by      UUID NOT NULL REFERENCES users(id),
    expires_at      TIMESTAMPTZ NOT NULL,
    accepted_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_invites_org ON invites(org_id);

-- Projects & boards (board is implicit: a project's tasks grouped by status)

CREATE TABLE projects (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    key             TEXT NOT NULL,
    name            TEXT NOT NULL,
    description     TEXT,
    next_task_number BIGINT NOT NULL DEFAULT 1,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, key)
);

CREATE INDEX idx_projects_org ON projects(org_id);

CREATE TABLE project_memberships (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role            project_role NOT NULL DEFAULT 'contributor',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, user_id)
);

CREATE INDEX idx_project_memberships_user ON project_memberships(user_id);

-- Tags

CREATE TABLE tags (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name            TEXT NOT NULL,
    color           TEXT NOT NULL DEFAULT '#6b7280',
    UNIQUE (project_id, name)
);

-- Tasks (TODOs)

CREATE TABLE tasks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id          UUID NOT NULL REFERENCES orgs(id) ON DELETE CASCADE,
    project_id      UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    task_number     BIGINT NOT NULL,
    title           TEXT NOT NULL,
    description     TEXT,
    status          TEXT NOT NULL DEFAULT 'pending'
                        CHECK (status IN ('pending', 'in_progress', 'blocked', 'completed')),
    priority        TEXT NOT NULL DEFAULT 'medium'
                        CHECK (priority IN ('low', 'medium', 'high', 'urgent')),
    assignee_id     UUID REFERENCES users(id) ON DELETE SET NULL,
    reporter_id     UUID NOT NULL REFERENCES users(id),
    due_date        DATE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (project_id, task_number)
);

CREATE INDEX idx_tasks_project_status ON tasks(project_id, status);
CREATE INDEX idx_tasks_org ON tasks(org_id);
CREATE INDEX idx_tasks_assignee ON tasks(assignee_id);

CREATE TABLE task_tags (
    task_id         UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    tag_id          UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (task_id, tag_id)
);

-- Comments

CREATE TABLE comments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id         UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    author_id       UUID NOT NULL REFERENCES users(id),
    body            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_comments_task ON comments(task_id);

-- Attachments (stored on disk/PVC via a storage key; see storage abstraction in api crate)

CREATE TABLE attachments (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id         UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    uploaded_by     UUID NOT NULL REFERENCES users(id),
    file_name       TEXT NOT NULL,
    content_type    TEXT NOT NULL,
    size_bytes      BIGINT NOT NULL,
    storage_key     TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_attachments_task ON attachments(task_id);
