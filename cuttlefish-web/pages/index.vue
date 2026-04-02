<template>
  <div class="min-h-screen bg-gray-950 text-white">
    <header class="bg-gray-900 border-b border-gray-800 px-6 py-3 flex items-center gap-4">
      <span class="text-xl font-bold text-cyan-400">🐙 Cuttlefish</span>
      <span class="text-sm text-gray-400">Multi-Agent Coding Platform</span>
      <span class="ml-auto text-sm" :class="connected ? 'text-green-400' : 'text-red-400'">
        {{ connected ? '● Connected' : '● Disconnected' }}
      </span>
    </header>
    
    <main class="max-w-6xl mx-auto px-6 py-8">
      <div class="bg-gray-900 rounded-xl border border-gray-800 p-6 mb-6">
        <h2 class="text-lg font-semibold mb-4">New Project</h2>
        <div class="flex gap-3">
          <input
            v-model="newProjectName"
            placeholder="Project name"
            class="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm focus:outline-none focus:border-cyan-500"
          />
          <input
            v-model="newProjectDesc"
            placeholder="Description"
            class="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm focus:outline-none focus:border-cyan-500"
          />
          <button
            @click="createProject"
            class="bg-cyan-600 hover:bg-cyan-500 text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors"
          >
            Create
          </button>
        </div>
      </div>

      <div v-if="projects.length" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <NuxtLink
          v-for="project in projects"
          :key="project.id"
          :to="`/project/${project.id}`"
          class="bg-gray-900 rounded-xl border border-gray-800 p-5 hover:border-cyan-800 transition-colors cursor-pointer"
        >
          <div class="flex items-start justify-between mb-2">
            <h3 class="font-semibold text-white">{{ project.name }}</h3>
            <span class="text-xs px-2 py-0.5 rounded-full" :class="project.status === 'active' ? 'bg-green-900 text-green-300' : 'bg-gray-800 text-gray-400'">
              {{ project.status }}
            </span>
          </div>
          <p class="text-sm text-gray-400 line-clamp-2">{{ project.description }}</p>
        </NuxtLink>
      </div>
      <div v-else class="text-center py-16 text-gray-500">
        <div class="text-4xl mb-4">🐙</div>
        <p>No projects yet. Create your first project above.</p>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
const { connected } = useWebSocket()

interface Project { id: string; name: string; description: string; status: string }

const projects = ref<Project[]>([])
const newProjectName = ref('')
const newProjectDesc = ref('')

const createProject = async () => {
  if (!newProjectName.value) return
  try {
    const config = useRuntimeConfig()
    const res = await $fetch<Project>(`${config.public.apiBase}/api/projects`, {
      method: 'POST',
      body: { name: newProjectName.value, description: newProjectDesc.value || 'No description' },
    })
    projects.value.unshift(res)
    newProjectName.value = ''
    newProjectDesc.value = ''
  } catch (e) {
    console.error('Failed to create project', e)
  }
}
</script>