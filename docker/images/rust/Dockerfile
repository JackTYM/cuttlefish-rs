# syntax=docker/dockerfile:1
# Rust 1.82+ sandbox image for Cuttlefish
FROM rust:1.82-slim AS builder

WORKDIR /tmp/build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Install additional Rust tools
RUN rustup component add rustfmt clippy

# Final stage
FROM rust:1.82-slim

LABEL org.opencontainers.image.title="Cuttlefish Rust Sandbox"
LABEL org.opencontainers.image.description="Production-ready Rust 1.82+ sandbox for Cuttlefish"
LABEL org.opencontainers.image.version="1.0"
LABEL org.opencontainers.image.vendor="Cuttlefish"

# Create non-root user
RUN groupadd -g 1000 sandbox && \
    useradd -u 1000 -g 1000 -m -s /sbin/nologin sandbox

# Set working directory
WORKDIR /workspace

# Copy Rust toolchain from builder
COPY --from=builder /usr/local/rustup /usr/local/rustup
COPY --from=builder /usr/local/cargo /usr/local/cargo

# Verify Rust tools
RUN rustc --version && cargo --version && rustfmt --version && clippy-driver --version

# Change ownership to sandbox user
RUN chown -R sandbox:sandbox /workspace

# Switch to non-root user
USER sandbox

# Keep container running
CMD ["tail", "-f", "/dev/null"]
