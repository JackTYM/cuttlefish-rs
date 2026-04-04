<script setup lang="ts">
/**
 * DiffPreview - Rich diff preview with syntax highlighting
 * 
 * Displays unified diff format with:
 * - Green background for additions (+)
 * - Red background for deletions (-)
 * - Line numbers for old/new content
 * - Collapsible hunks for large diffs
 * - Stats summary (X added, Y removed)
 */
export interface DiffLine {
  content: string
  isAddition: boolean
  isDeletion: boolean
  oldLine?: number
  newLine?: number
}

export interface DiffHunk {
  header: string
  oldStart: number
  oldLines: number
  newStart: number
  newLines: number
  lines: DiffLine[]
}

export interface DiffStats {
  linesAdded: number
  linesRemoved: number
  hunks: number
}

const props = withDefaults(defineProps<{
  /** The unified diff content */
  diff: string
  /** File path being diffed */
  filePath?: string
  /** Detected language for syntax highlighting hint */
  language?: string
  /** Whether this is a new file */
  isNewFile?: boolean
  /** Whether this is a deletion */
  isDeletion?: boolean
  /** Show split view instead of unified */
  splitView?: boolean
  /** Maximum visible lines before collapse */
  maxVisibleLines?: number
}>(), {
  splitView: false,
  maxVisibleLines: 500,
})

const expandedHunks = ref<Set<number>>(new Set([0])) // First hunk expanded by default
const showAllHunks = ref(false)

// Parse unified diff into structured data
const parsedDiff = computed(() => {
  if (!props.diff) return { hunks: [], stats: { linesAdded: 0, linesRemoved: 0, hunks: 0 } }
  
  const lines = props.diff.split('\n')
  const hunks: DiffHunk[] = []
  let currentHunk: DiffHunk | null = null
  let oldLineNum = 0
  let newLineNum = 0
  let stats = { linesAdded: 0, linesRemoved: 0, hunks: 0 }
  
  for (const line of lines) {
    // Hunk header: @@ -oldStart,oldLines +newStart,newLines @@
    const hunkMatch = line.match(/^@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@/)
    if (hunkMatch) {
      if (currentHunk) hunks.push(currentHunk)
      currentHunk = {
        header: line,
        oldStart: parseInt(hunkMatch[1]),
        oldLines: parseInt(hunkMatch[2] || '1'),
        newStart: parseInt(hunkMatch[3]),
        newLines: parseInt(hunkMatch[4] || '1'),
        lines: [],
      }
      oldLineNum = currentHunk.oldStart
      newLineNum = currentHunk.newStart
      stats.hunks++
      continue
    }
    
    if (!currentHunk) continue
    
    // Skip file headers
    if (line.startsWith('---') || line.startsWith('+++') || line.startsWith('diff --git') || line.startsWith('index ')) {
      continue
    }
    
    if (line.startsWith('+')) {
      currentHunk.lines.push({
        content: line.slice(1),
        isAddition: true,
        isDeletion: false,
        newLine: newLineNum++,
      })
      stats.linesAdded++
    } else if (line.startsWith('-')) {
      currentHunk.lines.push({
        content: line.slice(1),
        isAddition: false,
        isDeletion: true,
        oldLine: oldLineNum++,
      })
      stats.linesRemoved++
    } else if (line.startsWith(' ')) {
      currentHunk.lines.push({
        content: line.slice(1),
        isAddition: false,
        isDeletion: false,
        oldLine: oldLineNum++,
        newLine: newLineNum++,
      })
    } else if (line === '\\ No newline at end of file') {
      // Skip this marker
    } else if (line.trim()) {
      // Context line without leading space (some diffs)
      currentHunk.lines.push({
        content: line,
        isAddition: false,
        isDeletion: false,
        oldLine: oldLineNum++,
        newLine: newLineNum++,
      })
    }
  }
  
  if (currentHunk) hunks.push(currentHunk)
  
  return { hunks, stats }
})

const visibleHunks = computed(() => {
  if (showAllHunks.value) return parsedDiff.value.hunks
  return parsedDiff.value.hunks.filter((_, i) => expandedHunks.value.has(i))
})

const toggleHunk = (index: number) => {
  const newSet = new Set(expandedHunks.value)
  if (newSet.has(index)) {
    newSet.delete(index)
  } else {
    newSet.add(index)
  }
  expandedHunks.value = newSet
}

const expandAllHunks = () => {
  showAllHunks.value = true
}

const collapseAllHunks = () => {
  showAllHunks.value = false
  expandedHunks.value = new Set([0])
}

// Syntax highlighting for common patterns
const highlightLine = (content: string, isAddition: boolean, isDeletion: boolean) => {
  // Basic syntax highlighting - keywords, strings, comments
  let highlighted = content
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
  
  // Keywords (Rust, TypeScript, JavaScript, Python)
  const keywords = /\b(fn|let|const|var|if|else|for|while|return|match|struct|enum|impl|trait|pub|mod|use|import|export|from|class|interface|type|async|await|function|def|self|Self|true|false|null|undefined|None|Some|Ok|Err)\b/g
  highlighted = highlighted.replace(keywords, '<span class="text-purple-400">$1</span>')
  
  // Strings
  highlighted = highlighted.replace(/(["'`])(?:(?!\1)[^\\]|\\.)*\1/g, '<span class="text-green-300">$&</span>')
  
  // Comments
  highlighted = highlighted.replace(/(\/\/.*$)/gm, '<span class="text-gray-500">$1</span>')
  highlighted = highlighted.replace(/(#.*$)/gm, '<span class="text-gray-500">$1</span>')
  
  // Numbers
  highlighted = highlighted.replace(/\b(\d+\.?\d*)\b/g, '<span class="text-amber-400">$1</span>')
  
  return highlighted
}

// Keyboard navigation
const handleKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    emit('close')
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

const emit = defineEmits<{
  close: []
}>()
</script>

<template>
  <div class="diff-preview flex flex-col h-full bg-gray-950">
    <!-- Header -->
    <div class="bg-gray-900 border-b border-gray-800 px-4 py-3 flex items-center justify-between shrink-0">
      <div class="flex items-center gap-3">
        <!-- File path -->
        <div v-if="filePath" class="flex items-center gap-2">
          <svg class="w-4 h-4 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <span class="text-sm font-mono text-gray-300">{{ filePath }}</span>
        </div>
        
        <!-- File status badges -->
        <span v-if="isNewFile" class="text-xs px-2 py-0.5 rounded-full bg-green-900/50 text-green-400 border border-green-700/50">
          New file
        </span>
        <span v-if="isDeletion" class="text-xs px-2 py-0.5 rounded-full bg-red-900/50 text-red-400 border border-red-700/50">
          Deleted
        </span>
        
        <!-- Stats -->
        <span v-if="parsedDiff.stats.hunks > 0" class="text-xs text-gray-500 flex items-center gap-2">
          <span class="text-green-400">+{{ parsedDiff.stats.linesAdded }}</span>
          <span class="text-gray-600">/</span>
          <span class="text-red-400">-{{ parsedDiff.stats.linesRemoved }}</span>
        </span>
      </div>
      
      <!-- View controls -->
      <div class="flex items-center gap-2">
        <button
          v-if="parsedDiff.hunks.length > 1"
          @click="showAllHunks ? collapseAllHunks() : expandAllHunks()"
          class="text-xs px-2 py-1 rounded transition-colors text-gray-500 hover:text-gray-300 hover:bg-gray-800"
        >
          {{ showAllHunks ? 'Collapse all' : 'Expand all' }}
        </button>
      </div>
    </div>
    
    <!-- Diff content -->
    <div class="flex-1 overflow-auto font-mono text-sm">
      <!-- Empty state -->
      <div v-if="!diff || parsedDiff.hunks.length === 0" class="flex items-center justify-center h-full text-gray-500">
        <div class="text-center py-12">
          <svg class="w-12 h-12 mx-auto mb-3 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <p>No changes to preview</p>
        </div>
      </div>
      
      <!-- Unified diff view -->
      <div v-else class="divide-y divide-gray-800">
        <div v-for="(hunk, hunkIdx) in parsedDiff.hunks" :key="hunkIdx">
          <!-- Hunk header -->
          <button
            @click="toggleHunk(hunkIdx)"
            class="w-full px-4 py-2 bg-gray-900 text-cyan-400 text-xs sticky top-0 border-b border-gray-800 flex items-center gap-2 hover:bg-gray-800 transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
          >
            <svg
              class="w-3 h-3 transition-transform"
              :class="expandedHunks.has(hunkIdx) || showAllHunks ? 'rotate-90' : ''"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
            <span class="font-mono">{{ hunk.header }}</span>
          </button>
          
          <!-- Hunk lines -->
          <div v-if="expandedHunks.has(hunkIdx) || showAllHunks">
            <div
              v-for="(line, lineIdx) in hunk.lines"
              :key="lineIdx"
              class="flex group"
            >
              <!-- Old line number -->
              <div
                class="w-12 shrink-0 text-right pr-3 py-0.5 text-xs select-none border-r border-gray-800"
                :class="{
                  'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                  'text-green-700 bg-green-950/30': line.isAddition,
                  'text-red-700 bg-red-950/30': line.isDeletion,
                }"
              >
                {{ line.oldLine ?? '' }}
              </div>
              
              <!-- New line number -->
              <div
                class="w-12 shrink-0 text-right pr-3 py-0.5 text-xs select-none"
                :class="{
                  'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                  'text-green-700 bg-green-950/30': line.isAddition,
                  'text-red-700 bg-red-950/30': line.isDeletion,
                }"
              >
                {{ line.newLine ?? '' }}
              </div>
              
              <!-- Diff indicator -->
              <div
                class="w-6 shrink-0 text-center py-0.5 text-xs font-bold"
                :class="{
                  'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                  'text-green-400 bg-green-950/50': line.isAddition,
                  'text-red-400 bg-red-950/50': line.isDeletion,
                }"
              >
                {{ line.isAddition ? '+' : line.isDeletion ? '-' : ' ' }}
              </div>
              
              <!-- Line content -->
              <div
                class="flex-1 px-3 py-0.5 whitespace-pre overflow-x-auto"
                :class="{
                  'text-gray-300': !line.isAddition && !line.isDeletion,
                  'text-green-300 bg-green-950/20': line.isAddition,
                  'text-red-300 bg-red-950/20': line.isDeletion,
                }"
              >
                <span v-html="highlightLine(line.content, line.isAddition, line.isDeletion)" />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.diff-preview {
  font-family: 'JetBrains Mono', ui-monospace, monospace;
}

/* Smooth transitions for hunk expansion */
.diff-preview > div > div {
  transition: max-height 0.2s ease-out;
}

/* Ensure code doesn't wrap unexpectedly */
.diff-preview pre,
.diff-preview code {
  white-space: pre;
  overflow-x: auto;
}

/* Focus styles for keyboard navigation */
.diff-preview button:focus-visible {
  outline: 2px solid theme('colors.cyan.400');
  outline-offset: 2px;
}

/* Reduced motion support */
@media (prefers-reduced-motion: reduce) {
  .diff-preview * {
    transition-duration: 0.01ms !important;
  }
}
</style>