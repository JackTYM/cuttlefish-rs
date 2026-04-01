FROM rust:1.82-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace

CMD ["tail", "-f", "/dev/null"]
