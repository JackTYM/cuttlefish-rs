<template>
  <div class="p-4 sm:p-6">
    <div class="max-w-6xl mx-auto">
      <!-- Page Header -->
      <header class="mb-6">
        <h1 class="text-2xl font-bold text-white mb-2">Templates</h1>
        <p class="text-gray-400 text-sm">Start your project with a pre-configured template</p>
      </header>

      <!-- Filters Bar -->
      <section class="bg-gray-900 rounded-xl border border-gray-800 p-4 mb-6" aria-labelledby="filters-heading">
        <h2 id="filters-heading" class="sr-only">Filter templates</h2>
        <div class="flex flex-col sm:flex-row gap-3">
          <!-- Search -->
          <div class="flex-1 relative">
            <label for="template-search" class="sr-only">Search templates</label>
            <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
            </svg>
            <input
              id="template-search"
              v-model="searchQuery"
              type="search"
              placeholder="Search templates..."
              class="w-full bg-gray-800 border border-gray-700 rounded-lg pl-10 pr-4 py-2.5 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none"
            />
          </div>

          <!-- Category Filter -->
          <div class="relative">
            <label for="category-filter" class="sr-only">Filter by category</label>
            <select
              id="category-filter"
              v-model="selectedCategory"
              class="w-full sm:w-48 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none appearance-none cursor-pointer"
            >
              <option value="">All Categories</option>
              <option v-for="cat in categories" :key="cat" :value="cat">{{ cat }}</option>
            </select>
            <svg class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </div>

          <!-- Sort -->
          <div class="relative">
            <label for="sort-filter" class="sr-only">Sort templates</label>
            <select
              id="sort-filter"
              v-model="sortBy"
              class="w-full sm:w-44 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2.5 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none appearance-none cursor-pointer"
            >
              <option value="popularity">Most Popular</option>
              <option value="name">Name (A-Z)</option>
              <option value="updated">Recently Updated</option>
            </select>
            <svg class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </div>
        </div>

        <!-- Category Pills (Quick Filters) -->
        <div class="flex flex-wrap gap-2 mt-4" role="group" aria-label="Quick category filters">
          <button
            v-for="cat in ['All', ...categories.slice(0, 5)]"
            :key="cat"
            @click="selectedCategory = cat === 'All' ? '' : cat"
            class="px-3 py-2.5 sm:py-1.5 min-h-[44px] sm:min-h-0 rounded-full text-xs font-medium transition-all duration-200 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
            :class="(cat === 'All' ? !selectedCategory : selectedCategory === cat)
              ? 'bg-cyan-600 text-white'
              : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-gray-200'"
            :aria-pressed="cat === 'All' ? !selectedCategory : selectedCategory === cat"
          >
            {{ cat }}
          </button>
        </div>
      </section>

      <!-- Loading State -->
      <div v-if="loading" class="flex items-center justify-center py-16" role="status" aria-live="polite">
        <div class="flex flex-col items-center gap-3">
          <svg class="w-8 h-8 text-cyan-400 animate-spin" fill="none" viewBox="0 0 24 24" aria-hidden="true">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
          </svg>
          <span class="text-gray-400 text-sm">Loading templates...</span>
        </div>
      </div>

      <!-- Error State -->
      <div v-else-if="error" class="bg-red-900/20 border border-red-800/50 rounded-xl p-6 text-center" role="alert">
        <svg class="w-12 h-12 mx-auto mb-3 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <h3 class="text-lg font-semibold text-red-300 mb-1">Failed to load templates</h3>
        <p class="text-red-400/80 text-sm mb-4">{{ error }}</p>
        <button
          @click="fetchTemplates"
          class="px-4 py-2 bg-red-900/50 hover:bg-red-800/50 text-red-300 rounded-lg text-sm transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400"
        >
          Try Again
        </button>
      </div>

      <!-- Empty State -->
      <div v-else-if="filteredTemplates.length === 0" class="text-center py-16" role="status">
        <svg class="w-16 h-16 mx-auto mb-4 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
        </svg>
        <h3 class="text-lg font-medium text-gray-300 mb-1">No templates found</h3>
        <p class="text-gray-500 text-sm">Try adjusting your search or filters</p>
      </div>

      <!-- Template Grid -->
      <section v-else aria-labelledby="templates-heading">
        <h2 id="templates-heading" class="sr-only">Available templates</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <article
            v-for="template in filteredTemplates"
            :key="template.id"
            class="bg-gray-900 rounded-xl border border-gray-800 p-5 hover:border-cyan-800 transition-all duration-200 motion-reduce:transition-none cursor-pointer group focus-within:ring-2 focus-within:ring-cyan-400 focus-within:ring-offset-2 focus-within:ring-offset-gray-950"
            tabindex="0"
            role="button"
            :aria-label="`View details for ${template.name} template`"
            @click="openTemplateDetail(template)"
            @keydown.enter="openTemplateDetail(template)"
            @keydown.space.prevent="openTemplateDetail(template)"
          >
            <!-- Header -->
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-3">
                <!-- Icon -->
                <div
                  class="w-10 h-10 rounded-lg flex items-center justify-center text-lg"
                  :class="getCategoryColor(template.category)"
                  aria-hidden="true"
                >
                  {{ getCategoryIcon(template.category) }}
                </div>
                <div>
                  <h3 class="font-semibold text-white group-hover:text-cyan-400 transition-colors motion-reduce:transition-none">
                    {{ template.name }}
                  </h3>
                  <span class="text-xs text-gray-500">{{ template.language }}</span>
                </div>
              </div>
              <!-- Use Count -->
              <div class="flex items-center gap-1 text-gray-500" :aria-label="`Used ${template.useCount} times`">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
                </svg>
                <span class="text-xs">{{ formatCount(template.useCount) }}</span>
              </div>
            </div>

            <!-- Description -->
            <p class="text-sm text-gray-400 line-clamp-2 mb-4">{{ template.description }}</p>

            <!-- Tags -->
            <div class="flex flex-wrap gap-1.5 mb-4" aria-label="Template tags">
              <span
                v-for="tag in template.tags.slice(0, 3)"
                :key="tag"
                class="px-2 py-0.5 rounded text-xs bg-gray-800 text-gray-400"
              >
                {{ tag }}
              </span>
              <span v-if="template.tags.length > 3" class="px-2 py-0.5 rounded text-xs bg-gray-800 text-gray-500">
                +{{ template.tags.length - 3 }}
              </span>
            </div>

            <!-- Footer -->
            <div class="flex items-center justify-between pt-3 border-t border-gray-800">
              <span class="text-xs text-gray-500">
                Updated {{ formatDate(template.updatedAt) }}
              </span>
              <span
                class="text-xs px-2 py-1 rounded-full"
                :class="getCategoryBadgeColor(template.category)"
              >
                {{ template.category }}
              </span>
            </div>
          </article>
        </div>
      </section>
    </div>

    <!-- Template Detail Modal -->
    <Teleport to="body">
      <div
        v-if="selectedTemplateDetail"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        role="dialog"
        aria-modal="true"
        aria-labelledby="modal-title"
      >
        <!-- Backdrop -->
        <div
          class="absolute inset-0 bg-black/70 backdrop-blur-sm"
          @click="closeTemplateDetail"
          aria-hidden="true"
        />

        <!-- Modal -->
        <div class="relative bg-gray-900 rounded-xl border border-gray-700 shadow-2xl max-w-2xl w-full max-h-[90vh] overflow-hidden flex flex-col">
          <!-- Header -->
          <header class="px-6 py-4 border-b border-gray-800 flex items-start justify-between shrink-0">
            <div class="flex items-center gap-4">
              <div
                class="w-12 h-12 rounded-lg flex items-center justify-center text-xl"
                :class="getCategoryColor(selectedTemplateDetail.category)"
                aria-hidden="true"
              >
                {{ getCategoryIcon(selectedTemplateDetail.category) }}
              </div>
              <div>
                <h2 id="modal-title" class="text-xl font-bold text-white">{{ selectedTemplateDetail.name }}</h2>
                <div class="flex items-center gap-2 mt-1">
                  <span class="text-sm text-gray-400">{{ selectedTemplateDetail.language }}</span>
                  <span class="text-gray-600" aria-hidden="true">·</span>
                  <span class="text-sm text-gray-500">{{ selectedTemplateDetail.category }}</span>
                </div>
              </div>
            </div>
            <button
              @click="closeTemplateDetail"
              class="p-1.5 text-gray-400 hover:text-white transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
              aria-label="Close modal"
            >
              <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </header>

          <!-- Content -->
          <div class="flex-1 overflow-y-auto p-6">
            <!-- Description -->
            <section class="mb-6" aria-labelledby="description-heading">
              <h3 id="description-heading" class="text-sm font-semibold text-gray-300 mb-2">Description</h3>
              <p class="text-gray-400 text-sm leading-relaxed">{{ selectedTemplateDetail.fullDescription || selectedTemplateDetail.description }}</p>
            </section>

            <!-- Tags -->
            <section class="mb-6" aria-labelledby="tags-heading">
              <h3 id="tags-heading" class="text-sm font-semibold text-gray-300 mb-2">Tags</h3>
              <div class="flex flex-wrap gap-2" role="list">
                <span
                  v-for="tag in selectedTemplateDetail.tags"
                  :key="tag"
                  class="px-2.5 py-1 rounded-lg text-xs bg-gray-800 text-gray-300"
                  role="listitem"
                >
                  {{ tag }}
                </span>
              </div>
            </section>

            <!-- File Structure Preview -->
            <section class="mb-6" aria-labelledby="files-heading">
              <h3 id="files-heading" class="text-sm font-semibold text-gray-300 mb-2">File Structure</h3>
              <div class="bg-gray-950 rounded-lg border border-gray-800 p-4 font-mono text-xs overflow-x-auto">
                <pre class="text-gray-300 whitespace-pre">{{ selectedTemplateDetail.fileStructure || 'Project structure preview coming soon...' }}</pre>
              </div>
            </section>

            <!-- Stats -->
            <section class="grid grid-cols-2 gap-4 mb-6" aria-labelledby="stats-heading">
              <h3 id="stats-heading" class="sr-only">Template statistics</h3>
              <div class="bg-gray-800/50 rounded-lg p-4">
                <div class="text-2xl font-bold text-cyan-400">{{ formatCount(selectedTemplateDetail.useCount) }}</div>
                <div class="text-xs text-gray-500">Projects created</div>
              </div>
              <div class="bg-gray-800/50 rounded-lg p-4">
                <div class="text-2xl font-bold text-purple-400">{{ selectedTemplateDetail.stars || 0 }}</div>
                <div class="text-xs text-gray-500">Stars</div>
              </div>
            </section>

            <!-- Source Link -->
            <section v-if="selectedTemplateDetail.sourceUrl" class="mb-6" aria-labelledby="source-heading">
              <h3 id="source-heading" class="text-sm font-semibold text-gray-300 mb-2">Source</h3>
              <a
                :href="selectedTemplateDetail.sourceUrl"
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center gap-2 text-sm text-cyan-400 hover:text-cyan-300 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
              >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
                </svg>
                View on GitHub
                <span class="sr-only">(opens in new tab)</span>
              </a>
            </section>
          </div>

          <!-- Footer -->
          <footer class="px-6 py-4 bg-gray-900/50 border-t border-gray-800 flex flex-col-reverse sm:flex-row justify-between items-center gap-3 shrink-0">
            <button
              @click="closeTemplateDetail"
              class="w-full sm:w-auto px-4 py-2.5 sm:py-2 text-sm text-gray-400 hover:text-white transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded-lg min-h-[44px] sm:min-h-0"
            >
              Cancel
            </button>
            <NuxtLink
              :to="`/?template=${selectedTemplateDetail.id}`"
              class="w-full sm:w-auto bg-cyan-600 hover:bg-cyan-500 text-white px-6 py-2.5 sm:py-2 rounded-lg text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 text-center min-h-[44px] sm:min-h-0"
              @click="closeTemplateDetail"
            >
              Use This Template
            </NuxtLink>
          </footer>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
interface Template {
  id: string
  name: string
  description: string
  fullDescription?: string
  language: string
  category: string
  tags: string[]
  useCount: number
  stars?: number
  updatedAt: string
  sourceUrl?: string
  fileStructure?: string
}

// State
const templates = ref<Template[]>([])
const loading = ref(true)
const error = ref<string | null>(null)
const searchQuery = ref('')
const selectedCategory = ref('')
const sortBy = ref('popularity')
const selectedTemplateDetail = ref<Template | null>(null)

// Categories derived from templates
const categories = computed(() => {
  const cats = new Set(templates.value.map(t => t.category))
  return Array.from(cats).sort()
})

// Filtered and sorted templates
const filteredTemplates = computed(() => {
  let result = templates.value

  // Filter by search query
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(t =>
      t.name.toLowerCase().includes(query) ||
      t.description.toLowerCase().includes(query) ||
      t.tags.some(tag => tag.toLowerCase().includes(query))
    )
  }

  // Filter by category
  if (selectedCategory.value) {
    result = result.filter(t => t.category === selectedCategory.value)
  }

  // Sort
  switch (sortBy.value) {
    case 'name':
      result = [...result].sort((a, b) => a.name.localeCompare(b.name))
      break
    case 'updated':
      result = [...result].sort((a, b) =>
        new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
      )
      break
    case 'popularity':
    default:
      result = [...result].sort((a, b) => b.useCount - a.useCount)
  }

  return result
})

// Fetch templates from API
const fetchTemplates = async () => {
  loading.value = true
  error.value = null

  try {
    const config = useRuntimeConfig()
    const data = await $fetch<Template[]>(`${config.public.apiBase}/api/templates`)
    templates.value = data
  } catch (e) {
    console.error('Failed to fetch templates:', e)
    error.value = e instanceof Error ? e.message : 'An unexpected error occurred'
    templates.value = []
  } finally {
    loading.value = false
  }
}

// Helper functions
const getCategoryIcon = (category: string): string => {
  const icons: Record<string, string> = {
    'Web': '🌐',
    'CLI': '⌨️',
    'API': '🔌',
    'Library': '📚',
    'Bot': '🤖',
  }
  return icons[category] || '📦'
}

const getCategoryColor = (category: string): string => {
  const colors: Record<string, string> = {
    'Web': 'bg-blue-900/50 text-blue-400',
    'CLI': 'bg-green-900/50 text-green-400',
    'API': 'bg-purple-900/50 text-purple-400',
    'Library': 'bg-yellow-900/50 text-yellow-400',
    'Bot': 'bg-pink-900/50 text-pink-400',
  }
  return colors[category] || 'bg-gray-800 text-gray-400'
}

const getCategoryBadgeColor = (category: string): string => {
  const colors: Record<string, string> = {
    'Web': 'bg-blue-900/30 text-blue-400',
    'CLI': 'bg-green-900/30 text-green-400',
    'API': 'bg-purple-900/30 text-purple-400',
    'Library': 'bg-yellow-900/30 text-yellow-400',
    'Bot': 'bg-pink-900/30 text-pink-400',
  }
  return colors[category] || 'bg-gray-800 text-gray-400'
}

const formatCount = (count: number): string => {
  if (count >= 1000) {
    return `${(count / 1000).toFixed(1)}k`
  }
  return String(count)
}

const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24))

  if (diffDays === 0) return 'today'
  if (diffDays === 1) return 'yesterday'
  if (diffDays < 7) return `${diffDays} days ago`
  if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`
  if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`
  return `${Math.floor(diffDays / 365)} years ago`
}

const openTemplateDetail = (template: Template) => {
  selectedTemplateDetail.value = template
}

const closeTemplateDetail = () => {
  selectedTemplateDetail.value = null
}

// Close modal on escape key
onMounted(() => {
  fetchTemplates()

  const handleEscape = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && selectedTemplateDetail.value) {
      closeTemplateDetail()
    }
  }
  window.addEventListener('keydown', handleEscape)

  onUnmounted(() => {
    window.removeEventListener('keydown', handleEscape)
  })
})

// SEO
useHead({
  title: 'Templates - Cuttlefish',
  meta: [
    { name: 'description', content: 'Browse and use project templates to quickly start new projects with Cuttlefish.' }
  ]
})
</script>

<style scoped>
/* Line clamp utility */
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Focus visible styles */
:focus-visible {
  outline: 2px solid theme('colors.cyan.400');
  outline-offset: 2px;
}

/* Prefers reduced motion */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
</style>
