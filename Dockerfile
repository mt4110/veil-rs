# Build Stage
FROM rust:1.83-slim-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies (pkg-config, libssl-dev for reqwest/openssl, git for libgit2 if needed)
RUN apt-get update && apt-get install -y pkg-config libssl-dev git && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/
COPY veil.toml .

# Build release binary features
# We enable 'wizard' and 'table' as they are user-facing, though wizard is less relevant for CI.
# 'table' is useful for CI logs.
RUN cargo build --release -p veil-cli --features table

# Runtime Stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (openssl, ca-certificates, git)
# git is needed if we use git features (scan history etc)
RUN apt-get update && apt-get install -y openssl ca-certificates git && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/veil-cli /usr/local/bin/veil-cli

# Set entrypoint
ENTRYPOINT ["veil-cli"]
