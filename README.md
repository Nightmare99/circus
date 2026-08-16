# Circus

A simple, self-hostable issue tracker: Organizations → Projects → Board → Tasks.
No epics, no story points — just tasks with a status, tags, an assignee, a reporter,
comments, and attachments.

## Stack

- **Backend**: Rust (Axum + SQLx + PostgreSQL), JWT auth, argon2 password hashing
- **Frontend**: React + TypeScript + Vite, TanStack Query, Tailwind
- **Deploy**: single Docker image (API serves the built SPA), Helm chart for k8s

## Repo layout

```
backend/crates/domain/   pure RBAC + core types, no I/O
backend/crates/db/       SQLx models + migrations
backend/crates/api/      Axum HTTP server (binary: circus-api)
frontend/                React app
helm/circus/              Helm chart
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

## Status

Phase 0 (scaffolding) complete: workspace builds, migrations run against a real
Postgres, `/healthz` and `/readyz` wired end-to-end through the frontend dev proxy.
See project history for the phased build-out plan (auth → org/RBAC → projects →
tasks → frontend → Docker → Helm → CI).
