// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  ssr: true,
  nitro: {
    preset: 'static'
  },
  modules: ['@nuxtjs/tailwindcss', '@nuxt/content'],
  tailwindcss: {
    configPath: 'tailwind.config.ts'
  },
  content: {
    highlight: {
      theme: 'github-dark'
    }
  },
  app: {
    head: {
      title: 'Cuttlefish - Multi-Agent AI Coding Platform',
      meta: [
        { name: 'description', content: 'A portable, multi-agent, multi-model agentic coding platform built in Rust' }
      ],
      link: [
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        { rel: 'stylesheet', href: 'https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&display=swap' },
      ],
    }
  }
})
