<template>
  <div class="bg-slate-900 border border-slate-800 rounded-xl p-6 hover:border-slate-700 hover:shadow-lg hover:shadow-cyan-900/10 transition-all duration-300 group">
    <div class="flex items-start gap-4 mb-4">
      <span class="text-4xl filter drop-shadow-lg group-hover:scale-110 transition-transform duration-300">{{ icon }}</span>
      <div>
        <h3 class="text-xl font-bold text-white mb-2">{{ title }}</h3>
        <ul class="space-y-1">
          <li v-for="line in description" :key="line" class="text-slate-400 text-sm">
            {{ line }}
          </li>
        </ul>
      </div>
    </div>
    <div class="mt-4 pt-4 border-t border-slate-800">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  icon: string
  title: string
  description: string[]
}>()
</script>