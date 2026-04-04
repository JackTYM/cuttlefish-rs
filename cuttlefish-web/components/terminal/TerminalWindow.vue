<script setup lang="ts">
/**
 * TerminalWindow - Terminal-style window chrome with optional typing animation
 * Wraps content in a macOS-style terminal window frame
 */
const props = withDefaults(defineProps<{
  /** Window title displayed in the title bar */
  title?: string
  /** Enable character-by-character typing animation for slot content */
  animated?: boolean
  /** Typing speed in milliseconds per character */
  typingSpeed?: number
}>(), {
  animated: false,
  typingSpeed: 30,
})

const slots = useSlots()
const displayedContent = ref('')
const isTyping = ref(false)

// Get text content from slot for animation
const getSlotTextContent = (): string => {
  if (!slots.default) return ''
  // Create a temporary div to extract text
  const tempDiv = document.createElement('div')
  const vnode = slots.default()
  // Render vnode to string (simplified approach)
  if (typeof vnode[0]?.children === 'string') {
    return vnode[0].children
  }
  return ''
}

const contentToDisplay = computed(() => {
  if (!props.animated) {
    return null // Return null to indicate we should use the slot directly
  }
  return displayedContent.value
})

// Start typing animation when component mounts or animated changes
watch(() => props.animated, async (newVal) => {
  if (newVal && slots.default) {
    const fullText = getSlotTextContent()
    displayedContent.value = ''
    isTyping.value = true
    
    for (let i = 0; i < fullText.length; i++) {
      await new Promise(resolve => setTimeout(resolve, props.typingSpeed))
      displayedContent.value += fullText[i]
    }
    
    isTyping.value = false
  }
}, { immediate: true })
</script>

<template>
  <div class="rounded-xl overflow-hidden border border-gray-700 shadow-2xl shadow-black/50" role="region" :aria-label="title ? `Terminal: ${title}` : 'Terminal window'">
    <!-- Title bar with traffic lights -->
    <div class="flex items-center gap-2 px-4 py-3 bg-gray-800 border-b border-gray-700" aria-hidden="true">
      <!-- Traffic light buttons -->
      <div class="flex items-center gap-2">
        <span class="w-3 h-3 rounded-full bg-red-500" />
        <span class="w-3 h-3 rounded-full bg-yellow-500" />
        <span class="w-3 h-3 rounded-full bg-green-500" />
      </div>
      
      <!-- Optional title -->
      <span 
        v-if="title" 
        class="ml-4 text-sm text-gray-400 font-medium"
        style="font-family: 'JetBrains Mono', monospace;"
      >
        {{ title }}
      </span>
      
      <!-- Typing indicator -->
      <span 
        v-if="animated && isTyping" 
        class="ml-auto text-xs text-cyan-400 animate-pulse motion-reduce:animate-none"
      >
        typing...
      </span>
    </div>
    
    <!-- Content area -->
    <div class="bg-gray-900 p-4 min-h-[100px]" style="font-family: 'JetBrains Mono', monospace;" :aria-live="animated ? 'polite' : undefined">
      <template v-if="animated">
        <pre class="text-gray-300 text-sm whitespace-pre-wrap">{{ contentToDisplay }}<span 
          v-if="isTyping" 
          class="inline-block w-2 h-4 bg-cyan-400 animate-pulse motion-reduce:animate-none ml-0.5"
          aria-hidden="true"
        /></pre>
      </template>
      <template v-else>
        <slot />
      </template>
    </div>
  </div>
</template>