<template>
  <div class="min-h-screen bg-slate-950 py-16">
    <div class="container mx-auto px-4">
      <!-- Header -->
      <header class="text-center mb-12">
        <h1 id="marketplace-heading" class="text-4xl font-bold text-white mb-4">Template Marketplace</h1>
        <p class="text-slate-400 text-lg">Start your project from proven templates</p>
      </header>
      
      <!-- Filters -->
      <div class="flex flex-col sm:flex-row gap-4 mb-8 max-w-2xl mx-auto" role="search" aria-label="Filter templates">
        <label for="template-search" class="sr-only">Search templates</label>
        <input 
          id="template-search"
          v-model="searchQuery"
          type="text"
          placeholder="Search templates..."
          class="flex-1 px-4 py-3 bg-slate-900 border border-slate-700 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500 transition motion-reduce:transition-none focus-visible:ring-2 focus-visible:ring-cyan-400"
        />
        <div class="flex gap-2" role="group" aria-label="Filter by category">
          <button
            v-for="cat in categories"
            :key="cat"
            @click="selectedCategory = cat === 'All' ? '' : cat"
            :class="[
              'px-4 py-2 rounded-lg text-sm font-medium transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
              (selectedCategory === cat || (!selectedCategory && cat === 'All'))
                ? 'bg-cyan-600 text-white'
                : 'bg-slate-800 text-slate-400 hover:bg-slate-700'
            ]"
            :aria-pressed="selectedCategory === cat || (!selectedCategory && cat === 'All')"
          >
            {{ cat }}
          </button>
        </div>
      </div>
      
      <!-- Grid -->
      <main aria-labelledby="marketplace-heading">
        <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6" role="list" aria-label="Available templates">
          <article 
            v-for="t in filteredTemplates" 
            :key="t.id"
            class="bg-slate-900 border border-slate-800 rounded-xl p-6 hover:border-slate-700 transition motion-reduce:transition-none group"
            role="listitem"
          >
            <div class="flex justify-between items-start mb-3">
              <h2 class="text-lg font-semibold text-white">{{ t.name }}</h2>
              <span class="text-xs px-2 py-1 bg-slate-800 text-slate-400 rounded">{{ t.language }}</span>
            </div>
            <p class="text-slate-400 text-sm mb-4">{{ t.description }}</p>
            <div class="flex justify-between items-center">
              <div class="flex items-center gap-2 text-sm text-slate-500">
                <span>by {{ t.author }}</span>
                <span aria-hidden="true">•</span>
                <span aria-label="{{ t.stars }} stars">⭐ {{ t.stars }}</span>
              </div>
              <a 
                :href="'https://app.cuttlefish.dev/new?template=' + t.name"
                class="px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white text-sm rounded-lg transition motion-reduce:transition-none opacity-0 group-hover:opacity-100 focus:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                :aria-label="`Use ${t.name} template`"
              >
                Use Template
              </a>
            </div>
            <div class="mt-3">
              <span :class="[
                'text-xs px-2 py-1 rounded',
                t.category === 'Official' ? 'bg-cyan-900/50 text-cyan-400' : 'bg-purple-900/50 text-purple-400'
              ]">
                {{ t.category }}
              </span>
            </div>
          </article>
        </div>
        
        <!-- Empty state -->
        <div v-if="filteredTemplates.length === 0" class="text-center py-12 text-slate-400" role="status">
          <p>No templates found matching your search.</p>
        </div>
      </main>
      
      <!-- Submit CTA -->
      <section class="text-center mt-16 p-8 bg-slate-900 border border-slate-800 rounded-xl" aria-labelledby="submit-heading">
        <h2 id="submit-heading" class="text-xl font-semibold text-white mb-2">Have a great template?</h2>
        <p class="text-slate-400 mb-4">Share it with the community</p>
        <a 
          href="https://github.com/JackTYM/cuttlefish-rs/blob/main/docs/templates.md" 
          target="_blank"
          rel="noopener noreferrer"
          class="inline-block px-6 py-3 bg-slate-800 hover:bg-slate-700 text-white rounded-lg transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
        >
          Submit Your Template →
          <span class="sr-only">(opens in new tab)</span>
        </a>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'default'
})

const searchQuery = ref('')
const selectedCategory = ref('')

const templates = ref([
  { id: '1', name: 'rust-cli', description: 'Rust CLI application starter', category: 'Official', language: 'Rust', author: 'cuttlefish', stars: 142 },
  { id: '2', name: 'rust-lib', description: 'Rust library with tests and CI', category: 'Official', language: 'Rust', author: 'cuttlefish', stars: 98 },
  { id: '3', name: 'nuxt-app', description: 'Nuxt 3 web application', category: 'Official', language: 'TypeScript', author: 'cuttlefish', stars: 156 },
  { id: '4', name: 'fastapi', description: 'Python FastAPI backend', category: 'Official', language: 'Python', author: 'cuttlefish', stars: 87 },
  { id: '5', name: 'discord-bot', description: 'Discord bot starter', category: 'Community', language: 'TypeScript', author: 'community', stars: 64 },
  { id: '6', name: 'tauri-app', description: 'Tauri desktop application', category: 'Community', language: 'Rust', author: 'contributor', stars: 45 },
])

const categories = ['All', 'Official', 'Community']

const filteredTemplates = computed(() => {
  return templates.value.filter(t => {
    const matchesSearch = !searchQuery.value || 
      t.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      t.description.toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchesCat = !selectedCategory.value || selectedCategory.value === 'All' || t.category === selectedCategory.value
    return matchesSearch && matchesCat
  })
})
</script>

<style scoped>
/* Screen reader only utility */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}

/* Prefers reduced motion support */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
</style>