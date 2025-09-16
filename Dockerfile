# Multi-stage Dockerfile for my-http-server (Rust + actix-web)
# - Builder: Rust stable on Debian (bookworm)
# - Runtime: debian:bookworm-slim, non-root user
# - Provides a full cofg.yaml tuned for container (bind 0.0.0.0, no watch/hot_reload)

FROM rust:1.89.0-slim AS builder
WORKDIR /app

# Pre-fetch dependencies for better layer cache
COPY Cargo.toml ./Cargo.toml
COPY Cargo.lock ./Cargo.lock
RUN cargo fetch --locked

# Copy sources and assets
COPY src ./src

# Build release binary
RUN cargo build --locked --release


FROM debian:bullseye-slim AS runtime
WORKDIR /app

# Minimal runtime deps
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN groupadd -r appuser && useradd -r -g appuser appuser

# Copy binary and assets
COPY --from=builder /app/target/release/my-http-server /usr/local/bin/my-http-server

# Create default template files so the app can run out-of-the-box (no heredoc to keep parser happy)
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

# Ensure ownership so the app user can write HTML outputs, logs, etc.
RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=info
EXPOSE 8080

# Persist content directory if users want to mount their own
VOLUME ["/app/public"]

ENTRYPOINT ["/usr/local/bin/my-http-server"]

