<script setup lang="ts">
/**
 * TerminalDemo - Animated terminal showing Cuttlefish in action
 * CSS-based typing animation that loops every 15 seconds
 */

interface TerminalLine {
  type: 'command' | 'output' | 'success' | 'agent'
  content: string
  delay: number
}

const terminalLines: TerminalLine[] = [
  { type: 'command', content: '$ cuttlefish init my-project', delay: 0 },
  { type: 'output', content: '🐙 Initializing project...', delay: 600 },
  { type: 'success', content: '✓ Created my-project/', delay: 1200 },
  { type: 'command', content: '$ cuttlefish run "Add dark mode"', delay: 2000 },
  { type: 'agent', content: '🧠 Orchestrator: Analyzing task...', delay: 2800 },
  { type: 'agent', content: '📋 Planner: Creating implementation plan...', delay: 3600 },
  { type: 'agent', content: '👨‍💻 Coder: Writing CSS variables...', delay: 4800 },
  { type: 'agent', content: '🔍 Critic: Reviewing changes...', delay: 6000 },
  { type: 'success', content: '✓ Dark mode added! 3 files changed.', delay: 7500 },
]

const visibleLines = ref<TerminalLine[]>([])
const currentLineIndex = ref(0)
const isTyping = ref(false)
const cursorVisible = ref(true)

// Animation timing
const TYPING_SPEED = 40 // ms per character
const LOOP_INTERVAL = 15000 // 15 seconds

const animateTerminal = async () => {
  visibleLines.value = []
  currentLineIndex.value = 0
  
  for (const line of terminalLines) {
    // Wait for the line's delay
    await new Promise(resolve => setTimeout(resolve, line.delay - (terminalLines.indexOf(line) > 0 ? terminalLines[terminalLines.indexOf(line) - 1].delay : 0)))
    
    isTyping.value = true
    visibleLines.value.push(line)
    
    // Simulate typing for commands
    if (line.type === 'command') {
      await new Promise(resolve => setTimeout(resolve, line.content.length * TYPING_SPEED))
    } else {
      await new Promise(resolve => setTimeout(resolve, 200))
    }
    
    isTyping.value = false
  }
}

// Cursor blink
let cursorInterval: ReturnType<typeof setInterval> | null = null

onMounted(() => {
  animateTerminal()
  // Loop the animation
  setInterval(animateTerminal, LOOP_INTERVAL)
  
  // Cursor blink
  cursorInterval = setInterval(() => {
    cursorVisible.value = !cursorVisible.value
  }, 530)
})

onUnmounted(() => {
  if (cursorInterval) clearInterval(cursorInterval)
})
</script>

<template>
  <div class="terminal-window rounded-xl overflow-hidden border border-slate-700/50 shadow-2xl shadow-cyan-500/10">
    <!-- Title bar with traffic lights -->
    <div class="flex items-center gap-2 px-4 py-3 bg-slate-800/80 border-b border-slate-700/50 backdrop-blur-sm">
      <!-- Traffic light buttons -->
      <div class="flex items-center gap-2">
        <span class="w-3 h-3 rounded-full bg-red-500/80 hover:bg-red-400 transition-colors cursor-pointer" />
        <span class="w-3 h-3 rounded-full bg-yellow-500/80 hover:bg-yellow-400 transition-colors cursor-pointer" />
        <span class="w-3 h-3 rounded-full bg-green-500/80 hover:bg-green-400 transition-colors cursor-pointer" />
      </div>
      
      <!-- Title -->
      <span class="ml-4 text-sm text-slate-400 font-mono">
        cuttlefish@dev:~
      </span>
      
      <!-- Status indicator -->
      <span 
        class="ml-auto text-xs px-2 py-0.5 rounded-full transition-all duration-300"
        :class="isTyping ? 'bg-cyan-500/20 text-cyan-400' : 'bg-slate-700/50 text-slate-500'"
      >
        {{ isTyping ? '● running' : '● idle' }}
      </span>
    </div>
    
    <!-- Terminal content -->
    <div class="bg-slate-900/95 p-5 min-h-[280px] font-mono text-sm leading-relaxed">
      <TransitionGroup name="terminal-line">
        <div 
          v-for="(line, index) in visibleLines" 
          :key="index"
          class="terminal-line mb-1"
          :class="{
            'text-cyan-400': line.type === 'command',
            'text-slate-400': line.type === 'output',
            'text-emerald-400': line.type === 'success',
            'text-purple-400': line.type === 'agent'
          }"
        >
          {{ line.content }}
        </div>
      </TransitionGroup>
      
      <!-- Cursor -->
      <span 
        class="inline-block w-2 h-5 bg-cyan-400 ml-1 align-middle transition-opacity duration-100"
        :class="cursorVisible ? 'opacity-100' : 'opacity-0'"
      />
    </div>
  </div>
</template>

<style scoped>
.terminal-window {
  font-family: 'JetBrains Mono', 'Fira Code', 'SF Mono', monospace;
}

.terminal-line-enter-active {
  animation: fadeInUp 0.3s ease-out;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(8px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

/* Subtle scan line effect */
.terminal-window::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: repeating-linear-gradient(
    0deg,
    transparent,
    transparent 2px,
    rgba(0, 0, 0, 0.03) 2px,
    rgba(0, 0, 0, 0.03) 4px
  );
  pointer-events: none;
}
</style>