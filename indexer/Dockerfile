# ── Build stage ──────────────────────────────────────────────────────────────
FROM rust:1.82-slim AS builder

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY contract ./contract
COPY indexer ./indexer
COPY mobile-sdk ./mobile-sdk

# Cache deps by building a dummy first
RUN mkdir -p indexer/src && echo "fn main(){}" > indexer/src/main.rs \
    && cargo build --release --manifest-path indexer/Cargo.toml \
    && rm -rf indexer/src

COPY indexer/src ./indexer/src
COPY indexer/migrations ./indexer/migrations

RUN touch indexer/src/main.rs && cargo build --release --manifest-path indexer/Cargo.toml

# ── Runtime stage ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/stellar-escrow-indexer ./indexer
COPY indexer/config.toml ./config.toml
COPY indexer/migrations ./migrations

RUN useradd -r -s /bin/false indexer && chown -R indexer:indexer /app
USER indexer

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

CMD ["./indexer", "--config", "config.toml"]
