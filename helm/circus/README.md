# Circus Helm chart

Self-hosted deploy of Circus: an Axum/Postgres backend serving a built React
SPA from a single container image.

## Quickstart

```bash
helm dependency update helm/circus
helm install circus helm/circus \
  --set jwt.secret="$(openssl rand -base64 32)" \
  --set bootstrapAdmin.email=admin@example.com \
  --set bootstrapAdmin.password=change-me-now \
  --set image.repository=ghcr.io/your-org/circus \
  --set image.tag=latest
```

That's enough to get a working instance with a bundled PostgreSQL and an
attachments PVC. `bootstrapAdmin` creates the instance's first superadmin on
first boot only — it's a no-op on every later upgrade once one exists, so
it's safe to leave those values set.

For anything beyond a quick trial, put real values in a `values.yaml` file
instead of `--set`, set `ingress.enabled=true` with a real host/TLS secret,
and set `config.cookieSecure=true` (the default) since you should be behind
TLS.

## Key values

| Key | Default | Notes |
|---|---|---|
| `image.repository` / `image.tag` | `ghcr.io/circus-app/circus` / chart appVersion | Point at your built image |
| `jwt.secret` / `jwt.existingSecret` | `""` | Required. Auto-generated and persisted across upgrades if left blank *and* no existingSecret is set |
| `bootstrapAdmin.email` / `.password` / `.existingSecret` | `""` | The only way to create an instance superadmin |
| `postgresql.enabled` | `true` | Bundled Bitnami PostgreSQL subchart |
| `postgresql.auth.username` / `.database` | `circus` / `circus` | Bundled DB name/user |
| `externalDatabase.url` / `.existingSecret` | `""` | Used when `postgresql.enabled=false` |
| `persistence.enabled` / `.size` | `true` / `5Gi` | PVC for task attachments (local filesystem storage) |
| `ingress.enabled` / `.hosts` / `.tls` | `false` | Standard `networking.k8s.io/v1` Ingress |
| `autoscaling.enabled` | `false` | HPA on CPU utilization |
| `replicaCount` | `1` | See note below on attachments + multiple replicas |

Run `helm show values helm/circus` for the full set.

## How it fits together

- **Migrations** run automatically on pod startup (`db::migrate` in
  `backend/crates/api/src/main.rs`) — there's no separate Helm migration Job.
- **DATABASE_URL**, when using the bundled subchart, is composed at container
  startup from `DB_HOST`/`DB_USER`/`DB_NAME` env vars plus `DB_PASSWORD`
  sourced from the Bitnami subchart's own generated secret — see the
  `command`/`args` override in `templates/deployment.yaml`. This avoids
  duplicating the auto-generated Postgres password into a secret this chart
  manages. With `postgresql.enabled=false`, `DATABASE_URL` is set directly
  from `externalDatabase.*` instead.
- **Attachments** live on a PVC mounted at `/app/data/attachments`
  (`ReadWriteOnce` by default). `replicaCount > 1` only works if your storage
  class supports `ReadWriteMany`, or you accept attachment access being
  inconsistent across replicas — single replica is the supported default.
- **Health checks**: `/healthz` (liveness) is unconditional; `/readyz`
  (readiness) checks the database connection.

## Local testing without a cluster

```bash
helm dependency update helm/circus
helm lint helm/circus
helm template test helm/circus --set jwt.secret=x --set bootstrapAdmin.email=a@b.com --set bootstrapAdmin.password=changeme123
```
