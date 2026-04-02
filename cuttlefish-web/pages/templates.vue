<template>
  <div class="p-6">
    <!-- Header -->
    <header class="mb-8">
      <h1 class="text-2xl font-bold text-white mb-2">Templates</h1>
      <p class="text-gray-400">Start your project from proven templates</p>
    </header>
    
    <!-- Filters -->
    <div class="flex flex-col sm:flex-row gap-4 mb-6" role="search">
      <!-- Search -->
      <label for="template-search" class="sr-only">Search templates</label>
      <input 
        id="template-search"
        v-model="searchQuery"
        type="search" 
        placeholder="Search templates..."
        class="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm text-white placeholder-gray-500 focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none min-h-[44px]"
      />
      
      <!-- Language filter -->
      <fieldset>
        <legend class="sr-only">Filter by language</legend>
        <div class="flex gap-2 flex-wrap" role="group" aria-label="Filter by programming language">
          <button 
            v-for="lang in languages" 
            :key="lang"
            @click="selectedLanguage = lang === 'All' ? '' : lang"
            :class="[
              'px-4 py-2 rounded-lg text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-950 min-h-[44px]',
              (selectedLanguage === lang || (!selectedLanguage && lang === 'All'))
                ? 'bg-cyan-600 text-white'
                : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white'
            ]"
            :aria-pressed="(selectedLanguage === lang || (!selectedLanguage && lang === 'All'))"
          >
            {{ lang }}
          </button>
        </div>
      </fieldset>
    </div>
    
    <!-- Template Grid -->
    <section v-if="filteredTemplates.length" aria-labelledby="templates-heading">
      <h2 id="templates-heading" class="sr-only">Available Templates</h2>
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <article 
          v-for="template in filteredTemplates" 
          :key="template.id"
          class="bg-gray-900 rounded-xl border border-gray-800 p-5 hover:border-cyan-800 transition-colors motion-reduce:transition-none group"
        >
          <div class="flex justify-between items-start mb-3">
            <h3 class="font-semibold text-white">{{ template.name }}</h3>
            <span class="text-xs px-2 py-0.5 rounded-full bg-gray-800 text-gray-400">{{ template.language }}</span>
          </div>
          <p class="text-sm text-gray-400 line-clamp-2 mb-4">{{ template.description }}</p>
          <div class="flex justify-between items-center">
            <span class="text-xs text-gray-500">by {{ template.author }}</span>
            <button 
              class="px-3 py-1.5 bg-cyan-600 hover:bg-cyan-500 text-white text-sm rounded-lg transition-colors motion-reduce:transition-none opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
              :aria-label="`Use ${template.name} template`"
            >
              Use Template
            </button>
          </div>
        </article>
      </div>
    </section>
    
    <!-- Empty state -->
    <div v-else class="text-center py-16 text-gray-500" role="status">
      <div class="text-4xl mb-4" role="img" aria-label="Empty box">📦</div>
      <p>No templates found matching your criteria</p>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Template {
  id: string
  name: string
  description: string
  language: string
  author: string
  stars: number
}

// State
const searchQuery = ref('')
const selectedLanguage = ref('')

// Mock data for now (will connect to API later)
const mockTemplates: Template[] = [
  { id: '1', name: 'rust-cli', description: 'Rust CLI application starter with argument parsing and logging', language: 'Rust', author: 'cuttlefish', stars: 42 },
  { id: '2', name: 'rust-lib', description: 'Rust library with tests, documentation, and CI/CD', language: 'Rust', author: 'cuttlefish', stars: 38 },
  { id: '3', name: 'nuxt-app', description: 'Nuxt 3 web application with Tailwind CSS', language: 'TypeScript', author: 'cuttlefish', stars: 56 },
  { id: '4', name: 'fastapi', description: 'Python FastAPI backend with async support', language: 'Python', author: 'cuttlefish', stars: 34 },
  { id: '5', name: 'discord-bot', description: 'Discord bot starter with slash commands', language: 'TypeScript', author: 'community', stars: 28 },
  { id: '6', name: 'go-microservice', description: 'Go microservice with gRPC and Docker', language: 'Go', author: 'community', stars: 31 },
]

const languages = ['All', 'Rust', 'TypeScript', 'Python', 'Go']

// Computed filtered templates
const filteredTemplates = computed(() => {
  return mockTemplates.filter(t => {
    const matchesSearch = t.name.toLowerCase().includes(searchQuery.value.toLowerCase()) || 
                          t.description.toLowerCase().includes(searchQuery.value.toLowerCase())
    const matchesLang = !selectedLanguage.value || selectedLanguage.value === 'All' || 
                        t.language === selectedLanguage.value
    return matchesSearch && matchesLang
  })
})
</script>
