# Deployment Guide

This directory contains deployment configurations and guides for Cuttlefish.

## Quick Start

### Local Development

```bash
# Start all services with Docker Compose
cd deploy
docker compose up -d

# Access:
# - Marketing Site: http://localhost:3001
# - Web Dashboard: http://localhost:3000
# - API: http://localhost:8080
```

### Environment Setup

1. Copy environment files:
```bash
cp cuttlefish-site/.env.example cuttlefish-site/.env.local
cp cuttlefish-web/.env.example cuttlefish-web/.env.local
```

2. Set required variables:
```bash
export CUTTLEFISH_API_KEY="your-secure-api-key"
export AWS_ACCESS_KEY_ID="your-aws-key"
export AWS_SECRET_ACCESS_KEY="your-aws-secret"
```

## Deployment Options

### Option 1: Vercel (Recommended for Static Site)

**Marketing Site** (`cuttlefish-site/`):

1. Push to GitHub
2. Connect repository to Vercel
3. Vercel auto-detects `vercel.json` configuration
4. Deploy to `cuttlefish.dev`

**Web Dashboard** (`cuttlefish-web/`):

1. Create separate Vercel project
2. Configure environment variables:
   - `NUXT_PUBLIC_API_BASE`: API endpoint
   - `NUXT_PUBLIC_WS_URL`: WebSocket URL
3. Deploy to `app.cuttlefish.dev`

### Option 2: Netlify (Alternative for Static Site)

**Marketing Site**:

1. Connect GitHub repository to Netlify
2. Netlify auto-detects `netlify.toml`
3. Configure build settings:
   - Build command: `npm run generate`
   - Publish directory: `.output/public`
4. Deploy to custom domain

### Option 3: Docker Compose (Self-Hosted)

**Full Stack Deployment**:

```bash
cd deploy

# Set environment variables
export CUTTLEFISH_API_KEY="your-key"
export AWS_ACCESS_KEY_ID="..."
export AWS_SECRET_ACCESS_KEY="..."

# Start services
docker compose up -d

# View logs
docker compose logs -f api
docker compose logs -f web
docker compose logs -f site
```

Services:
- **API** (port 8080): Rust backend with WebSocket
- **Web** (port 3000): Nuxt SSR dashboard
- **Site** (port 3001): Nuxt SSG marketing site
- **Tunnel** (port 8081/8082): Self-hosted access

### Option 4: Cloudflare Pages (Static Site)

**Marketing Site**:

1. Connect GitHub to Cloudflare Pages
2. Configure build:
   - Build command: `npm run generate`
   - Build output directory: `.output/public`
3. Set environment variables in Cloudflare dashboard
4. Deploy to `cuttlefish.dev`

## Configuration Files

### Marketing Site (`cuttlefish-site/`)

| File | Purpose |
|------|---------|
| `vercel.json` | Vercel deployment config (SSG) |
| `netlify.toml` | Netlify deployment config (SSG) |
| `.env.example` | Environment variables template |
| `nuxt.config.ts` | Nuxt configuration (SSG mode) |

### Web Dashboard (`cuttlefish-web/`)

| File | Purpose |
|------|---------|
| `vercel.json` | Vercel deployment config (SSR) |
| `.env.example` | Environment variables template |
| `nuxt.config.ts` | Nuxt configuration (SSR mode) |

### Docker Deployment

| File | Purpose |
|------|---------|
| `deploy/docker-compose.yml` | Full stack orchestration |
| `deploy/Dockerfile` | API server image |
| `deploy/Dockerfile.dashboard` | Web dashboard image |
| `deploy/Dockerfile.site` | Marketing site image |
| `deploy/Dockerfile.tunnel` | Tunnel daemon image |

## Environment Variables

### Marketing Site

```env
NUXT_PUBLIC_API_BASE=https://app.cuttlefish.dev
NUXT_PUBLIC_SITE_URL=https://cuttlefish.dev
NUXT_PUBLIC_ENABLE_MARKETPLACE=true
NUXT_PUBLIC_ENABLE_DOCS=true
```

### Web Dashboard

```env
NUXT_PUBLIC_API_BASE=http://localhost:8080
NUXT_PUBLIC_WS_URL=ws://localhost:8080/ws
NUXT_PUBLIC_SITE_URL=https://app.cuttlefish.dev
NUXT_API_BASE=http://localhost:8080
NUXT_WS_URL=ws://localhost:8080/ws
```

### API Server

```env
CUTTLEFISH_API_KEY=your-secure-api-key
AWS_ACCESS_KEY_ID=your-aws-key
AWS_SECRET_ACCESS_KEY=your-aws-secret
AWS_DEFAULT_REGION=us-east-1
DISCORD_BOT_TOKEN=your-discord-token
```

## Caching Strategy

### Static Assets

- **Marketing Site**: 1 year cache (immutable)
- **Web Dashboard**: 1 year cache for `/_nuxt/*` (immutable)
- **HTML Pages**: 1 hour cache with revalidation

### Cache Headers

```
/assets/*          → Cache-Control: public, max-age=31536000, immutable
/_nuxt/*           → Cache-Control: public, max-age=31536000, immutable
/*                 → Cache-Control: public, max-age=3600, s-maxage=3600
```

## Security Headers

All deployments include:

```
X-Content-Type-Options: nosniff
X-Frame-Options: DENY (site) / SAMEORIGIN (web)
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Content-Security-Policy: [configured per site]
```

## Health Checks

### API Server

```bash
curl http://localhost:8080/health
```

### Web Dashboard

```bash
curl http://localhost:3000
```

### Marketing Site

```bash
curl http://localhost:3001
```

## Monitoring

### Docker Compose

```bash
# View service status
docker compose ps

# View logs
docker compose logs -f [service-name]

# Check health
docker compose exec api curl http://localhost:8080/health
```

### Vercel

- Dashboard: https://vercel.com/dashboard
- Deployments tab shows build logs
- Analytics available for performance metrics

### Netlify

- Dashboard: https://app.netlify.com
- Deploys tab shows build history
- Analytics available for traffic metrics

## Troubleshooting

### Docker Compose Issues

**Services won't start:**
```bash
# Check logs
docker compose logs api

# Verify environment variables
docker compose config

# Rebuild images
docker compose build --no-cache
```

**Port conflicts:**
```bash
# Find process using port
lsof -i :8080

# Change port in docker-compose.yml
# Or stop conflicting service
```

### Build Failures

**Vercel/Netlify build fails:**
1. Check build logs in dashboard
2. Verify environment variables are set
3. Ensure `package.json` scripts are correct
4. Test locally: `npm run build` / `npm run generate`

**Docker build fails:**
1. Check Dockerfile syntax
2. Verify base image availability
3. Check for missing dependencies
4. Build with verbose output: `docker build --progress=plain`

## Production Checklist

- [ ] Environment variables configured in deployment platform
- [ ] API keys and secrets stored securely (not in code)
- [ ] HTTPS/TLS enabled
- [ ] Health checks configured
- [ ] Monitoring and alerting set up
- [ ] Backup strategy for database
- [ ] Log aggregation configured
- [ ] Rate limiting enabled
- [ ] CORS configured correctly
- [ ] Database migrations applied

## See Also

- [Tunnel Proxy Setup](tunnel-proxy.md) — Self-hosted access configuration
- [GitHub Actions](../.github/workflows/) — CI/CD pipelines
- [Docker Compose Override](../deploy/docker-compose.dev.yml) — Development overrides
