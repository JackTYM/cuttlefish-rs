---
name: nuxt-cloudflare
description: Nuxt 3 + Cloudflare Pages fullstack TypeScript application
language: typescript
docker_image: node:22-slim
variables:
  - name: project_name
    description: Name of the project
    required: true
  - name: description
    description: Project description
    default: "A Nuxt 3 application"
tags: [frontend, fullstack, typescript, vue, cloudflare]
---

# {{ project_name }}

{{ description }}

## Project Structure

```
{{ project_name }}/
├── nuxt.config.ts
├── app.vue
├── package.json
├── tsconfig.json
├── wrangler.toml
├── pages/
│   ├── index.vue
│   └── about.vue
├── server/
│   ├── api/
│   │   ├── hello.ts
│   │   └── users.ts
│   └── middleware/
│       └── auth.ts
├── components/
│   ├── Header.vue
│   └── Footer.vue
├── composables/
│   └── useApi.ts
└── public/
    └── favicon.ico
```

## Files

### nuxt.config.ts
```typescript
export default defineNuxtConfig({
  ssr: true,
  nitro: {
    prerender: {
      crawlLinks: true,
      routes: ['/sitemap.xml', '/rss.xml']
    }
  },
  modules: ['@nuxtjs/tailwindcss'],
  typescript: {
    strict: true
  }
})
```

### app.vue
```vue
<template>
  <div>
    <Header />
    <main class="container mx-auto px-4 py-8">
      <NuxtPage />
    </main>
    <Footer />
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'default'
})
</script>
```

### pages/index.vue
```vue
<template>
  <div>
    <h1 class="text-4xl font-bold mb-4">Welcome to {{ project_name }}</h1>
    <p class="text-lg text-gray-600 mb-8">{{ description }}</p>
    <button @click="fetchData" class="bg-blue-500 text-white px-4 py-2 rounded">
      Load Data
    </button>
    <div v-if="data" class="mt-4 p-4 bg-gray-100 rounded">
      {{ data }}
    </div>
  </div>
</template>

<script setup lang="ts">
const data = ref(null)

const fetchData = async () => {
  const response = await $fetch('/api/hello')
  data.value = response
}
</script>
```

### server/api/hello.ts
```typescript
export default defineEventHandler(async (event) => {
  return {
    message: 'Hello from {{ project_name }}',
    timestamp: new Date().toISOString()
  }
})
```

### wrangler.toml
```toml
name = "{{ project_name }}"
type = "javascript"
account_id = "{{ CLOUDFLARE_ACCOUNT_ID }}"
workers_dev = true
route = ""
zone_id = ""

[env.production]
name = "{{ project_name }}-prod"
route = "example.com/*"
zone_id = "{{ CLOUDFLARE_ZONE_ID }}"
```

### package.json
```json
{
  "name": "{{ project_name }}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "scripts": {
    "dev": "nuxi dev",
    "build": "nuxi build",
    "preview": "nuxi preview",
    "deploy": "wrangler deploy"
  },
  "dependencies": {
    "nuxt": "^3.9.0",
    "vue": "^3.3.0"
  },
  "devDependencies": {
    "@nuxtjs/tailwindcss": "^6.10.0",
    "typescript": "^5.3.0"
  }
}
```

### tsconfig.json
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "moduleResolution": "bundler"
  }
}
```

## Getting Started

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start development server:
   ```bash
   npm run dev
   ```

3. Build for production:
   ```bash
   npm run build
   ```

4. Deploy to Cloudflare Pages:
   ```bash
   npm run deploy
   ```

## Environment Variables

Create a `.env` file:
```
CLOUDFLARE_ACCOUNT_ID=your_account_id
CLOUDFLARE_ZONE_ID=your_zone_id
```
