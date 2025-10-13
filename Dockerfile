# syntax=docker/dockerfile:1.19
# Multi-stage Dockerfile for my-http-server (Rust + actix-web)
# - Builder: rust:1.90.0-slim (bookworm)
# - Runtime: debian:bookworm-slim (non-root)
# - BuildKit cache mounts for faster cargo builds
# - HEALTHCHECK using wget
# - Default templates baked in; mount volumes to override
# - TLS support: mount certificate and key files, use --tls-cert and --tls-key args



FROM rust:slim AS planner

WORKDIR /app

RUN cargo install cargo-chef
COPY Cargo.toml Cargo.lock ./
RUN cargo chef prepare  --recipe-path recipe.json

FROM rust:slim AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

ENV IN_DOCKER=true

# Pre-fetch deps for better layer cache
COPY Cargo.toml Cargo.lock ./
COPY build.rs build.rs ./
RUN cargo fetch --locked

# Copy source
COPY src ./src

COPY meta ./meta

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
COPY meta /app/meta


# Ensure ownership so appuser can write generated HTML
RUN chown -R appuser:appuser /app
USER appuser

ENV RUST_LOG=info
EXPOSE 8080
EXPOSE 8443

# Persist content directory if users want to mount their own
# Mount TLS certificates if needed: -v /path/to/cert.pem:/app/cert.pem:ro -v /path/to/key.pem:/app/key.pem:ro
VOLUME ["/app/public","/app/meta"]

# Container-internal healthcheck (requires server bind to 0.0.0.0)
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD wget -qO- http://127.0.0.1:8080/ > /dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/my-http-server", "--ip", "0.0.0.0"]