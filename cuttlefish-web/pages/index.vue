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
              :disabled="!newProjectName"
              class="bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed text-white px-6 py-3 sm:py-2 rounded-lg text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 min-h-[44px] sm:min-h-0"
            >
              Create
            </button>
          </div>
          <!-- Template Preview -->
          <div v-if="selectedTemplateInfo" class="mt-2 p-3 bg-gray-800/50 rounded-lg border border-gray-700/50" role="region" aria-label="Selected template details">
            <div class="flex items-center gap-2 text-sm">
              <span class="text-cyan-400">{{ selectedTemplateInfo.name }}</span>
              <span class="text-gray-500" aria-hidden="true">·</span>
              <span class="text-gray-400">{{ selectedTemplateInfo.language }}</span>
            </div>
            <p class="text-xs text-gray-500 mt-1">{{ selectedTemplateInfo.description }}</p>
          </div>
        </form>
      </section>

      <!-- Projects Grid -->
      <section v-if="projects.length" aria-labelledby="projects-heading">
        <h2 id="projects-heading" class="sr-only">Your Projects</h2>
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <NuxtLink
            v-for="project in projects"
            :key="project.id"
            :to="`/project/${project.id}`"
            class="bg-gray-900 rounded-xl border border-gray-800 p-5 hover:border-cyan-800 transition-colors motion-reduce:transition-none cursor-pointer group focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-950"
          >
            <article>
              <div class="flex items-start justify-between mb-2">
                <div class="flex items-center gap-2">
                  <h3 class="font-semibold text-white group-hover:text-cyan-400 transition-colors motion-reduce:transition-none">{{ project.name }}</h3>
                  <!-- Template Badge -->
                  <span
                    v-if="project.template"
                    class="text-xs px-2 py-0.5 rounded-full bg-purple-900/50 text-purple-300 border border-purple-700/50"
                  >
                    {{ project.template }}
                  </span>
                </div>
                <span
                  class="text-xs px-2 py-0.5 rounded-full"
                  :class="project.status === 'active' ? 'bg-green-900 text-green-300' : 'bg-gray-800 text-gray-400'"
                  role="status"
                >
                  {{ project.status }}
                </span>
              </div>
              <p class="text-sm text-gray-400 line-clamp-2">{{ project.description }}</p>
            </article>
          </NuxtLink>
        </div>
      </section>

      <!-- Empty State -->
      <div v-else class="text-center py-16 text-gray-500" role="status">
        <div class="text-4xl mb-4" role="img" aria-label="Cuttlefish mascot">🐙</div>
        <p>No projects yet. Create your first project above.</p>
      </div>
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
}

interface Template {
  id: string
  name: string
  description: string
  language: string
}

// Available templates (will be fetched from API later)
const templates: Template[] = [
  { id: 'rust-cli', name: 'Rust CLI', description: 'Command-line application with argument parsing and logging', language: 'Rust' },
  { id: 'rust-lib', name: 'Rust Library', description: 'Library with tests, documentation, and CI/CD', language: 'Rust' },
  { id: 'nuxt-app', name: 'Nuxt App', description: 'Nuxt 3 web application with Tailwind CSS', language: 'TypeScript' },
  { id: 'fastapi', name: 'FastAPI', description: 'Python FastAPI backend with async support', language: 'Python' },
  { id: 'discord-bot', name: 'Discord Bot', description: 'Discord bot starter with slash commands', language: 'TypeScript' },
  { id: 'go-microservice', name: 'Go Microservice', description: 'Go microservice with gRPC and Docker', language: 'Go' },
]

const projects = ref<Project[]>([])
const newProjectName = ref('')
const newProjectDesc = ref('')
const selectedTemplate = ref('')

const selectedTemplateInfo = computed(() =>
  templates.find(t => t.id === selectedTemplate.value)
)

const createProject = async () => {
  if (!newProjectName.value) return
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
    projects.value.unshift(res)
    newProjectName.value = ''
    newProjectDesc.value = ''
    selectedTemplate.value = ''
  } catch (e) {
    console.error('Failed to create project', e)
  }
}
</script>