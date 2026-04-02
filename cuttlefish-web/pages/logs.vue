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
  <div class="p-6">
    <div class="mb-8">
      <h1 class="text-2xl font-bold text-white mb-2">Agent Activity</h1>
      <p class="text-gray-400">Timeline of agent actions</p>
    </div>
    
    <div class="flex gap-4 mb-6">
      <select v-model="selectedProject" class="px-4 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white">
        <option v-for="p in projects" :key="p" :value="p === 'All' ? '' : p">{{ p }}</option>
      </select>
      <select v-model="selectedAgent" class="px-4 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white">
        <option v-for="a in agents" :key="a" :value="a === 'All' ? '' : a">{{ a }}</option>
      </select>
    </div>
    
    <div class="space-y-3">
      <div 
        v-for="log in filteredLogs" 
        :key="log.id"
        class="flex items-center gap-4 p-4 bg-gray-900 border border-gray-800 rounded-lg"
      >
        <span class="text-gray-500 text-sm w-20">{{ log.timestamp }}</span>
        <span :class="['px-2 py-1 rounded text-xs font-medium', agentColors[log.agent]]">
          {{ log.agent }}
        </span>
        <span class="text-white flex-1">{{ log.action }}</span>
        <span class="text-gray-600 text-sm">{{ log.project }}</span>
      </div>
    </div>
    
    <div v-if="filteredLogs.length === 0" class="text-center py-12 text-gray-400">
      No activity found
    </div>
  </div>
</template>