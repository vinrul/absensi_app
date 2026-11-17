# syntax=docker/dockerfile:1

FROM rust:1.99-slim-bookworm AS builder
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* rust-toolchain.toml ./
RUN mkdir -p src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

COPY migrations ./migrations
COPY src ./src
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home app
WORKDIR /app

COPY --from=builder /app/target/release/kinerja-api /usr/local/bin/kinerja-api
COPY --from=builder /app/migrations ./migrations

RUN mkdir -p /data/uploads && chown -R app:app /data
USER app
ENV APP__HOST=0.0.0.0 APP__PORT=8080 APP__UPLOAD_DIR=/data/uploads

EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s \
    CMD curl -fsS http://127.0.0.1:8080/health || exit 1
ENTRYPOINT ["kinerja-api"]
