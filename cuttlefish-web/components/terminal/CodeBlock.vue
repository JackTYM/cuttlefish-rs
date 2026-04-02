<script setup lang="ts">
/**
 * CodeBlock - Syntax highlighted code block with copy functionality
 * Displays code with optional line numbers and a copy button
 */
const props = withDefaults(defineProps<{
  /** The code content to display */
  code: string
  /** Programming language for syntax hint (display only) */
  language?: string
  /** Show line numbers on the left */
  showLineNumbers?: boolean
}>(), {
  showLineNumbers: false,
})

const copied = ref(false)

const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(props.code)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

const lines = computed(() => props.code.split('\n'))
const lineNumberWidth = computed(() => String(lines.value.length).length * 0.6 + 1)
</script>

<template>
  <div class="relative group rounded-lg bg-gray-900 border border-gray-800 overflow-hidden">
    <!-- Header with language badge and copy button -->
    <div class="flex items-center justify-between px-4 py-2 border-b border-gray-800 bg-gray-900/50">
      <span v-if="language" class="text-xs text-gray-500 font-mono uppercase tracking-wider">
        {{ language }}
      </span>
      <span v-else class="text-xs text-gray-600">code</span>
      
      <button
        @click="copyToClipboard"
        class="text-xs px-2 py-1 rounded transition-all duration-200"
        :class="copied 
          ? 'bg-green-900/50 text-green-400' 
          : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-gray-200'"
      >
        {{ copied ? '✓ Copied!' : 'Copy' }}
      </button>
    </div>
    
    <!-- Code content -->
    <div class="overflow-x-auto">
      <pre class="p-4 text-sm leading-relaxed" style="font-family: 'JetBrains Mono', monospace;"><code><div
        v-for="(line, i) in lines"
        :key="i"
        class="flex"
      ><span
          v-if="showLineNumbers"
          class="select-none text-gray-600 mr-4 text-right shrink-0"
          :style="{ width: `${lineNumberWidth}rem` }"
        >{{ i + 1 }}</span><span class="text-gray-300">{{ line }}</span></div></code></pre>
    </div>
  </div>
</template>