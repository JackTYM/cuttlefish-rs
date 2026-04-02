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
      ]
    }
  }
})
