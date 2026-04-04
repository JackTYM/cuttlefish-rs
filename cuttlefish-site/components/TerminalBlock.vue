<template>
  <div class="relative group" role="region" :aria-label="comment || 'Terminal command'">
    <!-- Terminal Window -->
    <div class="bg-slate-950 border border-slate-700 rounded-lg overflow-hidden">
      <!-- Terminal Header -->
      <div class="flex items-center gap-2 px-4 py-2 bg-slate-900 border-b border-slate-700" aria-hidden="true">
        <div class="flex gap-1.5">
          <div class="w-3 h-3 rounded-full bg-red-500/80"></div>
          <div class="w-3 h-3 rounded-full bg-yellow-500/80"></div>
          <div class="w-3 h-3 rounded-full bg-green-500/80"></div>
        </div>
        <span class="text-xs text-slate-500 ml-2">bash</span>
      </div>
      
      <!-- Terminal Content -->
      <div class="p-4 font-mono text-sm">
        <!-- Comment line -->
        <div v-if="comment" class="text-slate-500 mb-1">
          <span class="text-slate-600" aria-hidden="true"># </span><span class="sr-only">Comment: </span>{{ comment }}
        </div>
        
        <!-- Command line -->
        <div class="flex items-start gap-2">
          <span class="text-green-400 select-none" aria-hidden="true">$</span>
          <code class="flex-1 text-slate-200 break-all">{{ displayCommand }}</code>
        </div>
      </div>
    </div>
    
    <!-- Copy Button -->
    <button
      @click="copyToClipboard"
      class="absolute top-10 right-2 p-2 rounded bg-slate-800 hover:bg-slate-700 border border-slate-600 opacity-0 group-hover:opacity-100 focus:opacity-100 transition-all motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
      :aria-label="copied ? 'Copied to clipboard' : 'Copy command to clipboard'"
    >
      <svg v-if="!copied" class="w-4 h-4 text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
      </svg>
      <svg v-else class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

interface Props {
  command: string
  comment?: string
  multiline?: boolean
}

const props = defineProps<Props>()

const copied = ref(false)

const displayCommand = computed(() => {
  if (props.multiline) {
    return props.command
  }
  return props.command
})

const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(props.command)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}
</script>

<style scoped>
/* Smooth transitions */
button {
  transition: opacity 0.2s ease, background-color 0.2s ease;
}
</style>