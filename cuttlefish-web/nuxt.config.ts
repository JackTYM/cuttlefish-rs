// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  devtools: { enabled: true },
  modules: ['@nuxtjs/tailwindcss'],
  ssr: false, // SPA mode
  runtimeConfig: {
    public: {
      apiBase: 'http://localhost:8080',
      wsUrl: 'ws://localhost:8080/ws',
    },
  },
})