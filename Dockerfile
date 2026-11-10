# Build Stage
FROM rust:1.92.0-slim-bookworm@sha256:f1f73538ebe623fd3673a35aff3df358ae1084c64c55646516e5b17b321b6c9b AS builder

WORKDIR /usr/src/app

# Install build dependencies (pkg-config, libssl-dev for reqwest/openssl, git for libgit2 if needed)
RUN apt-get update && apt-get install -y git libssl-dev pkg-config && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY crates/ ./crates/

# Build release binary features
# We enable 'wizard' and 'table' as they are user-facing, though wizard is less relevant for CI.
# 'table' is useful for CI logs.
RUN cargo build --release -p veil-cli --features table

# Runtime Stage
FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc

WORKDIR /app

# Install runtime dependencies (openssl, ca-certificates, git)
# git is needed if we use git features (scan history etc)
RUN apt-get update && apt-get install -y openssl ca-certificates git && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/veil /usr/local/bin/veil

# Set entrypoint
ENTRYPOINT ["veil"]
