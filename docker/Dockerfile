# syntax=docker/dockerfile:1

# ---- frontend build --------------------------------------------------
FROM node:22-slim AS frontend-build
WORKDIR /app/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# ---- backend build -----------------------------------------------------
FROM rust:1-slim-bookworm AS backend-build
# `ring` (pulled in via rustls/jsonwebtoken) has a C build script; the slim
# image doesn't ship a compiler by default.
RUN apt-get update \
    && apt-get install -y --no-install-recommends build-essential \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY backend/ backend/
RUN cargo build --release -p api

# ---- runtime -------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --create-home --uid 1000 --shell /usr/sbin/nologin circus

COPY --from=backend-build /app/target/release/circus-api /usr/local/bin/circus-api
COPY --from=frontend-build /app/frontend/dist /app/static

ENV STATIC_DIR=/app/static \
    STORAGE_DIR=/app/data/attachments \
    BIND_ADDR=0.0.0.0:8080

RUN mkdir -p /app/data/attachments && chown -R circus:circus /app
USER circus
WORKDIR /app

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s \
    CMD curl -f http://127.0.0.1:8080/healthz || exit 1

ENTRYPOINT ["circus-api"]
