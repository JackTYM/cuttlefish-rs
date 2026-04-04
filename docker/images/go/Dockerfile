# syntax=docker/dockerfile:1
# Go 1.22 sandbox image for Cuttlefish
FROM golang:1.22-bookworm AS builder

WORKDIR /tmp/build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

# Final stage
FROM golang:1.22-bookworm

LABEL org.opencontainers.image.title="Cuttlefish Go Sandbox"
LABEL org.opencontainers.image.description="Production-ready Go 1.22 sandbox for Cuttlefish"
LABEL org.opencontainers.image.version="1.0"
LABEL org.opencontainers.image.vendor="Cuttlefish"

# Create non-root user
RUN groupadd -g 1000 sandbox && \
    useradd -u 1000 -g 1000 -m -s /sbin/nologin sandbox

# Set working directory
WORKDIR /workspace

# Verify Go installation
RUN go version

# Change ownership to sandbox user
RUN chown -R sandbox:sandbox /workspace

# Switch to non-root user
USER sandbox

# Keep container running
CMD ["tail", "-f", "/dev/null"]
