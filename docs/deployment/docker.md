# Docker Deployment Guide

Deploy Cuttlefish using Docker and Docker Compose for self-hosted environments.

## Quick Start

```bash
cd deploy

# Set environment variables
export CUTTLEFISH_API_KEY="your-secure-api-key"
export AWS_ACCESS_KEY_ID="your-aws-key"
export AWS_SECRET_ACCESS_KEY="your-aws-secret"
export AWS_DEFAULT_REGION="us-east-1"

# Start all services
docker compose up -d

# View logs
docker compose logs -f
```

Access:
- Marketing Site: http://localhost:3001
- Web Dashboard: http://localhost:3000
- API: http://localhost:8080

## Services

### API Server (Port 8080)

Rust backend with WebSocket support.

**Environment Variables:**
- `RUST_LOG`: Log level (default: `info`)
- `CUTTLEFISH_API_KEY`: API authentication key (required)
- `AWS_ACCESS_KEY_ID`: AWS credentials (if using Bedrock)
- `AWS_SECRET_ACCESS_KEY`: AWS credentials (if using Bedrock)
- `AWS_DEFAULT_REGION`: AWS region (default: `us-east-1`)
- `DISCORD_BOT_TOKEN`: Discord bot token (optional)

**Health Check:**
```bash
curl http://localhost:8080/health
```

### Web Dashboard (Port 3000)

Nuxt SSR application for project management.

**Environment Variables:**
- `NODE_ENV`: `production`
- `NUXT_PUBLIC_API_BASE`: API base URL
- `NUXT_PUBLIC_WS_URL`: WebSocket URL
- `NUXT_PUBLIC_SITE_URL`: Dashboard URL

**Health Check:**
```bash
curl http://localhost:3000
```

### Marketing Site (Port 3001)

Nuxt SSG static site.

**Environment Variables:**
- `NODE_ENV`: `production`
- `NUXT_PUBLIC_API_BASE`: API base URL
- `NUXT_PUBLIC_SITE_URL`: Site URL

**Health Check:**
```bash
curl http://localhost:3001
```

### Tunnel Daemon (Ports 8081/8082)

Self-hosted access tunnel for remote connections.

**Environment Variables:**
- `RUST_LOG`: Log level
- `TUNNEL_SECRET_KEY`: Tunnel authentication secret

## Building Images

### Build All Images

```bash
docker compose build
```

### Build Specific Image

```bash
# API server
docker build -t cuttlefish-api -f deploy/Dockerfile .

# Web dashboard
docker build -t cuttlefish-web -f deploy/Dockerfile.dashboard cuttlefish-web/

# Marketing site
docker build -t cuttlefish-site -f deploy/Dockerfile.site cuttlefish-site/

# Tunnel daemon
docker build -t cuttlefish-tunnel -f deploy/Dockerfile.tunnel .
```

## Production Deployment

### Environment Setup

Create `.env` file:

```bash
CUTTLEFISH_API_KEY=your-secure-api-key
AWS_ACCESS_KEY_ID=your-aws-key
AWS_SECRET_ACCESS_KEY=your-aws-secret
AWS_DEFAULT_REGION=us-east-1
DISCORD_BOT_TOKEN=your-discord-token
TUNNEL_SECRET_KEY=your-tunnel-secret
```

### Reverse Proxy (Nginx)

Configure Nginx for subdomain routing:

```nginx
upstream api {
  server api:8080;
}

upstream web {
  server web:3000;
}

upstream site {
  server site:3000;
}

server {
  listen 80;
  server_name cuttlefish.dev;
  
  location / {
    proxy_pass http://site;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
  }
}

server {
  listen 80;
  server_name app.cuttlefish.dev;
  
  location / {
    proxy_pass http://web;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
  }
  
  location /ws {
    proxy_pass http://api;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
  }
}

server {
  listen 80;
  server_name api.cuttlefish.dev;
  
  location / {
    proxy_pass http://api;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
  }
  
  location /ws {
    proxy_pass http://api;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
  }
}
```

### SSL/TLS with Let's Encrypt

```bash
# Install Certbot
sudo apt-get install certbot python3-certbot-nginx

# Generate certificates
sudo certbot certonly --standalone -d cuttlefish.dev -d app.cuttlefish.dev -d api.cuttlefish.dev

# Configure Nginx with SSL
# Update nginx.conf with ssl_certificate and ssl_certificate_key
```

### Systemd Service

Create `/etc/systemd/system/cuttlefish.service`:

```ini
[Unit]
Description=Cuttlefish Docker Compose
Requires=docker.service
After=docker.service

[Service]
Type=simple
WorkingDirectory=/opt/cuttlefish/deploy
ExecStart=/usr/bin/docker compose up
ExecStop=/usr/bin/docker compose down
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl daemon-reload
sudo systemctl enable cuttlefish
sudo systemctl start cuttlefish
```

## Monitoring

### View Logs

```bash
# All services
docker compose logs -f

# Specific service
docker compose logs -f api
docker compose logs -f web
docker compose logs -f site

# Last 100 lines
docker compose logs --tail=100 api
```

### Check Service Status

```bash
docker compose ps
```

### Health Checks

```bash
# API
docker compose exec api curl http://localhost:8080/health

# Web
docker compose exec web curl http://localhost:3000

# Site
docker compose exec site curl http://localhost:3000
```

## Maintenance

### Update Services

```bash
# Pull latest images
docker compose pull

# Rebuild images
docker compose build --no-cache

# Restart services
docker compose restart
```

### Database Backup

```bash
# Backup SQLite database
docker compose exec api cp /data/cuttlefish.db /data/cuttlefish.db.backup

# Copy to host
docker cp cuttlefish-api:/data/cuttlefish.db ./backup/
```

### Clean Up

```bash
# Remove stopped containers
docker compose down

# Remove unused images
docker image prune

# Remove unused volumes
docker volume prune
```

## Troubleshooting

### Services Won't Start

```bash
# Check logs
docker compose logs api

# Verify environment variables
docker compose config

# Rebuild images
docker compose build --no-cache
```

### Port Conflicts

```bash
# Find process using port
lsof -i :8080

# Change port in docker-compose.yml
# Or stop conflicting service
```

### WebSocket Connection Fails

```bash
# Check API logs
docker compose logs api

# Verify WebSocket endpoint
curl -i -N -H "Connection: Upgrade" -H "Upgrade: websocket" http://localhost:8080/ws
```

### Out of Disk Space

```bash
# Check disk usage
docker system df

# Clean up unused resources
docker system prune -a
```

## Performance Tuning

### Resource Limits

Configure in `docker-compose.yml`:

```yaml
services:
  api:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
```

### Database Optimization

```bash
# Analyze database
docker compose exec api sqlite3 /data/cuttlefish.db "ANALYZE;"

# Vacuum database
docker compose exec api sqlite3 /data/cuttlefish.db "VACUUM;"
```

## See Also

- [Docker Documentation](https://docs.docker.com)
- [Docker Compose Documentation](https://docs.docker.com/compose/)
- [Nginx Documentation](https://nginx.org/en/docs/)
- [Let's Encrypt](https://letsencrypt.org/)
