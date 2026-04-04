<script setup lang="ts">
import type { UsageSummary } from '~/composables/useUsageApi'

const props = defineProps<{
  summary: UsageSummary | null
  loading?: boolean
}>()

const formatCost = (cost: number): string => {
  if (cost >= 1) return `$${cost.toFixed(2)}`
  if (cost >= 0.01) return `$${cost.toFixed(3)}`
  return `$${cost.toFixed(4)}`
}

const formatTokens = (tokens: number): string => {
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(1)}M`
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(1)}K`
  return tokens.toString()
}

const totalTokens = computed(() => 
  (props.summary?.total_input_tokens ?? 0) + (props.summary?.total_output_tokens ?? 0)
)
</script>

<template>
  <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
    <!-- Total Cost Card -->
    <div class="bg-gray-900 border border-gray-800 rounded-xl p-5">
      <div class="flex items-center gap-3 mb-2">
        <div class="w-10 h-10 rounded-lg bg-cyan-900/50 flex items-center justify-center">
          <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
        </div>
        <span class="text-sm text-gray-400">Total Cost</span>
      </div>
      <div v-if="loading" class="h-8 w-24 bg-gray-800 animate-pulse rounded" />
      <p v-else class="text-2xl font-bold text-white font-mono">
        {{ summary ? formatCost(summary.total_cost_usd) : '$0.00' }}
      </p>
      <p class="text-xs text-gray-500 mt-1">{{ summary?.period ?? 'monthly' }} period</p>
    </div>

    <!-- Total Tokens Card -->
    <div class="bg-gray-900 border border-gray-800 rounded-xl p-5">
      <div class="flex items-center gap-3 mb-2">
        <div class="w-10 h-10 rounded-lg bg-purple-900/50 flex items-center justify-center">
          <svg class="w-5 h-5 text-purple-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01" />
          </svg>
        </div>
        <span class="text-sm text-gray-400">Total Tokens</span>
      </div>
      <div v-if="loading" class="h-8 w-24 bg-gray-800 animate-pulse rounded" />
      <p v-else class="text-2xl font-bold text-white font-mono">
        {{ formatTokens(totalTokens) }}
      </p>
      <p class="text-xs text-gray-500 mt-1">
        {{ summary ? formatTokens(summary.total_input_tokens) : '0' }} in / {{ summary ? formatTokens(summary.total_output_tokens) : '0' }} out
      </p>
    </div>

    <!-- Requests Card -->
    <div class="bg-gray-900 border border-gray-800 rounded-xl p-5">
      <div class="flex items-center gap-3 mb-2">
        <div class="w-10 h-10 rounded-lg bg-green-900/50 flex items-center justify-center">
          <svg class="w-5 h-5 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
          </svg>
        </div>
        <span class="text-sm text-gray-400">Requests</span>
      </div>
      <div v-if="loading" class="h-8 w-24 bg-gray-800 animate-pulse rounded" />
      <p v-else class="text-2xl font-bold text-white font-mono">
        {{ summary?.total_requests?.toLocaleString() ?? '0' }}
      </p>
      <p class="text-xs text-gray-500 mt-1">API calls this period</p>
    </div>

    <!-- Providers Card -->
    <div class="bg-gray-900 border border-gray-800 rounded-xl p-5">
      <div class="flex items-center gap-3 mb-2">
        <div class="w-10 h-10 rounded-lg bg-yellow-900/50 flex items-center justify-center">
          <svg class="w-5 h-5 text-yellow-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
        </div>
        <span class="text-sm text-gray-400">Providers</span>
      </div>
      <div v-if="loading" class="h-8 w-24 bg-gray-800 animate-pulse rounded" />
      <p v-else class="text-2xl font-bold text-white font-mono">
        {{ summary ? Object.keys(summary.by_provider).length : 0 }}
      </p>
      <p class="text-xs text-gray-500 mt-1">Active providers</p>
    </div>
  </div>
</template>