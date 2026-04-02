# Cuttlefish Deployment Guide

## Quick Start with Docker Compose

The fastest way to deploy the full Cuttlefish stack:

```bash
cd deploy

# Set required environment variables
export CUTTLEFISH_API_KEY="your-secure-api-key"
export AWS_ACCESS_KEY_ID="your-aws-key"
export AWS_SECRET_ACCESS_KEY="your-aws-secret"
export AWS_DEFAULT_REGION="us-east-1"

# Start all services
docker compose up -d
```

Access:
- Dashboard: http://localhost:3000
- API: http://localhost:8080

## Services

| Service | Port | Description |
|---------|------|-------------|
| api | 8080 | Rust backend (WebSocket + REST) |
| dashboard | 3000 | Nuxt frontend |
| tunnel | 8081/8082 | Self-hosted access tunnel |

## Building Images Manually

```bash
# API server
docker build -t cuttlefish-api -f deploy/Dockerfile .

# Dashboard
docker build -t cuttlefish-dashboard -f deploy/Dockerfile.dashboard cuttlefish-web/

# Tunnel daemon
docker build -t cuttlefish-tunnel -f deploy/Dockerfile.tunnel .
```

## Production Deployment

For production, consider:

1. **Reverse Proxy**: Use Caddy or nginx for TLS termination
2. **Secrets**: Use Docker secrets or a secrets manager
3. **Persistence**: Mount volumes for database and state
4. **Monitoring**: Add health checks and logging

See `docs/deployment/` for detailed guides:
- [Tunnel Proxy Setup](docs/deployment/tunnel-proxy.md)

## GitHub Actions

CI/CD is configured via GitHub Actions:

- `ci.yml`: Runs on every push (tests, clippy, formatting)
- `release.yml`: Creates releases on version tags
- `deploy-site.yml`: Deploys marketing site to GitHub Pages
- `docker.yml`: Builds and pushes Docker images to GHCR

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `CUTTLEFISH_API_KEY` | Yes | API authentication key |
| `AWS_ACCESS_KEY_ID` | If using Bedrock | AWS credentials |
| `AWS_SECRET_ACCESS_KEY` | If using Bedrock | AWS credentials |
| `AWS_DEFAULT_REGION` | If using Bedrock | AWS region |
| `DISCORD_BOT_TOKEN` | If using Discord | Discord bot token |
| `TUNNEL_SECRET_KEY` | If using tunnel | Tunnel authentication secret |
