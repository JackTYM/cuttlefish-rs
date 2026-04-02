<template>
  <div class="bg-slate-950 rounded-lg p-4 font-mono text-sm overflow-x-auto border border-slate-800">
    <pre class="text-slate-300"><template v-for="(line, i) in lines" :key="i">{{ line }}<br v-if="i < lines.length - 1"></template></pre>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  lines: string[]
  language?: string
}>()
</script>