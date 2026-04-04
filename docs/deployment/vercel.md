# Vercel Deployment Guide

Deploy Cuttlefish to Vercel for fast, serverless hosting.

## Marketing Site (`cuttlefish-site/`)

### Setup

1. **Connect Repository**
   - Go to https://vercel.com/new
   - Select GitHub repository
   - Vercel auto-detects `vercel.json`

2. **Configure Project**
   - Framework: Nuxt
   - Build Command: `npm run generate`
   - Output Directory: `.output/public`

3. **Environment Variables**
   - `NUXT_PUBLIC_API_BASE`: `https://app.cuttlefish.dev`
   - `NUXT_PUBLIC_SITE_URL`: `https://cuttlefish.dev`

4. **Deploy**
   - Click "Deploy"
   - Vercel builds and deploys automatically

### Custom Domain

1. Go to Project Settings → Domains
2. Add custom domain: `cuttlefish.dev`
3. Update DNS records (Vercel provides instructions)
4. Enable auto-renewal for SSL certificate

### Caching

Vercel automatically caches:
- Static assets (`.output/public/assets/*`): 1 year
- HTML pages: 60 seconds (with revalidation)

## Web Dashboard (`cuttlefish-web/`)

### Setup

1. **Create New Project**
   - Go to https://vercel.com/new
   - Select same GitHub repository
   - Vercel auto-detects `vercel.json`

2. **Configure Project**
   - Framework: Nuxt
   - Build Command: `npm run build`
   - Output Directory: `.output/server`
   - Install Command: `npm install`

3. **Environment Variables**
   - `NUXT_PUBLIC_API_BASE`: `https://api.cuttlefish.dev` (or your API URL)
   - `NUXT_PUBLIC_WS_URL`: `wss://api.cuttlefish.dev/ws`
   - `NUXT_PUBLIC_SITE_URL`: `https://app.cuttlefish.dev`
   - `NUXT_API_BASE`: `https://api.cuttlefish.dev`
   - `NUXT_WS_URL`: `wss://api.cuttlefish.dev/ws`

4. **Deploy**
   - Click "Deploy"
   - Vercel builds and deploys

### Custom Domain

1. Go to Project Settings → Domains
2. Add custom domain: `app.cuttlefish.dev`
3. Update DNS records
4. Enable auto-renewal for SSL certificate

### WebSocket Configuration

For WebSocket support:
1. Ensure API server is accessible at configured URL
2. Use `wss://` (secure WebSocket) in production
3. Configure CORS on API server to allow dashboard origin

## Monitoring

### Vercel Dashboard

- **Deployments**: View build logs and deployment history
- **Analytics**: Monitor performance metrics
- **Logs**: Real-time function logs
- **Monitoring**: Error tracking and alerts

### Environment Variables

Update environment variables:
1. Go to Project Settings → Environment Variables
2. Edit or add variables
3. Redeploy for changes to take effect

## Troubleshooting

### Build Fails

**Error: "Cannot find module"**
- Ensure all dependencies in `package.json`
- Run `npm install` locally to verify
- Check for missing environment variables

**Error: "Build timeout"**
- Increase build timeout in Project Settings
- Optimize build process (remove unused dependencies)
- Check for large assets

### Runtime Issues

**WebSocket connection fails**
- Verify API server is accessible
- Check CORS configuration
- Ensure `wss://` is used in production

**API calls fail**
- Verify `NUXT_PUBLIC_API_BASE` is correct
- Check API server health
- Review browser console for errors

## Performance Optimization

### Caching Strategy

```json
{
  "headers": [
    {
      "source": "/_nuxt/(.*)",
      "headers": [
        {
          "key": "Cache-Control",
          "value": "public, max-age=31536000, immutable"
        }
      ]
    }
  ]
}
```

### Image Optimization

- Use Vercel Image Optimization
- Configure in `nuxt.config.ts`:
```typescript
image: {
  provider: 'vercel'
}
```

### Bundle Analysis

```bash
npm run build -- --analyze
```

## Rollback

To rollback to a previous deployment:

1. Go to Deployments tab
2. Find the deployment to rollback to
3. Click "Promote to Production"

## See Also

- [Vercel Documentation](https://vercel.com/docs)
- [Nuxt on Vercel](https://nuxt.com/deploy/vercel)
- [Environment Variables](https://vercel.com/docs/concepts/projects/environment-variables)
