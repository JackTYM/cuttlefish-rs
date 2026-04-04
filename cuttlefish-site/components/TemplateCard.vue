<template>
  <article 
    class="group relative bg-slate-900 border border-slate-800 rounded-xl overflow-hidden hover:border-slate-700 transition-all duration-300 hover:shadow-lg hover:shadow-cyan-900/10 motion-reduce:transition-none"
    role="listitem"
  >
    <!-- Featured badge -->
    <div 
      v-if="featured"
      class="absolute top-3 right-3 z-10"
    >
      <span class="inline-flex items-center gap-1 px-2 py-1 bg-gradient-to-r from-amber-500/20 to-orange-500/20 border border-amber-500/30 rounded text-xs font-medium text-amber-400">
        <span aria-hidden="true">⭐</span>
        Featured
      </span>
    </div>

    <!-- Preview area -->
    <div class="h-32 bg-gradient-to-br from-slate-800 to-slate-900 flex items-center justify-center border-b border-slate-800">
      <span class="text-5xl opacity-60 group-hover:scale-110 transition-transform duration-300 motion-reduce:transition-none" aria-hidden="true">
        {{ template.icon }}
      </span>
    </div>

    <!-- Content -->
    <div class="p-5">
      <!-- Header -->
      <div class="flex items-start justify-between gap-3 mb-2">
        <h3 class="text-lg font-semibold text-white group-hover:text-cyan-400 transition-colors motion-reduce:transition-none">
          {{ template.name }}
        </h3>
        <span class="shrink-0 text-xs px-2 py-1 bg-slate-800 text-slate-400 rounded font-mono">
          {{ template.language }}
        </span>
      </div>

      <!-- Description -->
      <p class="text-slate-400 text-sm mb-4 line-clamp-2">
        {{ template.description }}
      </p>

      <!-- Tags -->
      <div class="flex flex-wrap gap-1.5 mb-4">
        <span 
          v-for="tag in template.tags.slice(0, 3)" 
          :key="tag"
          class="text-xs px-2 py-0.5 bg-slate-800/50 text-slate-500 rounded"
        >
          {{ tag }}
        </span>
        <span 
          v-if="template.tags.length > 3"
          class="text-xs px-2 py-0.5 text-slate-600"
        >
          +{{ template.tags.length - 3 }}
        </span>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between pt-4 border-t border-slate-800">
        <!-- Author and stats -->
        <div class="flex items-center gap-3 text-sm text-slate-500">
          <a 
            :href="template.authorUrl"
            target="_blank"
            rel="noopener noreferrer"
            class="hover:text-slate-300 transition-colors motion-reduce:transition-none"
          >
            {{ template.author }}
          </a>
          <span class="flex items-center gap-1" :aria-label="template.stars + ' stars'">
            <span aria-hidden="true">⭐</span>
            <span>{{ formatNumber(template.stars) }}</span>
          </span>
          <span class="flex items-center gap-1" :aria-label="template.downloads + ' downloads'">
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            <span>{{ formatNumber(template.downloads) }}</span>
          </span>
        </div>

        <!-- Category badge -->
        <span 
          :class="[
            'text-xs px-2 py-1 rounded',
            template.category === 'official' 
              ? 'bg-cyan-900/30 text-cyan-400 border border-cyan-800/50' 
              : 'bg-purple-900/30 text-purple-400 border border-purple-800/50'
          ]"
        >
          {{ template.category === 'official' ? 'Official' : 'Community' }}
        </span>
      </div>

      <!-- Action buttons (shown on hover) -->
      <div class="absolute inset-x-0 bottom-0 p-4 bg-gradient-to-t from-slate-900 via-slate-900/95 to-transparent translate-y-full group-hover:translate-y-0 transition-transform duration-300 motion-reduce:transition-none">
        <div class="flex gap-2">
          <a
            :href="template.previewUrl"
            target="_blank"
            rel="noopener noreferrer"
            class="flex-1 inline-flex items-center justify-center gap-2 px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-300 text-sm font-medium rounded-lg transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
            </svg>
            View
            <span class="sr-only">(opens in new tab)</span>
          </a>
          <a
            :href="'https://app.cuttlefish.dev/new?template=' + template.name"
            class="flex-1 inline-flex items-center justify-center gap-2 px-4 py-2 bg-cyan-600 hover:bg-cyan-500 text-white text-sm font-medium rounded-lg transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Use Template
          </a>
        </div>
      </div>
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

defineProps<{
  template: Template
  featured?: boolean
}>()

// Format numbers with K suffix
const formatNumber = (num: number): string => {
  if (num >= 1000) {
    return (num / 1000).toFixed(1) + 'K'
  }
  return num.toString()
}
</script>

<style scoped>
/* Line clamp for description */
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

/* Screen reader only utility */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}

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