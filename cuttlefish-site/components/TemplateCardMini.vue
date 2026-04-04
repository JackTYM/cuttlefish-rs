<template>
  <article 
    class="group bg-slate-900 border border-slate-800 rounded-lg p-4 hover:border-slate-700 transition-all duration-300 hover:shadow-lg hover:shadow-cyan-900/10 motion-reduce:transition-none cursor-pointer"
    role="listitem"
    tabindex="0"
    @click="navigateToTemplate"
    @keydown.enter="navigateToTemplate"
    @keydown.space.prevent="navigateToTemplate"
  >
    <!-- Header -->
    <div class="flex items-center gap-3 mb-2">
      <span class="text-2xl" aria-hidden="true">{{ template.icon }}</span>
      <div class="flex-1 min-w-0">
        <h3 class="text-sm font-semibold text-white truncate group-hover:text-cyan-400 transition-colors motion-reduce:transition-none">
          {{ template.name }}
        </h3>
        <p class="text-xs text-slate-500">
          {{ template.language }}
        </p>
      </div>
    </div>

    <!-- Stats -->
    <div class="flex items-center gap-3 text-xs text-slate-500">
      <span class="flex items-center gap-1">
        <span aria-hidden="true">⭐</span>
        {{ template.stars }}
      </span>
      <span class="flex items-center gap-1">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
        </svg>
        {{ template.downloads }}
      </span>
    </div>

    <!-- Hover indicator -->
    <div class="mt-3 pt-3 border-t border-slate-800 opacity-0 group-hover:opacity-100 transition-opacity motion-reduce:transition-none">
      <span class="text-xs text-cyan-400 flex items-center gap-1">
        Use template
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
        </svg>
      </span>
    </div>
  </article>
</template>

<script setup lang="ts">
interface Template {
  id: string
  name: string
  description: string
  category: string
  language: string
  author: string
  authorUrl: string
  stars: number
  downloads: number
  tags: string[]
  icon: string
  featured: boolean
  previewUrl: string
}

const props = defineProps<{
  template: Template
}>()

const navigateToTemplate = () => {
  window.open(`https://app.cuttlefish.dev/new?template=${props.template.name}`, '_blank')
}
</script>

<style scoped>
/* Prefers reduced motion support */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }
}
</style>