<script setup lang="ts">
import type { ProviderUsage, ModelUsage } from '~/composables/useUsageApi'

const props = defineProps<{
  providers: ProviderUsage[]
  loading?: boolean
}>()

const expandedProviders = ref<Set<string>>(new Set())

const toggleProvider = (provider: string) => {
  if (expandedProviders.value.has(provider)) {
    expandedProviders.value.delete(provider)
  } else {
    expandedProviders.value.add(provider)
  }
}

const formatCost = (cost: number): string => {
  if (cost >= 1) return `$${cost.toFixed(2)}`
  if (cost >= 0.01) return `$${cost.toFixed(3)}`
  return `$${cost.toFixed(4)}`
}

const formatTokens = (tokens: number): string => {
  if (tokens >= 1_000_000) return `${(tokens / 1_000_000).toFixed(2)}M`
  if (tokens >= 1_000) return `${(tokens / 1_000).toFixed(1)}K`
  return tokens.toString()
}

const allModels = computed(() => {
  const models: (ModelUsage & { provider: string })[] = []
  for (const provider of props.providers) {
    if (provider.models) {
      for (const model of provider.models) {
        models.push({ ...model, provider: provider.provider })
      }
    }
  }
  return models.sort((a, b) => b.estimated_cost - a.estimated_cost)
})
</script>

<template>
  <div class="bg-gray-900 border border-gray-800 rounded-xl overflow-hidden">
    <div class="p-5 border-b border-gray-800">
      <h3 class="text-lg font-semibold text-white">Model Usage</h3>
      <p class="text-sm text-gray-500 mt-1">Breakdown by model with input/output tokens</p>
    </div>
    
    <div v-if="loading" class="p-8 flex items-center justify-center">
      <div class="w-10 h-10 border-2 border-cyan-500/30 border-t-cyan-500 rounded-full animate-spin" />
    </div>
    
    <div v-else-if="providers.length === 0" class="p-8 text-center text-gray-500">
      <svg class="w-12 h-12 mx-auto mb-2 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 17v-2m3 2v-4m3 4v-6m2 10H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
      </svg>
      <p>No model usage data</p>
    </div>
    
    <div v-else class="overflow-x-auto">
      <table class="w-full">
        <thead class="bg-gray-800/50">
          <tr>
            <th class="px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider">Model</th>
            <th class="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">Input</th>
            <th class="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">Output</th>
            <th class="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">Requests</th>
            <th class="px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase tracking-wider">Cost</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-800">
          <template v-for="provider in providers" :key="provider.provider">
            <!-- Provider Row -->
            <tr 
              class="hover:bg-gray-800/30 cursor-pointer transition-colors"
              @click="toggleProvider(provider.provider)"
            >
              <td class="px-4 py-3">
                <div class="flex items-center gap-2">
                  <svg 
                    class="w-4 h-4 text-gray-500 transition-transform"
                    :class="{ 'rotate-90': expandedProviders.has(provider.provider) }"
                    fill="none" 
                    stroke="currentColor" 
                    viewBox="0 0 24 24"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                  <span class="font-medium text-white">{{ provider.provider }}</span>
                  <span class="text-xs text-gray-500">({{ provider.models?.length ?? 0 }} models)</span>
                </div>
              </td>
              <td class="px-4 py-3 text-right font-mono text-sm text-purple-400">{{ formatTokens(provider.input_tokens) }}</td>
              <td class="px-4 py-3 text-right font-mono text-sm text-green-400">{{ formatTokens(provider.output_tokens) }}</td>
              <td class="px-4 py-3 text-right font-mono text-sm text-gray-300">{{ provider.request_count.toLocaleString() }}</td>
              <td class="px-4 py-3 text-right font-mono text-sm text-cyan-400">{{ formatCost(provider.estimated_cost) }}</td>
            </tr>
            
            <!-- Model Rows (expanded) -->
            <template v-if="expandedProviders.has(provider.provider) && provider.models">
              <tr 
                v-for="model in provider.models" 
                :key="model.model"
                class="bg-gray-800/20 hover:bg-gray-800/40 transition-colors"
              >
                <td class="px-4 py-2 pl-10">
                  <span class="text-sm text-gray-300">{{ model.model }}</span>
                </td>
                <td class="px-4 py-2 text-right font-mono text-xs text-purple-400/80">{{ formatTokens(model.input_tokens) }}</td>
                <td class="px-4 py-2 text-right font-mono text-xs text-green-400/80">{{ formatTokens(model.output_tokens) }}</td>
                <td class="px-4 py-2 text-right font-mono text-xs text-gray-400">{{ model.request_count.toLocaleString() }}</td>
                <td class="px-4 py-2 text-right font-mono text-xs text-cyan-400/80">{{ formatCost(model.estimated_cost) }}</td>
              </tr>
            </template>
          </template>
        </tbody>
      </table>
    </div>
  </div>
</template>