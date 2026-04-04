# syntax=docker/dockerfile:1
# Node.js 22 LTS sandbox image for Cuttlefish
FROM node:22-slim AS builder

WORKDIR /tmp/build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    python3 \
    && rm -rf /var/lib/apt/lists/*

# Install package managers
RUN npm install -g yarn pnpm

# Final stage
FROM node:22-slim

LABEL org.opencontainers.image.title="Cuttlefish Node.js Sandbox"
LABEL org.opencontainers.image.description="Production-ready Node.js 22 LTS sandbox for Cuttlefish"
LABEL org.opencontainers.image.version="1.0"
LABEL org.opencontainers.image.vendor="Cuttlefish"

# Create non-root user
RUN groupadd -g 1000 sandbox && \
    useradd -u 1000 -g 1000 -m -s /sbin/nologin sandbox

# Set working directory
WORKDIR /workspace

# Copy package managers from builder
COPY --from=builder /usr/local/lib/node_modules /usr/local/lib/node_modules
COPY --from=builder /usr/local/bin/yarn /usr/local/bin/yarn
COPY --from=builder /usr/local/bin/pnpm /usr/local/bin/pnpm

# Ensure npm is available
RUN npm --version && yarn --version && pnpm --version

# Change ownership to sandbox user
RUN chown -R sandbox:sandbox /workspace

# Switch to non-root user
USER sandbox

# Keep container running
CMD ["tail", "-f", "/dev/null"]
