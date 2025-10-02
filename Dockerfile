# syntax=docker/dockerfile:1.18
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
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r appuser && useradd -r -g appuser appuser

# Copy binary
COPY --from=builder /app/bin/my-http-server /usr/local/bin/my-http-server
# Copy default runtime templates
# COPY docker/meta /app/meta


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