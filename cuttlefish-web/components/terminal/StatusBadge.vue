<script setup lang="ts">
/**
 * StatusBadge - Colored status indicator with optional pulsing animation
 * Displays status as a pill-shaped badge with semantic colors
 */
const props = withDefaults(defineProps<{
  /** Status type determining color */
  status: 'success' | 'warning' | 'error' | 'info' | 'pending'
  /** Optional custom label (defaults to status) */
  label?: string
  /** Enable pulsing animation (useful for pending states) */
  pulse?: boolean
}>(), {
  pulse: false,
})

const colorClasses = computed(() => {
  const colors = {
    success: 'bg-green-900/60 text-green-300 border-green-700/50 shadow-green-500/20',
    warning: 'bg-yellow-900/60 text-yellow-300 border-yellow-700/50 shadow-yellow-500/20',
    error: 'bg-red-900/60 text-red-300 border-red-700/50 shadow-red-500/20',
    info: 'bg-cyan-900/60 text-cyan-300 border-cyan-700/50 shadow-cyan-500/20',
    pending: 'bg-gray-800/60 text-gray-400 border-gray-700/50 shadow-gray-500/20',
  }
  return colors[props.status]
})

const dotColor = computed(() => {
  const colors = {
    success: 'bg-green-400',
    warning: 'bg-yellow-400',
    error: 'bg-red-400',
    info: 'bg-cyan-400',
    pending: 'bg-gray-500',
  }
  return colors[props.status]
})

const displayLabel = computed(() => props.label || props.status)
</script>

<template>
  <span 
    class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border shadow-sm"
    :class="colorClasses"
  >
    <span 
      class="w-1.5 h-1.5 rounded-full"
      :class="[dotColor, { 'animate-pulse': pulse }]"
    />
    {{ displayLabel }}
  </span>
</template>