<template>
  <div class="p-4 sm:p-6">
    <div class="max-w-6xl mx-auto">
      <!-- New Project Form -->
      <section class="bg-gray-900 rounded-xl border border-gray-800 p-4 sm:p-6 mb-6" aria-labelledby="new-project-heading">
        <h2 id="new-project-heading" class="text-lg font-semibold mb-4">New Project</h2>
        <form class="flex flex-col gap-3" @submit.prevent="createProject">
          <div class="flex flex-col sm:flex-row gap-3">
            <div class="flex-1">
              <label for="project-name" class="sr-only">Project name</label>
              <input
                id="project-name"
                ref="nameInputRef"
                v-model="newProjectName"
                placeholder="Project name"
                class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 sm:py-2 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none min-h-[44px]"
                autocomplete="off"
              />
            </div>
            <div class="flex-1">
              <label for="project-desc" class="sr-only">Description</label>
              <input
                id="project-desc"
                v-model="newProjectDesc"
                placeholder="Description"
                class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 sm:py-2 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none min-h-[44px]"
                autocomplete="off"
              />
            </div>
          </div>
          <div class="flex flex-col sm:flex-row gap-3 items-stretch sm:items-center">
            <!-- Template Selector -->
            <div class="relative flex-1">
              <label for="template-select" class="sr-only">Project template</label>
              <select
                id="template-select"
                v-model="selectedTemplate"
                class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 sm:py-2 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none appearance-none cursor-pointer text-gray-200 min-h-[44px]"
              >
                <option value="">No template (blank project)</option>
                <option v-for="tpl in templates" :key="tpl.id" :value="tpl.id">
                  {{ tpl.name }} — {{ tpl.language }}
                </option>
              </select>
              <svg class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
              </svg>
            </div>
            <button
              type="submit"
              :disabled="!newProjectName || isCreating"
              class="bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed text-white px-6 py-3 sm:py-2 rounded-lg text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 min-h-[44px] sm:min-h-0 flex items-center justify-center gap-2"
            >
              <svg v-if="isCreating" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
              </svg>
              {{ isCreating ? 'Creating...' : 'Create' }}
            </button>
          </div>
          <!-- Template Preview -->
          <Transition name="slide-fade">
            <div v-if="selectedTemplateInfo" class="mt-2 p-3 bg-gray-800/50 rounded-lg border border-gray-700/50" role="region" aria-label="Selected template details">
              <div class="flex items-center gap-2 text-sm">
                <span class="text-cyan-400">{{ selectedTemplateInfo.name }}</span>
                <span class="text-gray-500" aria-hidden="true">·</span>
                <span class="text-gray-400">{{ selectedTemplateInfo.language }}</span>
              </div>
              <p class="text-xs text-gray-500 mt-1">{{ selectedTemplateInfo.description }}</p>
            </div>
          </Transition>
        </form>
      </section>

      <!-- Toolbar: Sort, Filter, Search -->
      <section v-if="projects.length || searchQuery || statusFilter !== 'all'" class="mb-4 flex flex-col sm:flex-row gap-3 items-stretch sm:items-center" aria-label="Project filters">
        <!-- Search -->
        <div class="relative flex-1">
          <label for="search-input" class="sr-only">Search projects</label>
          <svg class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
          <input
            id="search-input"
            v-model="searchQuery"
            type="search"
            placeholder="Search projects..."
            class="w-full bg-gray-800 border border-gray-700 rounded-lg pl-10 pr-4 py-2 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none min-h-[44px]"
          />
        </div>
        
        <!-- Status Filter -->
        <div class="flex gap-2">
          <div class="relative">
            <label for="status-filter" class="sr-only">Filter by status</label>
            <select
              id="status-filter"
              v-model="statusFilter"
              class="appearance-none bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 pr-10 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none cursor-pointer min-h-[44px]"
            >
              <option value="all">All Status</option>
              <option value="active">Active</option>
              <option value="idle">Idle</option>
              <option value="archived">Archived</option>
            </select>
            <svg class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </div>
          
          <!-- Sort -->
          <div class="relative">
            <label for="sort-select" class="sr-only">Sort projects</label>
            <select
              id="sort-select"
              v-model="sortBy"
              class="appearance-none bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 pr-10 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none cursor-pointer min-h-[44px]"
            >
              <option value="newest">Newest</option>
              <option value="oldest">Oldest</option>
              <option value="name">Name A-Z</option>
              <option value="name-desc">Name Z-A</option>
            </select>
            <svg class="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </div>
        </div>
      </section>

      <!-- Projects Grid -->
      <section v-if="filteredProjects.length" aria-labelledby="projects-heading">
        <h2 id="projects-heading" class="sr-only">Your Projects</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <article
            v-for="project in filteredProjects"
            :key="project.id"
            class="bg-gray-900 rounded-xl border border-gray-800 p-5 hover:border-cyan-800 transition-all duration-200 motion-reduce:transition-none cursor-pointer group focus-within:ring-2 focus-within:ring-cyan-400 focus-within:ring-offset-2 focus-within:ring-offset-gray-950 relative"
            :class="{ 'opacity-60': project.isArchived }"
            tabindex="0"
            @click="navigateToProject(project.id)"
            @keydown.enter="navigateToProject(project.id)"
            @keydown.space.prevent="navigateToProject(project.id)"
          >
            <!-- Card Header -->
            <div class="flex items-start justify-between mb-3">
              <div class="flex items-center gap-2 min-w-0">
                <h3 class="font-semibold text-white group-hover:text-cyan-400 transition-colors motion-reduce:transition-none truncate" :class="{ 'line-through': project.isArchived }">
                  {{ project.name }}
                </h3>
                <!-- Template Badge -->
                <span
                  v-if="project.template"
                  class="text-xs px-2 py-0.5 rounded-full bg-purple-900/50 text-purple-300 border border-purple-700/50 shrink-0"
                >
                  {{ project.template }}
                </span>
              </div>
              <!-- Status Indicator -->
              <TerminalStatusBadge
                :status="project.isArchived ? 'pending' : project.status === 'active' ? 'success' : 'info'"
                :label="project.isArchived ? 'Archived' : project.status"
                class="shrink-0"
              />
            </div>
            
            <!-- Description -->
            <p class="text-sm text-gray-400 line-clamp-2 mb-4">{{ project.description || 'No description' }}</p>
            
            <!-- Meta Info -->
            <div class="flex items-center gap-3 text-xs text-gray-500 mb-3">
              <span class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                {{ formatRelativeTime(project.updatedAt) }}
              </span>
              <span v-if="project.messageCount" class="flex items-center gap-1">
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
                </svg>
                {{ project.messageCount }} messages
              </span>
            </div>
            
            <!-- Quick Actions -->
            <div class="flex items-center gap-2 pt-3 border-t border-gray-800">
              <button
                @click.stop="toggleArchive(project)"
                class="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-xs rounded-lg transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                :class="project.isArchived 
                  ? 'bg-green-900/30 text-green-400 hover:bg-green-900/50 border border-green-700/30' 
                  : 'bg-gray-800 text-gray-400 hover:bg-gray-700 border border-gray-700'"
                :aria-label="project.isArchived ? 'Restore project' : 'Archive project'"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path v-if="project.isArchived" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                  <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
                </svg>
                {{ project.isArchived ? 'Restore' : 'Archive' }}
              </button>
              <button
                @click.stop="duplicateProject(project)"
                class="flex-1 flex items-center justify-center gap-1.5 px-3 py-2 text-xs rounded-lg bg-gray-800 text-gray-400 hover:bg-gray-700 border border-gray-700 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                aria-label="Duplicate project"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                </svg>
                Duplicate
              </button>
              <button
                @click.stop="confirmDelete(project)"
                class="flex items-center justify-center gap-1.5 px-3 py-2 text-xs rounded-lg bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700/30 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400"
                aria-label="Delete project"
              >
                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                </svg>
              </button>
            </div>
          </article>
        </div>
      </section>

      <!-- Loading Skeletons -->
      <section v-else-if="isLoading" aria-label="Loading projects">
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div v-for="i in 6" :key="i" class="bg-gray-900 rounded-xl border border-gray-800 p-5 animate-pulse">
            <div class="flex items-start justify-between mb-3">
              <div class="h-5 bg-gray-800 rounded w-1/3" />
              <div class="h-5 bg-gray-800 rounded-full w-16" />
            </div>
            <div class="space-y-2 mb-4">
              <div class="h-4 bg-gray-800 rounded w-full" />
              <div class="h-4 bg-gray-800 rounded w-2/3" />
            </div>
            <div class="flex gap-2 pt-3 border-t border-gray-800">
              <div class="h-8 bg-gray-800 rounded flex-1" />
              <div class="h-8 bg-gray-800 rounded flex-1" />
              <div class="h-8 bg-gray-800 rounded w-12" />
            </div>
          </div>
        </div>
      </section>

      <!-- Empty State -->
      <div v-else class="text-center py-16" role="status">
        <div class="max-w-md mx-auto">
          <div class="relative inline-block mb-6">
            <div class="text-6xl" role="img" aria-label="Cuttlefish mascot">🐙</div>
            <div class="absolute -top-1 -right-1 w-4 h-4 bg-cyan-500 rounded-full animate-ping opacity-75" />
            <div class="absolute -top-1 -right-1 w-4 h-4 bg-cyan-500 rounded-full" />
          </div>
          <h3 class="text-xl font-semibold text-gray-200 mb-2">
            {{ searchQuery ? 'No matching projects' : 'No projects yet' }}
          </h3>
          <p class="text-gray-500 mb-6">
            {{ searchQuery 
              ? `No projects match "${searchQuery}". Try a different search term.`
              : 'Create your first project to get started with Cuttlefish.' }}
          </p>
          <button
            v-if="!searchQuery"
            @click="focusNameInput"
            class="inline-flex items-center gap-2 bg-cyan-600 hover:bg-cyan-500 text-white px-6 py-3 rounded-lg text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-950"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Create Your First Project
          </button>
          <button
            v-else
            @click="searchQuery = ''"
            class="inline-flex items-center gap-2 bg-gray-800 hover:bg-gray-700 text-gray-300 px-6 py-3 rounded-lg text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-950"
          >
            Clear Search
          </button>
        </div>
      </div>

      <!-- Error Toast -->
      <Transition name="slide-up">
        <div
          v-if="error"
          class="fixed bottom-4 right-4 bg-red-900/90 border border-red-700 rounded-lg px-4 py-3 shadow-lg flex items-center gap-3 z-50"
          role="alert"
        >
          <svg class="w-5 h-5 text-red-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span class="text-sm text-red-200">{{ error }}</span>
          <button
            @click="error = null"
            class="text-red-400 hover:text-red-300 transition-colors motion-reduce:transition-none"
            aria-label="Dismiss error"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
      </Transition>

      <!-- Delete Confirmation Modal -->
      <Teleport to="body">
        <Transition name="fade">
          <div v-if="showDeleteModal" class="fixed inset-0 z-50 flex items-center justify-center p-4" role="dialog" aria-modal="true" aria-labelledby="delete-modal-title">
            <!-- Backdrop -->
            <div 
              class="absolute inset-0 bg-black/70 backdrop-blur-sm"
              @click="showDeleteModal = false"
              aria-hidden="true"
            />
            <!-- Modal -->
            <div class="relative bg-gray-900 rounded-xl border border-gray-700 shadow-2xl max-w-md w-full overflow-hidden">
              <!-- Header -->
              <div class="px-6 py-4 border-b border-gray-800 flex items-center gap-3">
                <div class="w-10 h-10 rounded-full bg-red-900/50 flex items-center justify-center" aria-hidden="true">
                  <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                </div>
                <div>
                  <h3 id="delete-modal-title" class="text-lg font-semibold text-white">Delete Project</h3>
                  <p class="text-sm text-gray-400">This action cannot be undone</p>
                </div>
              </div>
              <!-- Body -->
              <div class="px-6 py-4">
                <p class="text-gray-300">
                  Are you sure you want to delete <span class="font-semibold text-white">{{ projectToDelete?.name }}</span>? 
                  All files, history, and configuration will be permanently removed.
                </p>
              </div>
              <!-- Footer -->
              <div class="px-6 py-4 bg-gray-900/50 border-t border-gray-800 flex justify-end gap-3">
                <button
                  @click="showDeleteModal = false"
                  class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 rounded-lg"
                >
                  Cancel
                </button>
                <button
                  @click="deleteProject"
                  class="px-4 py-2 text-sm bg-red-600 hover:bg-red-500 text-white rounded-lg transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                >
                  Delete Permanently
                </button>
              </div>
            </div>
          </div>
        </Transition>
      </Teleport>
    </div>
  </div>
</template>

<script setup lang="ts">
interface Project {
  id: string
  name: string
  description: string
  status: string
  template?: string
  isArchived?: boolean
  updatedAt?: string
  messageCount?: number
}

interface Template {
  id: string
  name: string
  description: string
  language: string
}

// State
const templates = ref<Template[]>([])
const projects = ref<Project[]>([])
const newProjectName = ref('')
const newProjectDesc = ref('')
const selectedTemplate = ref('')
const searchQuery = ref('')
const statusFilter = ref<'all' | 'active' | 'idle' | 'archived'>('all')
const sortBy = ref<'newest' | 'oldest' | 'name' | 'name-desc'>('newest')
const isLoading = ref(false)
const isCreating = ref(false)
const error = ref<string | null>(null)
const showDeleteModal = ref(false)
const projectToDelete = ref<Project | null>(null)
const nameInputRef = ref<HTMLInputElement>()

// Computed
const selectedTemplateInfo = computed(() =>
  templates.value.find(t => t.id === selectedTemplate.value)
)

const filteredProjects = computed(() => {
  let result = [...projects.value]
  
  // Filter by search query
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(p => 
      p.name.toLowerCase().includes(query) ||
      p.description?.toLowerCase().includes(query) ||
      p.template?.toLowerCase().includes(query)
    )
  }
  
  // Filter by status
  if (statusFilter.value === 'archived') {
    result = result.filter(p => p.isArchived)
  } else if (statusFilter.value === 'active') {
    result = result.filter(p => p.status === 'active' && !p.isArchived)
  } else if (statusFilter.value === 'idle') {
    result = result.filter(p => p.status !== 'active' && !p.isArchived)
  } else {
    // 'all' - show non-archived by default
    result = result.filter(p => !p.isArchived)
  }
  
  // Sort
  result.sort((a, b) => {
    switch (sortBy.value) {
      case 'newest':
        return (b.updatedAt || '').localeCompare(a.updatedAt || '')
      case 'oldest':
        return (a.updatedAt || '').localeCompare(b.updatedAt || '')
      case 'name':
        return a.name.localeCompare(b.name)
      case 'name-desc':
        return b.name.localeCompare(a.name)
      default:
        return 0
    }
  })
  
  return result
})

// Methods
const formatRelativeTime = (dateStr?: string) => {
  if (!dateStr) return 'Unknown'
  const date = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)
  
  if (diffMins < 1) return 'Just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`
  return date.toLocaleDateString()
}

const navigateToProject = (id: string) => {
  navigateTo(`/project/${id}`)
}

const focusNameInput = () => {
  nameInputRef.value?.focus()
}

const createProject = async () => {
  if (!newProjectName.value || isCreating.value) return
  
  isCreating.value = true
  error.value = null
  
  try {
    const config = useRuntimeConfig()
    const body: Record<string, string> = {
      name: newProjectName.value,
      description: newProjectDesc.value || 'No description',
    }
    if (selectedTemplate.value) {
      body.template = selectedTemplate.value
    }
    const res = await $fetch<Project>(`${config.public.apiBase}/api/projects`, {
      method: 'POST',
      body,
    })
    projects.value.unshift({
      ...res,
      updatedAt: new Date().toISOString(),
      messageCount: 0,
    })
    newProjectName.value = ''
    newProjectDesc.value = ''
    selectedTemplate.value = ''
  } catch (e) {
    console.error('Failed to create project', e)
    error.value = 'Failed to create project. Please try again.'
  } finally {
    isCreating.value = false
  }
}

const toggleArchive = async (project: Project) => {
  try {
    const config = useRuntimeConfig()
    await $fetch(`${config.public.apiBase}/api/projects/${project.id}/archive`, {
      method: 'POST',
      body: { archive: !project.isArchived },
    })
    project.isArchived = !project.isArchived
  } catch (e) {
    console.error('Failed to toggle archive', e)
    error.value = 'Failed to update project. Please try again.'
  }
}

const duplicateProject = async (project: Project) => {
  try {
    const config = useRuntimeConfig()
    const res = await $fetch<Project>(`${config.public.apiBase}/api/projects`, {
      method: 'POST',
      body: {
        name: `${project.name} (copy)`,
        description: project.description,
        template: project.template,
      },
    })
    projects.value.unshift({
      ...res,
      updatedAt: new Date().toISOString(),
      messageCount: 0,
    })
  } catch (e) {
    console.error('Failed to duplicate project', e)
    error.value = 'Failed to duplicate project. Please try again.'
  }
}

const confirmDelete = (project: Project) => {
  projectToDelete.value = project
  showDeleteModal.value = true
}

const deleteProject = async () => {
  if (!projectToDelete.value) return
  
  try {
    const config = useRuntimeConfig()
    await $fetch(`${config.public.apiBase}/api/projects/${projectToDelete.value.id}`, {
      method: 'DELETE',
    })
    projects.value = projects.value.filter(p => p.id !== projectToDelete.value!.id)
    showDeleteModal.value = false
    projectToDelete.value = null
  } catch (e) {
    console.error('Failed to delete project', e)
    error.value = 'Failed to delete project. Please try again.'
  }
}

// Fetch projects and templates on mount
onMounted(async () => {
  isLoading.value = true
  try {
    const config = useRuntimeConfig()
    
    // Fetch projects and templates in parallel
    const [projectsRes, templatesRes] = await Promise.all([
      $fetch<Project[]>(`${config.public.apiBase}/api/projects`).catch(() => []),
      $fetch<Template[]>(`${config.public.apiBase}/api/templates`).catch(() => [])
    ])
    
    projects.value = projectsRes.map(p => ({
      ...p,
      updatedAt: p.updatedAt || new Date().toISOString(),
      messageCount: p.messageCount || 0,
    }))
    
    templates.value = templatesRes
  } catch (e) {
    console.error('Failed to fetch data', e)
  } finally {
    isLoading.value = false
  }
})

// Keyboard shortcuts
onMounted(() => {
  const handleKeydown = (e: KeyboardEvent) => {
    // Ctrl/Cmd + K to focus search
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
      e.preventDefault()
      document.getElementById('search-input')?.focus()
    }
    // Ctrl/Cmd + N to focus new project name
    if ((e.ctrlKey || e.metaKey) && e.key === 'n') {
      e.preventDefault()
      focusNameInput()
    }
  }
  window.addEventListener('keydown', handleKeydown)
  onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
})
</script>

<style scoped>
/* Transitions */
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.2s ease;
}

.slide-fade-enter-from,
.slide-fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(20px);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
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

/* Focus visible styles for keyboard navigation */
.focus-visible:focus-visible,
button:focus-visible,
a:focus-visible,
input:focus-visible,
select:focus-visible,
textarea:focus-visible {
  outline: 2px solid #00d4aa;
  outline-offset: 2px;
}

/* Remove default focus outline when using focus-visible */
button:focus:not(:focus-visible),
a:focus:not(:focus-visible),
input:focus:not(:focus-visible),
select:focus:not(:focus-visible),
textarea:focus:not(:focus-visible) {
  outline: none;
}

/* Card hover effect */
article {
  transform: translateY(0);
}

article:hover {
  transform: translateY(-2px);
}

@media (prefers-reduced-motion: reduce) {
  article:hover {
    transform: none;
  }
}
</style>