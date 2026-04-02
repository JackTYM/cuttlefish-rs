<script setup lang="ts">
const selectedProject = ref('')
const selectedAgent = ref('')

const logs = ref([
  { id: '1', timestamp: '10:30 AM', agent: 'orchestrator', project: 'my-app', action: 'Planning implementation' },
  { id: '2', timestamp: '10:31 AM', agent: 'coder', project: 'my-app', action: 'Writing authentication middleware' },
  { id: '3', timestamp: '10:32 AM', agent: 'coder', project: 'my-app', action: 'Created src/auth/middleware.rs' },
  { id: '4', timestamp: '10:33 AM', agent: 'critic', project: 'my-app', action: 'Reviewing code changes' },
  { id: '5', timestamp: '10:34 AM', agent: 'critic', project: 'my-app', action: 'Approved changes' },
])

const projects = ['All', 'my-app', 'other-project']
const agents = ['All', 'orchestrator', 'coder', 'critic']

const agentColors: Record<string, string> = {
  orchestrator: 'bg-purple-500/20 text-purple-400',
  coder: 'bg-yellow-500/20 text-yellow-400',
  critic: 'bg-red-500/20 text-red-400',
}

const filteredLogs = computed(() => {
  return logs.value.filter(log => {
    const matchesProject = !selectedProject.value || selectedProject.value === 'All' || log.project === selectedProject.value
    const matchesAgent = !selectedAgent.value || selectedAgent.value === 'All' || log.agent === selectedAgent.value
    return matchesProject && matchesAgent
  })
})
</script>

<template>
  <div class="p-4 sm:p-6">
    <header class="mb-6 sm:mb-8">
      <h1 class="text-xl sm:text-2xl font-bold text-white mb-2">Agent Activity</h1>
      <p class="text-gray-400 text-sm sm:text-base">Timeline of agent actions</p>
    </header>
    
    <div class="flex flex-col sm:flex-row gap-3 sm:gap-4 mb-6" role="group" aria-label="Filter logs">
      <div>
        <label for="project-filter" class="sr-only">Filter by project</label>
        <select 
          id="project-filter"
          v-model="selectedProject" 
          class="w-full px-4 py-3 sm:py-2 bg-gray-900 border border-gray-700 rounded-lg text-white text-sm min-h-[44px] focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none"
        >
          <option v-for="p in projects" :key="p" :value="p === 'All' ? '' : p">{{ p }}</option>
        </select>
      </div>
      <div>
        <label for="agent-filter" class="sr-only">Filter by agent</label>
        <select 
          id="agent-filter"
          v-model="selectedAgent" 
          class="w-full px-4 py-3 sm:py-2 bg-gray-900 border border-gray-700 rounded-lg text-white text-sm min-h-[44px] focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none"
        >
          <option v-for="a in agents" :key="a" :value="a === 'All' ? '' : a">{{ a }}</option>
        </select>
      </div>
    </div>
    
    <section aria-labelledby="activity-heading">
      <h2 id="activity-heading" class="sr-only">Activity Log</h2>
      <ul class="space-y-3" role="log" aria-live="polite">
        <li 
          v-for="log in filteredLogs" 
          :key="log.id"
          class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4 p-4 bg-gray-900 border border-gray-800 rounded-lg"
        >
          <!-- Mobile: Top row with timestamp and agent -->
          <div class="flex items-center gap-2 sm:hidden">
            <time class="text-gray-500 text-sm">{{ log.timestamp }}</time>
            <span :class="['px-2 py-1 rounded text-xs font-medium', agentColors[log.agent]]">
              {{ log.agent }}
            </span>
          </div>
          
          <!-- Desktop: Horizontal layout -->
          <time class="hidden sm:block text-gray-500 text-sm w-20 flex-shrink-0">{{ log.timestamp }}</time>
          <span 
            :class="['hidden sm:inline-block px-2 py-1 rounded text-xs font-medium flex-shrink-0', agentColors[log.agent]]"
          >
            {{ log.agent }}
          </span>
          <span class="text-white flex-1 text-sm">{{ log.action }}</span>
          <span class="text-gray-600 text-xs sm:text-sm">{{ log.project }}</span>
        </li>
      </ul>
    </section>
    
    <div v-if="filteredLogs.length === 0" class="text-center py-12 text-gray-400" role="status">
      No activity found
    </div>
  </div>
</template>
