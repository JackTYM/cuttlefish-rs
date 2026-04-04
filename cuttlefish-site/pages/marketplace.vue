<template>
  <div class="min-h-screen bg-slate-950">
    <!-- Hero Section -->
    <section class="pt-24 pb-12 px-4" aria-labelledby="marketplace-heading">
      <div class="max-w-7xl mx-auto text-center">
        <div class="text-5xl mb-4" aria-hidden="true">🛒</div>
        <h1 id="marketplace-heading" class="text-4xl md:text-5xl font-bold mb-4">
          <span class="bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
            Template Marketplace
          </span>
        </h1>
        <p class="text-xl text-slate-400 max-w-2xl mx-auto">
          Start your project from proven templates. Community-curated starters for every stack.
        </p>
      </div>
    </section>

    <!-- Search and Filters -->
    <section class="px-4 mb-8" aria-label="Search and filter templates">
      <div class="max-w-7xl mx-auto">
        <div class="flex flex-col lg:flex-row gap-4 items-stretch lg:items-center justify-between">
          <!-- Search -->
          <div class="relative flex-1 max-w-md">
            <label for="template-search" class="sr-only">Search templates</label>
            <svg 
              class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-slate-500 pointer-events-none" 
              fill="none" 
              stroke="currentColor" 
              viewBox="0 0 24 24"
              aria-hidden="true"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <input 
              id="template-search"
              v-model="searchQuery"
              type="text"
              placeholder="Search templates by name, language, or tag..."
              class="w-full pl-12 pr-4 py-3 bg-slate-900 border border-slate-800 rounded-lg text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500 transition motion-reduce:transition-none"
            />
          </div>

          <!-- Category Tabs -->
          <div class="flex flex-wrap gap-2" role="tablist" aria-label="Filter by category">
            <button
              v-for="cat in categories"
              :key="cat.id"
              @click="selectedCategory = cat.id"
              :class="[
                'px-4 py-2.5 sm:py-2 min-h-[44px] sm:min-h-0 rounded-lg text-sm font-medium transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
                selectedCategory === cat.id
                  ? 'bg-cyan-600 text-white'
                  : 'bg-slate-800 text-slate-400 hover:bg-slate-700 hover:text-slate-300'
              ]"
              role="tab"
              :aria-selected="selectedCategory === cat.id"
              :aria-controls="'panel-' + cat.id"
            >
              <span aria-hidden="true" class="mr-1.5">{{ cat.icon }}</span>
              {{ cat.label }}
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- Featured Templates -->
    <section 
      v-if="selectedCategory === 'all' && !searchQuery"
      class="px-4 mb-12" 
      aria-labelledby="featured-heading"
    >
      <div class="max-w-7xl mx-auto">
        <div class="flex items-center gap-3 mb-6">
          <span class="text-2xl" aria-hidden="true">⭐</span>
          <h2 id="featured-heading" class="text-2xl font-bold text-white">Featured Templates</h2>
        </div>
        <div class="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          <TemplateCard
            v-for="t in featuredTemplates"
            :key="t.id"
            :template="t"
            featured
          />
        </div>
      </div>
    </section>

    <!-- Popular Templates -->
    <section 
      v-if="selectedCategory === 'all' && !searchQuery"
      class="px-4 mb-12" 
      aria-labelledby="popular-heading"
    >
      <div class="max-w-7xl mx-auto">
        <div class="flex items-center gap-3 mb-6">
          <span class="text-2xl" aria-hidden="true">🔥</span>
          <h2 id="popular-heading" class="text-2xl font-bold text-white">Popular This Week</h2>
        </div>
        <div class="grid md:grid-cols-2 lg:grid-cols-4 gap-4">
          <TemplateCardMini
            v-for="t in popularTemplates"
            :key="t.id"
            :template="t"
          />
        </div>
      </div>
    </section>

    <!-- All Templates Grid -->
    <section class="px-4 pb-16" aria-labelledby="all-templates-heading">
      <div class="max-w-7xl mx-auto">
        <div class="flex items-center justify-between mb-6">
          <h2 id="all-templates-heading" class="text-2xl font-bold text-white">
            {{ searchQuery ? 'Search Results' : selectedCategory === 'all' ? 'All Templates' : categories.find(c => c.id === selectedCategory)?.label }}
          </h2>
          <span class="text-slate-500 text-sm">
            {{ filteredTemplates.length }} template{{ filteredTemplates.length !== 1 ? 's' : '' }}
          </span>
        </div>

        <div 
          v-if="filteredTemplates.length > 0"
          class="grid md:grid-cols-2 lg:grid-cols-3 gap-6" 
          role="list"
          aria-label="Available templates"
        >
          <TemplateCard
            v-for="t in filteredTemplates"
            :key="t.id"
            :template="t"
          />
        </div>

        <!-- Empty State -->
        <div 
          v-else 
          class="text-center py-16 bg-slate-900/50 rounded-xl border border-slate-800"
          role="status"
        >
          <div class="text-5xl mb-4" aria-hidden="true">🔍</div>
          <h3 class="text-xl font-semibold text-white mb-2">No templates found</h3>
          <p class="text-slate-400 mb-6">
            Try adjusting your search or filter criteria
          </p>
          <button
            @click="searchQuery = ''; selectedCategory = 'all'"
            class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
          >
            Clear filters
          </button>
        </div>
      </div>
    </section>

    <!-- Submit CTA -->
    <section class="px-4 pb-24" aria-labelledby="submit-cta-heading">
      <div class="max-w-4xl mx-auto">
        <div class="relative overflow-hidden rounded-xl border border-slate-800 bg-gradient-to-br from-slate-900 via-slate-900 to-cyan-950/30">
          <!-- Decorative elements -->
          <div class="absolute top-0 right-0 w-64 h-64 bg-cyan-500/5 blur-3xl rounded-full -translate-y-1/2 translate-x-1/2" aria-hidden="true" />
          <div class="absolute bottom-0 left-0 w-48 h-48 bg-purple-500/5 blur-3xl rounded-full translate-y-1/2 -translate-x-1/2" aria-hidden="true" />
          
          <div class="relative p-8 md:p-12 text-center">
            <div class="text-4xl mb-4" aria-hidden="true">🚀</div>
            <h2 id="submit-cta-heading" class="text-2xl md:text-3xl font-bold text-white mb-3">
              Have a great template?
            </h2>
            <p class="text-slate-400 mb-6 max-w-lg mx-auto">
              Share your project starters with the community. Templates are markdown-based and easy to create.
            </p>
            
            <!-- Requirements -->
            <div class="flex flex-wrap justify-center gap-3 mb-8">
              <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-800/50 rounded-lg text-sm text-slate-300">
                <svg class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                README.md
              </span>
              <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-800/50 rounded-lg text-sm text-slate-300">
                <svg class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                template.yaml
              </span>
              <span class="inline-flex items-center gap-1.5 px-3 py-1.5 bg-slate-800/50 rounded-lg text-sm text-slate-300">
                <svg class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
                MIT License
              </span>
            </div>

            <div class="flex flex-col sm:flex-row gap-4 justify-center">
              <a 
                href="https://github.com/JackTYM/cuttlefish-rs/blob/main/docs/templates.md" 
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center justify-center gap-2 px-6 py-3 bg-cyan-600 hover:bg-cyan-500 text-white font-semibold rounded-lg transition-all duration-200 hover:shadow-lg hover:shadow-cyan-500/25 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950"
              >
                <svg class="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
                </svg>
                Submit Your Template
                <span class="sr-only">(opens in new tab)</span>
              </a>
              <a 
                href="https://github.com/JackTYM/cuttlefish-rs/tree/main/templates" 
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center justify-center gap-2 px-6 py-3 bg-slate-800 hover:bg-slate-700 text-slate-300 font-semibold rounded-lg transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950"
              >
                <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />
                </svg>
                View Example Templates
                <span class="sr-only">(opens in new tab)</span>
              </a>
            </div>
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'default'
})

// Search and filter state
const searchQuery = ref('')
const selectedCategory = ref('all')

// Categories with icons
const categories = [
  { id: 'all', label: 'All', icon: '📦' },
  { id: 'official', label: 'Official', icon: '✓' },
  { id: 'community', label: 'Community', icon: '👥' },
  { id: 'rust', label: 'Rust', icon: '🦀' },
  { id: 'typescript', label: 'TypeScript', icon: '🔷' },
  { id: 'python', label: 'Python', icon: '🐍' },
  { id: 'go', label: 'Go', icon: '🐹' },
]

// Mock template data
const templates = ref([
  {
    id: '1',
    name: 'rust-cli',
    description: 'Production-ready Rust CLI application with argument parsing, logging, and error handling',
    category: 'official',
    language: 'Rust',
    author: 'cuttlefish',
    authorUrl: 'https://github.com/JackTYM',
    stars: 142,
    downloads: 1247,
    tags: ['cli', 'production', 'clap'],
    icon: '🦀',
    featured: true,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/rust-cli'
  },
  {
    id: '2',
    name: 'rust-lib',
    description: 'Rust library skeleton with tests, CI/CD, documentation, and publishing setup',
    category: 'official',
    language: 'Rust',
    author: 'cuttlefish',
    authorUrl: 'https://github.com/JackTYM',
    stars: 98,
    downloads: 856,
    tags: ['library', 'testing', 'ci'],
    icon: '📚',
    featured: true,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/rust-lib'
  },
  {
    id: '3',
    name: 'nuxt-app',
    description: 'Nuxt 3 web application with Tailwind CSS, authentication, and deployment config',
    category: 'official',
    language: 'TypeScript',
    author: 'cuttlefish',
    authorUrl: 'https://github.com/JackTYM',
    stars: 156,
    downloads: 1834,
    tags: ['web', 'nuxt', 'tailwind'],
    icon: '💚',
    featured: true,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/nuxt-app'
  },
  {
    id: '4',
    name: 'fastapi-backend',
    description: 'Python FastAPI backend with async database, authentication, and OpenAPI docs',
    category: 'official',
    language: 'Python',
    author: 'cuttlefish',
    authorUrl: 'https://github.com/JackTYM',
    stars: 87,
    downloads: 723,
    tags: ['api', 'async', 'openapi'],
    icon: '⚡',
    featured: true,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/fastapi-backend'
  },
  {
    id: '5',
    name: 'discord-bot',
    description: 'Discord bot with slash commands, event handlers, and database integration',
    category: 'community',
    language: 'TypeScript',
    author: 'devcontrib',
    authorUrl: 'https://github.com/devcontrib',
    stars: 64,
    downloads: 412,
    tags: ['discord', 'bot', 'typescript'],
    icon: '🤖',
    featured: false,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/discord-bot'
  },
  {
    id: '6',
    name: 'tauri-app',
    description: 'Tauri desktop application with React frontend and Rust backend',
    category: 'community',
    language: 'Rust',
    author: 'contributor',
    authorUrl: 'https://github.com/contributor',
    stars: 45,
    downloads: 289,
    tags: ['desktop', 'tauri', 'react'],
    icon: '🖥️',
    featured: false,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/tauri-app'
  },
  {
    id: '7',
    name: 'go-microservice',
    description: 'Go microservice with gRPC, health checks, and Kubernetes deployment',
    category: 'community',
    language: 'Go',
    author: 'gopher',
    authorUrl: 'https://github.com/gopher',
    stars: 38,
    downloads: 198,
    tags: ['microservice', 'grpc', 'k8s'],
    icon: '🐹',
    featured: false,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/go-microservice'
  },
  {
    id: '8',
    name: 'axum-api',
    description: 'Rust Axum REST API with JWT auth, database, and OpenAPI spec generation',
    category: 'official',
    language: 'Rust',
    author: 'cuttlefish',
    authorUrl: 'https://github.com/JackTYM',
    stars: 112,
    downloads: 967,
    tags: ['api', 'rest', 'jwt'],
    icon: '🦀',
    featured: false,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/axum-api'
  },
  {
    id: '9',
    name: 'next-pages',
    description: 'Next.js Pages Router with Prisma, NextAuth, and Vercel deployment',
    category: 'community',
    language: 'TypeScript',
    author: 'webdev',
    authorUrl: 'https://github.com/webdev',
    stars: 73,
    downloads: 534,
    tags: ['nextjs', 'prisma', 'auth'],
    icon: '▲',
    featured: false,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/next-pages'
  },
  {
    id: '10',
    name: 'django-api',
    description: 'Django REST Framework API with JWT auth, CORS, and Docker deployment',
    category: 'community',
    language: 'Python',
    author: 'pydev',
    authorUrl: 'https://github.com/pydev',
    stars: 52,
    downloads: 378,
    tags: ['django', 'rest', 'docker'],
    icon: '🎸',
    featured: false,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/django-api'
  },
  {
    id: '11',
    name: 'bevy-game',
    description: 'Bevy game engine starter with 2D rendering, input handling, and asset loading',
    category: 'community',
    language: 'Rust',
    author: 'gamedev',
    authorUrl: 'https://github.com/gamedev',
    stars: 29,
    downloads: 156,
    tags: ['game', 'bevy', '2d'],
    icon: '🎮',
    featured: false,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/bevy-game'
  },
  {
    id: '12',
    name: 'flask-api',
    description: 'Flask REST API with SQLAlchemy, migrations, and pytest setup',
    category: 'community',
    language: 'Python',
    author: 'flaskfan',
    authorUrl: 'https://github.com/flaskfan',
    stars: 41,
    downloads: 287,
    tags: ['flask', 'sqlalchemy', 'pytest'],
    icon: '🌶️',
    featured: false,
    previewUrl: 'https://github.com/JackTYM/cuttlefish-rs/tree/main/templates/flask-api'
  },
])

// Computed: Featured templates
const featuredTemplates = computed(() => 
  templates.value.filter(t => t.featured)
)

// Computed: Popular templates (sorted by downloads)
const popularTemplates = computed(() => 
  [...templates.value]
    .sort((a, b) => b.downloads - a.downloads)
    .slice(0, 4)
)

// Computed: Filtered templates
const filteredTemplates = computed(() => {
  return templates.value.filter(t => {
    // Search filter
    const searchLower = searchQuery.value.toLowerCase()
    const matchesSearch = !searchQuery.value || 
      t.name.toLowerCase().includes(searchLower) ||
      t.description.toLowerCase().includes(searchLower) ||
      t.language.toLowerCase().includes(searchLower) ||
      t.tags.some(tag => tag.toLowerCase().includes(searchLower))
    
    // Category filter
    const matchesCategory = selectedCategory.value === 'all' || 
      t.category === selectedCategory.value ||
      t.language.toLowerCase() === selectedCategory.value
    
    return matchesSearch && matchesCategory
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