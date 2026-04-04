<template>
  <div class="relative group" role="region" aria-label="Code example">
    <div class="bg-slate-950 rounded-lg p-4 font-mono text-sm overflow-x-auto border border-slate-800">
      <pre class="text-slate-300"><code><template v-for="(line, i) in lines" :key="i">{{ line }}<br v-if="i < lines.length - 1"></template></code></pre>
    </div>
    <button
      @click="copyToClipboard"
      class="absolute top-2 right-2 p-2 rounded bg-slate-800 hover:bg-slate-700 text-slate-400 hover:text-white transition opacity-0 group-hover:opacity-100 focus:opacity-100 focus:outline-none focus:ring-2 focus:ring-cyan-400"
      :aria-label="copied ? 'Copied!' : 'Copy code'"
      :title="copied ? 'Copied!' : 'Copy code'"
    >
      <svg v-if="!copied" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
      </svg>
      <svg v-else class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
      </svg>
    </button>
  </div>
</template>

<script setup lang="ts">
const props = defineProps<{
  lines: string[]
  language?: string
}>()

const copied = ref(false)

const copyToClipboard = async () => {
  const code = props.lines.join('\n')
  try {
    await navigator.clipboard.writeText(code)
    copied.value = true
    setTimeout(() => {
      copied.value = false
    }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}
</script>