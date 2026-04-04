<script setup lang="ts">
/**
 * ApprovalModal - Modal for pending action approval with diff preview
 * 
 * Features:
 * - Shows action description, confidence score, risk factors
 * - Includes diff preview for file operations
 * - Approve/Reject buttons with keyboard shortcuts
 * - Countdown timer for timeout
 * - Real-time updates via WebSocket
 */
export interface RiskFactor {
  type: string
  description: string
}

export interface PendingAction {
  id: string
  projectId: string
  actionType: string
  description: string
  path?: string
  command?: string
  confidence: number
  confidenceReasoning: string
  riskFactors?: RiskFactor[]
  createdAt: string
  timeoutSecs: number
  hasDiff: boolean
}

const props = defineProps<{
  /** The pending action to display */
  action: PendingAction
  /** Whether the modal is visible */
  visible: boolean
  /** Loading state for approve/reject actions */
  loading?: boolean
}>()

const emit = defineEmits<{
  approve: [actionId: string]
  reject: [actionId: string]
  close: []
}>()

// Diff state
const diffContent = ref('')
const diffLoading = ref(false)
const diffError = ref<string | null>(null)

// Timer state
const timeRemaining = ref(0)
const timerInterval = ref<ReturnType<typeof setInterval> | null>(null)

// Fetch diff when action changes
watch(() => props.action, async (newAction) => {
  if (newAction?.hasDiff && newAction.id) {
    diffLoading.value = true
    diffError.value = null
    try {
      const config = useRuntimeConfig()
      const response = await fetch(`${config.public.apiBase}/api/actions/${newAction.id}/diff`, {
        headers: {
          'Authorization': `Bearer ${localStorage.getItem('apiKey') || ''}`,
        },
      })
      if (response.ok) {
        const data = await response.json()
        diffContent.value = data.unified_diff || ''
      } else {
        diffError.value = 'Failed to load diff preview'
      }
    } catch (err) {
      diffError.value = 'Failed to load diff preview'
    } finally {
      diffLoading.value = false
    }
  } else {
    diffContent.value = ''
  }
}, { immediate: true })

// Timer countdown
watch(() => props.visible, (isVisible) => {
  if (isVisible && props.action) {
    timeRemaining.value = props.action.timeoutSecs
    if (timerInterval.value) clearInterval(timerInterval.value)
    timerInterval.value = setInterval(() => {
      if (timeRemaining.value > 0) {
        timeRemaining.value--
      } else {
        // Timeout - auto reject
        emit('reject', props.action.id)
      }
    }, 1000)
  } else {
    if (timerInterval.value) {
      clearInterval(timerInterval.value)
      timerInterval.value = null
    }
  }
})

// Cleanup on unmount
onUnmounted(() => {
  if (timerInterval.value) {
    clearInterval(timerInterval.value)
  }
})

// Format time remaining
const formattedTime = computed(() => {
  const mins = Math.floor(timeRemaining.value / 60)
  const secs = timeRemaining.value % 60
  return `${mins}:${secs.toString().padStart(2, '0')}`
})

// Timer warning state
const timerWarning = computed(() => {
  return timeRemaining.value < 60 // Less than 1 minute
})

// Confidence level styling
const confidenceLevel = computed(() => {
  const conf = props.action?.confidence ?? 0
  if (conf >= 0.8) return { label: 'High', color: 'text-green-400', bg: 'bg-green-900/30' }
  if (conf >= 0.5) return { label: 'Medium', color: 'text-yellow-400', bg: 'bg-yellow-900/30' }
  return { label: 'Low', color: 'text-red-400', bg: 'bg-red-900/30' }
})

// Action type icon
const actionIcon = computed(() => {
  const type = props.action?.actionType?.toLowerCase() || ''
  if (type.includes('file') || type.includes('write')) {
    return `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />`
  }
  if (type.includes('bash') || type.includes('command')) {
    return `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 9l3 3-3 3m5 0h3M5 20h14a2 2 0 002-2V6a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />`
  }
  if (type.includes('git')) {
    return `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 20l4-16m4 4l4 4-4 4M6 16l-4-4 4-4" />`
  }
  return `<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />`
})

// Keyboard shortcuts
const handleKeydown = (event: KeyboardEvent) => {
  if (!props.visible) return
  
  if (event.key === 'Enter' && !event.shiftKey) {
    event.preventDefault()
    emit('approve', props.action.id)
  } else if (event.key === 'Escape') {
    event.preventDefault()
    emit('close')
  } else if (event.key === 'r' && event.shiftKey) {
    event.preventDefault()
    emit('reject', props.action.id)
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

const handleApprove = () => {
  emit('approve', props.action.id)
}

const handleReject = () => {
  emit('reject', props.action.id)
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div
        v-if="visible && action"
        class="fixed inset-0 z-50 flex items-center justify-center p-4"
        role="dialog"
        aria-modal="true"
        aria-labelledby="approval-modal-title"
      >
        <!-- Backdrop -->
        <div
          class="absolute inset-0 bg-black/70 backdrop-blur-sm"
          @click="emit('close')"
          aria-hidden="true"
        />
        
        <!-- Modal -->
        <div class="relative bg-gray-900 rounded-xl border border-gray-700 shadow-2xl max-w-4xl w-full max-h-[90vh] flex flex-col overflow-hidden">
          <!-- Header -->
          <div class="px-6 py-4 border-b border-gray-800 flex items-center justify-between shrink-0">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 rounded-full bg-amber-900/50 flex items-center justify-center">
                <svg class="w-5 h-5 text-amber-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" v-html="actionIcon" />
              </div>
              <div>
                <h3 id="approval-modal-title" class="text-lg font-semibold text-white">
                  Action Requires Approval
                </h3>
                <p class="text-sm text-gray-400">{{ action.actionType }}</p>
              </div>
            </div>
            
            <!-- Timer -->
            <div
              class="flex items-center gap-2 px-3 py-1.5 rounded-lg"
              :class="timerWarning ? 'bg-red-900/50 text-red-400' : 'bg-gray-800 text-gray-400'"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <span class="text-sm font-mono">{{ formattedTime }}</span>
            </div>
          </div>
          
          <!-- Body -->
          <div class="flex-1 overflow-y-auto p-6 space-y-4">
            <!-- Action description -->
            <div class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
              <div class="text-sm text-gray-400 mb-1">Description</div>
              <div class="text-white">{{ action.description }}</div>
            </div>
            
            <!-- Path or command -->
            <div v-if="action.path" class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
              <div class="text-sm text-gray-400 mb-1">File Path</div>
              <div class="font-mono text-cyan-400 text-sm">{{ action.path }}</div>
            </div>
            
            <div v-if="action.command" class="bg-gray-800/50 rounded-lg p-4 border border-gray-700">
              <div class="text-sm text-gray-400 mb-1">Command</div>
              <code class="font-mono text-amber-400 text-sm bg-gray-900 px-2 py-1 rounded">{{ action.command }}</code>
            </div>
            
            <!-- Confidence score -->
            <div class="flex items-center gap-4">
              <div class="flex-1">
                <div class="text-sm text-gray-400 mb-2">Confidence Score</div>
                <div class="flex items-center gap-3">
                  <div class="flex-1 h-2 bg-gray-800 rounded-full overflow-hidden">
                    <div
                      class="h-full transition-all duration-300"
                      :class="confidenceLevel.bg.replace('bg-', 'bg-').replace('/30', '')"
                      :style="{ width: `${(action.confidence ?? 0) * 100}%` }"
                    />
                  </div>
                  <span class="text-sm font-mono" :class="confidenceLevel.color">
                    {{ ((action.confidence ?? 0) * 100).toFixed(0) }}%
                  </span>
                </div>
                <div class="mt-1 text-xs text-gray-500">{{ action.confidenceReasoning }}</div>
              </div>
            </div>
            
            <!-- Risk factors -->
            <div v-if="action.riskFactors?.length" class="space-y-2">
              <div class="text-sm text-gray-400">Risk Factors</div>
              <div class="flex flex-wrap gap-2">
                <span
                  v-for="(risk, i) in action.riskFactors"
                  :key="i"
                  class="text-xs px-2 py-1 rounded-full bg-red-900/30 text-red-400 border border-red-700/50"
                >
                  {{ risk.description || risk.type }}
                </span>
              </div>
            </div>
            
            <!-- Diff preview -->
            <div v-if="action.hasDiff" class="space-y-2">
              <div class="text-sm text-gray-400">Changes Preview</div>
              <div class="rounded-lg border border-gray-700 overflow-hidden" style="max-height: 300px;">
                <div v-if="diffLoading" class="flex items-center justify-center py-8 text-gray-500">
                  <svg class="w-6 h-6 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                </div>
                <div v-else-if="diffError" class="flex items-center justify-center py-8 text-red-400">
                  {{ diffError }}
                </div>
                <DiffPreview
                  v-else
                  :diff="diffContent"
                  :file-path="action.path"
                  style="max-height: 300px;"
                />
              </div>
            </div>
          </div>
          
          <!-- Footer -->
          <div class="px-6 py-4 bg-gray-900/50 border-t border-gray-800 flex items-center justify-between shrink-0">
            <div class="text-xs text-gray-500">
              <kbd class="px-1.5 py-0.5 bg-gray-800 rounded text-gray-400">Enter</kbd> to approve
              <span class="mx-2">•</span>
              <kbd class="px-1.5 py-0.5 bg-gray-800 rounded text-gray-400">Esc</kbd> to close
              <span class="mx-2">•</span>
              <kbd class="px-1.5 py-0.5 bg-gray-800 rounded text-gray-400">Shift+R</kbd> to reject
            </div>
            <div class="flex items-center gap-3">
              <button
                @click="handleReject"
                :disabled="loading"
                class="px-4 py-2 text-sm rounded-lg transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                :class="loading
                  ? 'bg-gray-800 text-gray-500 cursor-not-allowed'
                  : 'bg-red-900/50 text-red-400 hover:bg-red-800/50 border border-red-700/50'"
              >
                Reject
              </button>
              <button
                @click="handleApprove"
                :disabled="loading"
                class="px-4 py-2 text-sm rounded-lg transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                :class="loading
                  ? 'bg-gray-800 text-gray-500 cursor-not-allowed'
                  : 'bg-green-600 hover:bg-green-500 text-white'"
              >
                <span v-if="loading" class="flex items-center gap-2">
                  <svg class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                  Processing...
                </span>
                <span v-else>Approve</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
/* Modal transition */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-active .relative,
.modal-leave-active .relative {
  transition: transform 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .relative,
.modal-leave-to .relative {
  transform: scale(0.95);
}

/* Keyboard shortcut styling */
kbd {
  font-family: 'JetBrains Mono', ui-monospace, monospace;
  font-size: 0.75rem;
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {
  .modal-enter-active,
  .modal-leave-active,
  .modal-enter-active .relative,
  .modal-leave-active .relative {
    transition-duration: 0.01ms !important;
  }
}
</style>