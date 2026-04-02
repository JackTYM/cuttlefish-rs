<template>
  <div class="border border-slate-700 rounded-lg overflow-hidden">
    <button
      @click="isOpen = !isOpen"
      class="w-full flex items-center justify-between p-4 bg-slate-900 hover:bg-slate-800 transition-colors text-left"
    >
      <span class="font-medium text-white">{{ question }}</span>
      <svg
        class="w-5 h-5 text-slate-400 transition-transform duration-200"
        :class="{ 'rotate-180': isOpen }"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
      </svg>
    </button>
    <div
      class="overflow-hidden transition-all duration-200"
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

const isOpen = ref(false)
</script>