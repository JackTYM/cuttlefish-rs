# Netlify Deployment Guide

Deploy Cuttlefish to Netlify for fast, serverless hosting with advanced features.

## Marketing Site (`cuttlefish-site/`)

### Setup

1. **Connect Repository**
   - Go to https://app.netlify.com
   - Click "New site from Git"
   - Select GitHub repository
   - Netlify auto-detects `netlify.toml`

2. **Configure Build**
   - Build command: `npm run generate`
   - Publish directory: `.output/public`
   - Node version: 20

3. **Environment Variables**
   - Go to Site Settings → Build & Deploy → Environment
   - Add variables:
     - `NUXT_PUBLIC_API_BASE`: `https://app.cuttlefish.dev`
     - `NUXT_PUBLIC_SITE_URL`: `https://cuttlefish.dev`

4. **Deploy**
   - Click "Deploy site"
   - Netlify builds and deploys automatically

### Custom Domain

1. Go to Site Settings → Domain Management
2. Add custom domain: `cuttlefish.dev`
3. Update DNS records (Netlify provides instructions)
4. Enable auto-renewal for SSL certificate

### Redirects and Rewrites

Configured in `netlify.toml`:

```toml
[[redirects]]
from = "/github"
to = "https://github.com/JackTYM/cuttlefish-rs"
status = 302

[[redirects]]
from = "/*"
to = "/index.html"
status = 200
```

### Caching

Configured in `netlify.toml`:

```toml
[[headers]]
for = "/assets/*"
[headers.values]
Cache-Control = "public, max-age=31536000, immutable"
```

## Web Dashboard (`cuttlefish-web/`)

### Setup

1. **Create New Site**
   - Go to https://app.netlify.com
   - Click "New site from Git"
   - Select same GitHub repository
   - Netlify auto-detects `netlify.toml` (if present)

2. **Configure Build**
   - Build command: `npm run build`
   - Publish directory: `.output/server`
   - Node version: 20

3. **Environment Variables**
   - Go to Site Settings → Build & Deploy → Environment
   - Add variables:
     - `NUXT_PUBLIC_API_BASE`: `https://api.cuttlefish.dev`
     - `NUXT_PUBLIC_WS_URL`: `wss://api.cuttlefish.dev/ws`
     - `NUXT_PUBLIC_SITE_URL`: `https://app.cuttlefish.dev`
     - `NUXT_API_BASE`: `https://api.cuttlefish.dev`
     - `NUXT_WS_URL`: `wss://api.cuttlefish.dev/ws`

4. **Deploy**
   - Click "Deploy site"
   - Netlify builds and deploys

### Custom Domain

1. Go to Site Settings → Domain Management
2. Add custom domain: `app.cuttlefish.dev`
3. Update DNS records
4. Enable auto-renewal for SSL certificate

### WebSocket Configuration

For WebSocket support:
1. Ensure API server is accessible at configured URL
2. Use `wss://` (secure WebSocket) in production
3. Configure CORS on API server

## Advanced Features

### Netlify Functions

Deploy serverless functions alongside your site:

```javascript
// netlify/functions/api.js
export async function handler(event, context) {
  return {
    statusCode: 200,
    body: JSON.stringify({ message: 'Hello from Netlify Functions' })
  }
}
```

### Netlify Forms

Capture form submissions:

```html
<form name="contact" method="POST" netlify>
  <input type="email" name="email" required />
  <textarea name="message" required></textarea>
  <button type="submit">Send</button>
</form>
```

### Netlify Analytics

Monitor site performance:
1. Go to Site Settings → Analytics
2. Enable Netlify Analytics
3. View real-time traffic and performance metrics

## Monitoring

### Netlify Dashboard

- **Deploys**: View build logs and deployment history
- **Analytics**: Monitor traffic and performance
- **Functions**: Monitor serverless function execution
- **Forms**: View form submissions

### Build Logs

1. Go to Deploys tab
2. Click on a deployment
3. View build logs and errors

## Troubleshooting

### Build Fails

**Error: "Cannot find module"**
- Ensure all dependencies in `package.json`
- Run `npm install` locally to verify
- Check for missing environment variables

**Error: "Build timeout"**
- Increase build timeout in Site Settings
- Optimize build process
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

### Asset Optimization

- Enable Netlify's asset optimization
- Configure in Site Settings → Build & Deploy → Post processing
- Minify CSS, JavaScript, and HTML

### Image Optimization

- Use Netlify Image CDN
- Configure in `netlify.toml`:
```toml
[build]
command = "npm run generate"
publish = ".output/public"
```

### Bundle Analysis

```bash
npm run build -- --analyze
```

## Rollback

To rollback to a previous deployment:

1. Go to Deploys tab
2. Find the deployment to rollback to
3. Click "Publish deploy"

## Environment-Specific Builds

Deploy different versions to different branches:

1. Go to Site Settings → Build & Deploy → Deploy contexts
2. Configure branch-specific settings
3. Set environment variables per branch

## See Also

- [Netlify Documentation](https://docs.netlify.com)
- [Nuxt on Netlify](https://nuxt.com/deploy/netlify)
- [Environment Variables](https://docs.netlify.com/configure-builds/environment-variables/)
- [Netlify Functions](https://docs.netlify.com/functions/overview/)
