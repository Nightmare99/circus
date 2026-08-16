# Circus

A simple, self-hostable issue tracker: Organizations → Projects → Board → Tasks.
No epics, no story points — just tasks with a status, tags, an assignee, a reporter,
comments, and attachments.

This repo also has **mini-circus**, a separate, much smaller CLI tool that shares
the same task-status vocabulary but has no accounts, orgs, or permissions — see
[below](#mini-circus).

## Stack

- **Backend**: Rust (Axum + SQLx + PostgreSQL), JWT auth, argon2 password hashing
- **Frontend**: React + TypeScript + Vite, TanStack Query, Tailwind
- **Deploy**: single Docker image (API serves the built SPA), Helm chart for k8s

## Repo layout

```
common/                  TaskStatus/Priority - shared by circus and mini-circus
backend/crates/domain/   RBAC + circus-specific types, built on common
backend/crates/db/       SQLx models + migrations (Postgres)
backend/crates/api/      Axum HTTP server (binary: circus-api)
frontend/                React app
mini-circus/             standalone CLI task board (binary: mini-circus, SQLite)
helm/circus/             Helm chart
docker/                  Dockerfile, docker-compose.yml (local dev)
```

## Local development

```bash
cp .env.example .env

# Postgres
docker compose -f docker/docker-compose.yml up -d

# Backend (applies migrations on startup)
cargo run -p api

# Frontend (proxies /api, /healthz, /readyz to the backend on :8080)
cd frontend && npm install && npm run dev
```

Backend listens on `:8080`, frontend dev server on `:5173`.

## mini-circus

A minimal task board with no accounts and no permissions: anyone with access to
the database file can create boards and tasks and assign them to any (free-text)
name. Built for coordinating work across multiple independent processes reading
and writing the same board concurrently — every mutation is a single atomic SQL
statement, so concurrent writers can't corrupt or double-claim state. It shares
`TaskStatus`/`Priority` with circus via the `common` crate, but nothing else:
no orgs, no users, no RBAC, no web server. Just a SQLite file and a CLI.

```bash
curl -fsSL https://raw.githubusercontent.com/Nightmare99/circus/main/mini-circus/install.sh | sh
```

**[mini-circus/README.md](mini-circus/README.md)** has the full picture:
installer options, every command, JSON output schemas, storage/concurrency
details, and how to build from source or cut a release.

## Status

Phases 0-9 of the original circus build-out are done: auth, multi-tenant RBAC,
projects/boards, tasks (comments, tags, attachments), the React frontend,
Docker image, Helm chart, CI (build/lint/test + image + chart publishing), and
WebSocket-based live board updates. mini-circus was added afterward as a
separate, deliberately smaller sibling project.
