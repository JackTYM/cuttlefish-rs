# syntax=docker/dockerfile:1
# Python 3.12 sandbox image for Cuttlefish
FROM python:3.12-slim AS builder

WORKDIR /tmp/build

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create virtual environment and install package managers
RUN python -m venv /opt/venv && \
    . /opt/venv/bin/activate && \
    pip install --upgrade pip setuptools wheel && \
    pip install poetry uv

# Final stage
FROM python:3.12-slim

LABEL org.opencontainers.image.title="Cuttlefish Python Sandbox"
LABEL org.opencontainers.image.description="Production-ready Python 3.12 sandbox for Cuttlefish"
LABEL org.opencontainers.image.version="1.0"
LABEL org.opencontainers.image.vendor="Cuttlefish"

# Create non-root user
RUN groupadd -g 1000 sandbox && \
    useradd -u 1000 -g 1000 -m -s /sbin/nologin sandbox

# Set working directory
WORKDIR /workspace

# Copy virtual environment from builder
COPY --from=builder /opt/venv /opt/venv

# Set PATH to use venv
ENV PATH="/opt/venv/bin:$PATH"

# Verify package managers
RUN pip --version && poetry --version && uv --version

# Change ownership to sandbox user
RUN chown -R sandbox:sandbox /workspace

# Switch to non-root user
USER sandbox

# Keep container running
CMD ["tail", "-f", "/dev/null"]
