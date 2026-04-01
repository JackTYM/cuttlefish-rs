FROM node:22-slim

# Install common tools
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

# Keep container running
CMD ["tail", "-f", "/dev/null"]
