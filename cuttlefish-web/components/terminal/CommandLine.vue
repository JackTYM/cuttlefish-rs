<script setup lang="ts">
/**
 * CommandLine - Terminal-style command display with copy functionality
 * Shows a command with $ prefix and optional output below
 */
const props = defineProps<{
  /** The command to display (without $ prefix) */
  command: string
  /** Optional output to show below the command */
  output?: string
}>()

const copied = ref(false)

const copyCommand = async () => {
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

<template>
  <div class="group inline-flex flex-col rounded-lg bg-gray-900/50 border border-gray-800 overflow-hidden">
    <!-- Command line -->
    <div 
      class="flex items-center gap-2 px-3 py-2 cursor-pointer hover:bg-gray-800/50 transition-colors"
      @click="copyCommand"
      title="Click to copy"
    >
      <span class="text-cyan-400 font-semibold select-none">$</span>
      <code class="text-gray-200 text-sm" style="font-family: 'JetBrains Mono', monospace;">{{ command }}</code>
      <span 
        class="ml-auto text-xs px-1.5 py-0.5 rounded transition-all duration-200 shrink-0"
        :class="copied 
          ? 'bg-green-900/50 text-green-400' 
          : 'bg-gray-800 text-gray-500 opacity-0 group-hover:opacity-100'"
      >
        {{ copied ? '✓' : 'Copy' }}
      </span>
    </div>
    
    <!-- Optional output -->
    <div 
      v-if="output" 
      class="px-3 py-2 border-t border-gray-800 bg-black/20"
    >
      <pre class="text-gray-500 text-sm whitespace-pre-wrap" style="font-family: 'JetBrains Mono', monospace;">{{ output }}</pre>
    </div>
  </div>
</template>