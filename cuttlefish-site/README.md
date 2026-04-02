# Cuttlefish Marketing Site

Marketing website for Cuttlefish, built with Nuxt 3 and Tailwind CSS.

## Features

- **Nuxt 3** with SSG (Static Site Generation)
- **Tailwind CSS** with dark theme defaults
- **Responsive Design** with sticky navigation
- **Marketing Layout** with hero section and CTA

## Development

```bash
# Install dependencies
npm install

# Start dev server
npm run dev

# Build for production
npm run build

# Generate static site
npm run generate

# Preview production build
npm run preview
```

## Structure

```
cuttlefish-site/
├── layouts/
│   └── default.vue          # Main marketing layout with sticky header
├── pages/
│   └── index.vue            # Hero page (Coming Soon)
├── nuxt.config.ts           # Nuxt configuration (SSG mode)
├── tailwind.config.ts       # Tailwind CSS configuration
└── app.vue                  # Root component
```

## Configuration

### SSG Mode

The site is configured for static site generation:

```typescript
export default defineNuxtConfig({
  ssr: true,
  nitro: {
    preset: 'static'
  }
})
```

This generates static HTML files in `.output/public/` that can be deployed to any static hosting.

### Tailwind Theme

Dark theme with slate-950 background and cyan/purple accents:

```typescript
darkMode: 'class',
theme: {
  extend: {
    colors: {
      'dark-bg': '#0f172a',
      'dark-card': '#1e293b',
    }
  }
}
```

## Deployment

Generate static files:

```bash
npm run generate
```

Deploy the `.output/public/` directory to any static hosting (Vercel, Netlify, GitHub Pages, etc.).

## Navigation

The default layout includes:

- **Logo**: 🐙 Cuttlefish
- **Nav Items**: Features, Install, Docs, Marketplace
- **CTA Button**: "Launch App →" linking to app.cuttlefish.dev
- **Sticky Header**: Dark background with backdrop blur

## License

MIT
