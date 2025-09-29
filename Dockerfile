# syntax=docker/dockerfile:1.7
# Multi-stage Dockerfile for my-http-server (Rust + actix-web)
# - Builder: rust:1.89.0-slim (bookworm)
# - Runtime: debian:bookworm-slim (non-root)
# - BuildKit cache mounts for faster cargo builds
# - HEALTHCHECK using wget
# - Default templates baked in; mount volumes to override

FROM rust:1.90.0-slim AS builder
WORKDIR /app

# Speed up release build without requiring strip in runtime
ENV RUSTFLAGS="-C strip=symbols"

# Pre-fetch deps for better layer cache
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch --locked

# Copy source
COPY src ./src

# Build with BuildKit cache mounts
# Enable BuildKit before building: DOCKER_BUILDKIT=1
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --locked --release && \
    mkdir -p /app/bin && \
    cp -v /app/target/release/my-http-server /app/bin/my-http-server


FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Minimal runtime deps: certs + wget for healthcheck
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates wget \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy binary
COPY --from=builder /app/bin/my-http-server /usr/local/bin/my-http-server

# Default templates so container runs out-of-the-box
RUN set -eux; \
    mkdir -p /app/meta /app/public; \
    printf "%s\n" \
    '<!DOCTYPE html>' \
    '<html>' \
    '  <head>' \
    '    <meta charset="utf-8" />' \
    '    <link rel="stylesheet" type="text/css" media="screen" href="/css/main.css" />' \
    '    <link rel="stylesheet" type="text/css" media="screen" href="/css/github.css" />' \
    '  </head>' \
    '  <body class="markdown-body">' \
    '    {{& body}}' \
    '    <hr />' \
    '    <a href="/">goto root</a>' \
    '    <footer style="text-align: center">' \
    '      <a style="color: rgba(0, 0, 0, 0.489)" href="https://github.com/Paul-16098/my-http-server/">' \
    '        my-http-server v{{server-version}}' \
    '      </a>' \
    '    </footer>' \
    '  </body>' \
    '</html>' \
    > /app/meta/html-t.templating; \
    printf "%s\n" \
    '<!doctype html>' \
    '<html lang="en">' \
    '  <head>' \
    '    <meta charset="utf-8" />' \
    '    <meta name="viewport" content="width=device-width, initial-scale=1" />' \
    '    <title>404 Not Found</title>' \
    '  </head>' \
    '  <body>' \
    '    <h1>404 Not Found</h1>' \
    '    <p>The requested resource was not found on this server.</p>' \
    '  </body>' \
    '</html>' \
    > /app/meta/404.html

# Ensure ownership so appuser can write generated HTML
RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=info
EXPOSE 8080

# Persist content directory if users want to mount their own
VOLUME ["/app/public","/app/meta"]

# Container-internal healthcheck (requires server bind to 0.0.0.0)
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget -qO- http://127.0.0.1:8080/ > /dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/my-http-server"]