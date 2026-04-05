<script setup lang="ts">
/**
 * Agent Activity Logs Page
 * Real-time log stream with filtering, expansion, and live/pause toggle
 */

// Types
interface LogEntry {
  id: string
  timestamp: string
  agent: string
  action: string
  level: 'info' | 'warn' | 'error'
  project: string
  context?: string
  stackTrace?: string
  metadata?: Record<string, unknown>
}

// WebSocket integration
const { connected } = useWebSocket()

// Filter state
const selectedProject = ref('')
const selectedAgent = ref('')
const selectedLevel = ref('')
const dateFrom = ref('')
const dateTo = ref('')

// Live mode state
const isLive = ref(true)
const logContainer = ref<HTMLElement>()

// Expanded log state
const expandedLogId = ref<string | null>(null)

// Virtualization - visible range
const scrollTop = ref(0)
const itemHeight = 72 // Approximate height of each log item
const containerHeight = ref(600)
const overscan = 5

// Real log entries - populated from WebSocket
const allLogs = ref<LogEntry[]>([])

// Available filter options
const projects = computed(() => {
  const set = new Set(allLogs.value.map(l => l.project))
  return ['All Projects', ...Array.from(set)]
})

const agents = ['All Agents', 'orchestrator', 'planner', 'coder', 'critic', 'explorer', 'librarian', 'devops']

const levels = ['All Levels', 'info', 'warn', 'error']

// Agent color mapping (matching project/[id].vue pattern)
const agentColors: Record<string, string> = {
  orchestrator: 'bg-purple-700',
  planner: 'bg-indigo-700',
  coder: 'bg-yellow-700',
  critic: 'bg-red-700',
  explorer: 'bg-green-700',
  librarian: 'bg-blue-700',
  devops: 'bg-orange-700',
}

const agentBadgeColors: Record<string, string> = {
  orchestrator: 'bg-purple-500/20 text-purple-400 border-purple-500/30',
  planner: 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30',
  coder: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30',
  critic: 'bg-red-500/20 text-red-400 border-red-500/30',
  explorer: 'bg-green-500/20 text-green-400 border-green-500/30',
  librarian: 'bg-blue-500/20 text-blue-400 border-blue-500/30',
  devops: 'bg-orange-500/20 text-orange-400 border-orange-500/30',
}

// Level color mapping
const levelColors: Record<string, string> = {
  info: 'text-cyan-400',
  warn: 'text-yellow-400',
  error: 'text-red-400',
}

const levelBadgeColors: Record<string, string> = {
  info: 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30',
  warn: 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30',
  error: 'bg-red-500/20 text-red-400 border-red-500/30',
}

// Filtered logs
const filteredLogs = computed(() => {
  return allLogs.value.filter(log => {
    const matchesProject = !selectedProject.value || selectedProject.value === 'All Projects' || log.project === selectedProject.value
    const matchesAgent = !selectedAgent.value || selectedAgent.value === 'All Agents' || log.agent === selectedAgent.value
    const matchesLevel = !selectedLevel.value || selectedLevel.value === 'All Levels' || log.level === selectedLevel.value
    
    let matchesDateRange = true
    if (dateFrom.value) {
      const fromDate = new Date(dateFrom.value).getTime()
      const logDate = new Date(log.timestamp).getTime()
      matchesDateRange = matchesDateRange && logDate >= fromDate
    }
    if (dateTo.value) {
      const toDate = new Date(dateTo.value).getTime() + 86400000 // Add 1 day to include end date
      const logDate = new Date(log.timestamp).getTime()
      matchesDateRange = matchesDateRange && logDate <= toDate
    }
    
    return matchesProject && matchesAgent && matchesLevel && matchesDateRange
  })
})

// Virtualization calculations
const visibleRange = computed(() => {
  const start = Math.max(0, Math.floor(scrollTop.value / itemHeight) - overscan)
  const end = Math.min(
    filteredLogs.value.length,
    Math.ceil((scrollTop.value + containerHeight.value) / itemHeight) + overscan
  )
  return { start, end }
})

const visibleLogs = computed(() => {
  return filteredLogs.value.slice(visibleRange.value.start, visibleRange.value.end)
})

const totalHeight = computed(() => filteredLogs.value.length * itemHeight)

const offsetY = computed(() => visibleRange.value.start * itemHeight)

// Format timestamp for display
const formatTimestamp = (iso: string) => {
  const date = new Date(iso)
  return date.toLocaleTimeString('en-US', { 
    hour: '2-digit', 
    minute: '2-digit',
    second: '2-digit',
    hour12: false 
  })
}

const formatDate = (iso: string) => {
  const date = new Date(iso)
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
}

// Toggle log expansion
const toggleExpand = (logId: string) => {
  expandedLogId.value = expandedLogId.value === logId ? null : logId
}

// Copy log entry to clipboard
const copyLog = async (log: LogEntry) => {
  const text = `[${log.timestamp}] [${log.agent.toUpperCase()}] [${log.level.toUpperCase()}] [${log.project}]\n${log.action}${log.context ? `\n\nContext:\n${log.context}` : ''}${log.stackTrace ? `\n\nStack Trace:\n${log.stackTrace}` : ''}`
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    // Fallback for older browsers
    const textarea = document.createElement('textarea')
    textarea.value = text
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
  }
}

// Handle scroll for virtualization and auto-scroll
const handleScroll = (event: Event) => {
  const target = event.target as HTMLElement
  scrollTop.value = target.scrollTop
  
  // If user scrolls up while in live mode, pause live mode
  if (isLive.value && target.scrollTop < target.scrollHeight - target.clientHeight - 100) {
    isLive.value = false
  }
}

// Toggle live mode
const toggleLive = () => {
  isLive.value = !isLive.value
  if (isLive.value && logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

// Auto-scroll when new logs arrive in live mode
watch([filteredLogs, isLive], () => {
  if (isLive.value && logContainer.value) {
    nextTick(() => {
      if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight
      }
    })
  }
}, { deep: true })

// Simulated log generation removed - logs come from real WebSocket events
// The WebSocket connection (useWebSocket composable) will push log entries

onMounted(() => {
  // Update container height on resize
  const updateContainerHeight = () => {
    if (logContainer.value) {
      containerHeight.value = logContainer.value.clientHeight
    }
  }
  
  updateContainerHeight()
  window.addEventListener('resize', updateContainerHeight)
})

onUnmounted(() => {
  // Cleanup handled by useWebSocket composable
})

// Clear filters
const clearFilters = () => {
  selectedProject.value = ''
  selectedAgent.value = ''
  selectedLevel.value = ''
  dateFrom.value = ''
  dateTo.value = ''
}
</script>

<template>
  <div class="flex flex-col h-full">
    <!-- Header -->
    <header class="bg-gray-900 border-b border-gray-800 px-4 sm:px-6 py-4 shrink-0">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <h1 class="text-xl sm:text-2xl font-bold text-white">Agent Activity</h1>
          <p class="text-gray-400 text-sm mt-1">Real-time log stream from all agents</p>
        </div>
        
        <!-- Live/Pause Toggle -->
        <div class="flex items-center gap-3">
          <div class="flex items-center gap-2 text-sm" role="status" aria-live="polite">
            <span 
              class="w-2 h-2 rounded-full transition-colors duration-300 motion-reduce:transition-none"
              :class="connected ? 'bg-green-400' : 'bg-red-400'"
              aria-hidden="true"
            />
            <span class="text-gray-400">{{ connected ? 'Connected' : 'Disconnected' }}</span>
          </div>
          
          <button
            @click="toggleLive"
            class="flex items-center gap-2 px-4 py-2 rounded-lg text-sm font-medium transition-all duration-200 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
            :class="isLive 
              ? 'bg-green-600 hover:bg-green-500 text-white shadow-lg shadow-green-500/20' 
              : 'bg-gray-800 hover:bg-gray-700 text-gray-300 border border-gray-700'"
            :aria-pressed="isLive"
            :aria-label="isLive ? 'Pause live updates' : 'Resume live updates'"
          >
            <svg v-if="isLive" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 9v6m4-6v6m7-3a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            {{ isLive ? 'Live' : 'Paused' }}
          </button>
        </div>
      </div>
    </header>
    
    <!-- Filters -->
    <div class="bg-gray-900/50 border-b border-gray-800 px-4 sm:px-6 py-3 shrink-0">
      <div class="flex flex-wrap items-center gap-3">
        <!-- Project Filter -->
        <div class="relative">
          <label for="project-filter" class="sr-only">Filter by project</label>
          <select
            id="project-filter"
            v-model="selectedProject"
            class="appearance-none bg-gray-800 border border-gray-700 rounded-lg pl-3 pr-8 py-2.5 sm:py-2 min-h-[44px] sm:min-h-0 text-sm text-gray-200 focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none cursor-pointer min-w-[140px]"
          >
            <option v-for="p in projects" :key="p" :value="p === 'All Projects' ? '' : p">{{ p }}</option>
          </select>
          <svg class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </div>
        
        <!-- Agent Filter -->
        <div class="relative">
          <label for="agent-filter" class="sr-only">Filter by agent</label>
          <select
            id="agent-filter"
            v-model="selectedAgent"
            class="appearance-none bg-gray-800 border border-gray-700 rounded-lg pl-3 pr-8 py-2.5 sm:py-2 min-h-[44px] sm:min-h-0 text-sm text-gray-200 focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none cursor-pointer min-w-[140px]"
          >
            <option v-for="a in agents" :key="a" :value="a === 'All Agents' ? '' : a">{{ a }}</option>
          </select>
          <svg class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </div>
        
        <!-- Level Filter -->
        <div class="relative">
          <label for="level-filter" class="sr-only">Filter by level</label>
          <select
            id="level-filter"
            v-model="selectedLevel"
            class="appearance-none bg-gray-800 border border-gray-700 rounded-lg pl-3 pr-8 py-2.5 sm:py-2 min-h-[44px] sm:min-h-0 text-sm text-gray-200 focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none cursor-pointer min-w-[120px]"
          >
            <option v-for="l in levels" :key="l" :value="l === 'All Levels' ? '' : l">{{ l }}</option>
          </select>
          <svg class="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500 pointer-events-none" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
          </svg>
        </div>
        
        <!-- Date Range -->
        <div class="flex flex-wrap items-center gap-2">
          <label for="date-from" class="sr-only">From date</label>
          <input
            id="date-from"
            v-model="dateFrom"
            type="date"
            class="bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 sm:py-2 min-h-[44px] sm:min-h-0 text-sm text-gray-200 focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none"
          >
          <span class="text-gray-500 hidden sm:inline" aria-hidden="true">—</span>
          <label for="date-to" class="sr-only">To date</label>
          <input
            id="date-to"
            v-model="dateTo"
            type="date"
            class="bg-gray-800 border border-gray-700 rounded-lg px-3 py-2.5 sm:py-2 min-h-[44px] sm:min-h-0 text-sm text-gray-200 focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 transition-colors motion-reduce:transition-none"
          >
        </div>
        
        <!-- Clear Filters -->
        <button
          v-if="selectedProject || selectedAgent || selectedLevel || dateFrom || dateTo"
          @click="clearFilters"
          class="text-sm text-gray-400 hover:text-white transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded px-3 py-2.5 sm:px-2 sm:py-1 min-h-[44px] sm:min-h-0"
          aria-label="Clear all filters"
        >
          Clear filters
        </button>
        
        <!-- Log Count -->
        <span class="w-full sm:w-auto sm:ml-auto text-sm text-gray-500" aria-live="polite">
          {{ filteredLogs.length }} {{ filteredLogs.length === 1 ? 'entry' : 'entries' }}
        </span>
      </div>
    </div>
    
    <!-- Log Stream -->
    <div 
      ref="logContainer"
      class="flex-1 overflow-y-auto bg-gray-950"
      @scroll="handleScroll"
      role="log"
      aria-label="Agent activity log stream"
      aria-live="polite"
    >
      <!-- Virtualized Container -->
      <div 
        class="relative"
        :style="{ height: `${totalHeight}px` }"
      >
        <div 
          class="absolute left-0 right-0"
          :style="{ transform: `translateY(${offsetY}px)` }"
        >
          <div 
            v-for="log in visibleLogs" 
            :key="log.id"
            class="border-b border-gray-800/50 hover:bg-gray-900/50 transition-colors motion-reduce:transition-none"
            :class="{ 'bg-gray-900/30': expandedLogId === log.id }"
          >
            <!-- Log Entry Row -->
            <button
              @click="toggleExpand(log.id)"
              class="w-full text-left px-4 sm:px-6 py-3 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-cyan-400"
              :aria-expanded="expandedLogId === log.id"
              :aria-controls="`log-detail-${log.id}`"
            >
              <div class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
                <!-- Timestamp -->
                <div class="flex items-center gap-2 sm:w-28 shrink-0">
                  <time class="text-xs text-gray-500 font-mono" :datetime="log.timestamp">
                    <span class="hidden sm:inline">{{ formatDate(log.timestamp) }} </span>{{ formatTimestamp(log.timestamp) }}
                  </time>
                </div>
                
                <!-- Level Badge -->
                <span 
                  class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium border shrink-0 w-fit"
                  :class="levelBadgeColors[log.level]"
                >
                  {{ log.level }}
                </span>
                
                <!-- Agent Badge -->
                <span 
                  class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded text-xs font-medium border shrink-0 w-fit"
                  :class="agentBadgeColors[log.agent] || 'bg-gray-500/20 text-gray-400 border-gray-500/30'"
                >
                  <span 
                    class="w-1.5 h-1.5 rounded-full"
                    :class="agentColors[log.agent] || 'bg-gray-500'"
                    aria-hidden="true"
                  />
                  {{ log.agent }}
                </span>
                
                <!-- Action -->
                <span 
                  class="flex-1 text-sm truncate"
                  :class="levelColors[log.level]"
                >
                  {{ log.action }}
                </span>
                
                <!-- Project -->
                <span class="text-xs text-gray-500 shrink-0 hidden sm:block">
                  {{ log.project }}
                </span>
                
                <!-- Expand Icon -->
                <svg 
                  class="w-4 h-4 text-gray-500 transition-transform duration-200 motion-reduce:transition-none shrink-0 hidden sm:block"
                  :class="{ 'rotate-180': expandedLogId === log.id }"
                  fill="none" 
                  stroke="currentColor" 
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
                </svg>
              </div>
            </button>
            
            <!-- Expanded Detail -->
            <div 
              v-if="expandedLogId === log.id"
              :id="`log-detail-${log.id}`"
              class="px-4 sm:px-6 pb-4"
            >
              <div class="bg-gray-900 rounded-lg border border-gray-800 p-4 space-y-3">
                <!-- Metadata Row -->
                <div class="flex flex-wrap items-center gap-4 text-xs text-gray-400">
                  <div>
                    <span class="text-gray-500">Project:</span>
                    <span class="ml-1 text-gray-300">{{ log.project }}</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Agent:</span>
                    <span class="ml-1 text-gray-300">{{ log.agent }}</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Level:</span>
                    <span class="ml-1" :class="levelColors[log.level]">{{ log.level }}</span>
                  </div>
                  <div>
                    <span class="text-gray-500">Time:</span>
                    <span class="ml-1 text-gray-300">{{ log.timestamp }}</span>
                  </div>
                </div>
                
                <!-- Context -->
                <div v-if="log.context">
                  <div class="text-xs text-gray-500 mb-1">Context</div>
                  <pre class="bg-gray-950 rounded p-3 text-xs text-gray-300 overflow-x-auto font-mono">{{ log.context }}</pre>
                </div>
                
                <!-- Stack Trace -->
                <div v-if="log.stackTrace">
                  <div class="text-xs text-red-400 mb-1">Stack Trace</div>
                  <pre class="bg-red-950/30 rounded p-3 text-xs text-red-300 overflow-x-auto font-mono border border-red-900/50">{{ log.stackTrace }}</pre>
                </div>
                
                <!-- Actions -->
                <div class="flex justify-end pt-2">
                  <button
                    @click.stop="copyLog(log)"
                    class="flex items-center gap-1.5 px-3 py-1.5 text-xs text-gray-400 hover:text-white bg-gray-800 hover:bg-gray-700 rounded transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                    aria-label="Copy log entry to clipboard"
                  >
                    <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                    </svg>
                    Copy
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      
      <!-- Empty State -->
      <div 
        v-if="filteredLogs.length === 0" 
        class="flex items-center justify-center h-full text-gray-500"
        role="status"
      >
        <div class="text-center py-16">
          <svg class="w-16 h-16 mx-auto mb-4 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p class="text-lg font-medium text-gray-400">No logs found</p>
          <p class="text-sm mt-1">Try adjusting your filters or wait for new activity</p>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* Custom scrollbar for log container */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: theme('colors.gray.900');
}

::-webkit-scrollbar-thumb {
  background: theme('colors.gray.700');
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: theme('colors.gray.600');
}

/* Date input styling for dark mode */
input[type="date"]::-webkit-calendar-picker-indicator {
  filter: invert(0.7);
  cursor: pointer;
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