<template>
  <div class="flex min-h-screen bg-slate-950">
    <!-- Mobile Menu Toggle -->
    <button 
      @click="showMobileMenu = !showMobileMenu"
      class="lg:hidden fixed top-16 left-4 z-50 p-3 min-h-[44px] min-w-[44px] bg-slate-800 border border-slate-700 rounded-lg text-slate-300 hover:text-white transition-colors"
      aria-label="Toggle navigation menu"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
      </svg>
    </button>
    
    <!-- Sidebar -->
    <aside 
      class="w-64 bg-slate-900 p-4 border-r border-slate-800 sticky top-0 h-screen overflow-y-auto shrink-0 fixed lg:relative left-0 z-40 transition-transform duration-300"
      :class="showMobileMenu ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'"
    >
      <nav class="space-y-2">
        <NuxtLink 
          v-for="doc in navigation" 
          :key="doc.path"
          :to="doc.path"
          @click="showMobileMenu = false"
          class="block px-3 py-3 sm:py-2 min-h-[44px] sm:min-h-0 rounded text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
          :class="{ 'bg-slate-800 text-white': isActive(doc.path) }"
        >
          {{ doc.title }}
        </NuxtLink>
      </nav>
    </aside>
    
    <!-- Mobile Overlay -->
    <div 
      v-if="showMobileMenu" 
      class="lg:hidden fixed inset-0 bg-black/50 z-30"
      @click="showMobileMenu = false"
    />
    
    <!-- Content -->
    <main class="flex-1 p-4 sm:p-8 pt-20 lg:pt-8 max-w-4xl overflow-x-auto">
      <ContentDoc class="prose prose-invert max-w-none" />
    </main>
  </div>
</template>

<script setup>
const route = useRoute()
const showMobileMenu = ref(false)

// Static navigation list for all documentation pages
const navigation = [
  { title: 'Documentation', path: '/docs' },
  { title: 'Getting Started', path: '/docs/getting-started' },
  { title: 'Configuration', path: '/docs/configuration' },
  { title: 'Agents', path: '/docs/agents' },
  { title: 'Templates', path: '/docs/templates' },
  { title: 'API Reference', path: '/docs/api' }
]

const isActive = (path) => {
  return route.path === path
}
</script>

<style scoped>
:deep(.prose) {
  --tw-prose-body: rgb(226 232 240);
  --tw-prose-headings: rgb(248 250 252);
  --tw-prose-links: rgb(34 211 238);
  --tw-prose-code: rgb(226 232 240);
  --tw-prose-pre-bg: rgb(15 23 42);
  --tw-prose-pre-code: rgb(226 232 240);
}

:deep(.prose code) {
  background-color: rgb(30 41 59);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
}

:deep(.prose pre) {
  background-color: rgb(15 23 42);
  border: 1px solid rgb(51 65 85);
}

:deep(.prose a) {
  color: rgb(34 211 238);
  text-decoration: underline;
}

:deep(.prose a:hover) {
  color: rgb(6 182 212);
}
</style>
