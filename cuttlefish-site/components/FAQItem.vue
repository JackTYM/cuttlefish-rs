<template>
  <div class="border border-slate-700 rounded-lg overflow-hidden">
    <h3>
      <button
        @click="isOpen = !isOpen"
        class="w-full flex items-center justify-between p-4 bg-slate-900 hover:bg-slate-800 transition-colors motion-reduce:transition-none text-left focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-cyan-400"
        :aria-expanded="isOpen"
        :aria-controls="`faq-panel-${id}`"
        :id="`faq-button-${id}`"
      >
        <span class="font-medium text-white">{{ question }}</span>
        <svg
          class="w-5 h-5 text-slate-400 transition-transform duration-200 motion-reduce:transition-none"
          :class="{ 'rotate-180': isOpen }"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
        </svg>
      </button>
    </h3>
    <div
      :id="`faq-panel-${id}`"
      role="region"
      :aria-labelledby="`faq-button-${id}`"
      :aria-hidden="!isOpen"
      class="overflow-hidden transition-all duration-200 motion-reduce:transition-none"
      :class="isOpen ? 'max-h-96' : 'max-h-0'"
    >
      <div class="p-4 bg-slate-950 border-t border-slate-700 text-slate-300">
        {{ answer }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

interface Props {
  question: string
  answer: string
}

defineProps<Props>()

// Generate unique ID for ARIA relationships
const id = Math.random().toString(36).substring(2, 9)

const isOpen = ref(false)
</script>