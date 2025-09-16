# Multi-stage Dockerfile for my-http-server (Rust + actix-web)
# - Builder: Rust stable on Debian (bookworm)
# - Runtime: debian:bookworm-slim, non-root user
# - Provides a full cofg.yaml tuned for container (bind 0.0.0.0, no watch/hot_reload)

FROM rust:1.89.0-slim AS builder
WORKDIR /app

# Pre-fetch dependencies for better layer cache
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
    printf '%s\n' \
    '<!doctype html>' \
    '<html lang="en">' \
    '  <head>' \
    '    <meta charset="utf-8" />' \
    '    <meta name="viewport" content="width=device-width, initial-scale=1" />' \
    '    <title>{{ title | default: "my-http-server" }}</title>' \
    '    <style>' \
    '      body { font-family: system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, "Apple Color Emoji", "Segoe UI Emoji"; margin: 2rem auto; max-width: 900px; padding: 0 1rem; }' \
    '      nav { margin-bottom: 1rem; }' \
    '      pre, code { background: #f6f8fa; }' \
    '      .path { color: #888; font-size: .9em; }' \
    '    </style>' \
    '  </head>' \
    '  <body>' \
    '    <nav class="path">{{ path | default: "index" }} — v{{ server-version }}</nav>' \
    '    <main>' \
    '      {{ body | safe }}' \
    '    </main>' \
    '  </body>' \
    '</html>' \
    > /app/meta/html-t.templating; \
    printf '%s\n' \
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

# Provide a container-friendly default configuration (complete schema)
# Note: The application expects a full config file; do not trim fields.
RUN set -eux && cat > /app/cofg.yaml <<'YAML'
addrs:
	ip: 0.0.0.0
	port: 8080
middleware:
	normalize_path: true
	compress: true
	logger:
		enabling: true
		format: '%{url}xi %s "%{Referer}i" "%{User-Agent}i"'
watch: false
templating:
	hot_reload: false
	value:
		# - "name:value"
		# - "name:env:ENV_VALUE"
toc:
	make_toc: true
	path: index.html
	ext:
		- html
		- pdf
public_path: ./public/
YAML

# Ensure ownership so the app user can write HTML outputs, logs, etc.
RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=info
EXPOSE 8080

# Persist content directory if users want to mount their own
VOLUME ["/app/public"]

ENTRYPOINT ["/usr/local/bin/my-http-server"]

