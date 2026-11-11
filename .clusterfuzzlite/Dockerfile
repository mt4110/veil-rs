FROM gcr.io/oss-fuzz-base/base-builder-rust:latest@sha256:3428d0a90b1e7efbe2ba947bd1d0c9a70594986b4b6553804547f1fb24cfc92f

ENV RUSTUP_TOOLCHAIN=nightly-2026-11-11
RUN rustup toolchain install "$RUSTUP_TOOLCHAIN" --profile minimal

WORKDIR $SRC/veil-rs
COPY Cargo.toml ./
COPY crates/ crates/
COPY fuzz/ fuzz/
COPY .clusterfuzzlite/build.sh $SRC/build.sh
