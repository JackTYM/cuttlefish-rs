<script setup lang="ts">
import type { UsageSummary, DailyUsage, ProviderUsage, TimePeriod } from '~/composables/useUsageApi'

definePageMeta({
  layout: 'default',
})

const {
  loading,
  error,
  fetchUsageSummary,
  fetchDailyUsage,
  fetchProviderUsage,
  getExportUrl,
} = useUsageApi()

const selectedPeriod = ref<TimePeriod>('monthly')
const summary = ref<UsageSummary | null>(null)
const dailyData = ref<DailyUsage[]>([])
const providerData = ref<ProviderUsage[]>([])
const chartType = ref<'pie' | 'bar'>('pie')

const loadData = async () => {
  const [summaryResult, dailyResult, providerResult] = await Promise.all([
    fetchUsageSummary(selectedPeriod.value),
    fetchDailyUsage(selectedPeriod.value),
    fetchProviderUsage(selectedPeriod.value),
  ])
  
  if (summaryResult) summary.value = summaryResult
  if (dailyResult) dailyData.value = dailyResult
  if (providerResult) providerData.value = providerResult
}

watch(selectedPeriod, loadData)
onMounted(loadData)

const exportUrl = computed(() => getExportUrl(selectedPeriod.value))
</script>

<template>
  <div class="p-4 sm:p-6">
    <div class="max-w-7xl mx-auto">
      <!-- Header -->
      <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
        <div>
          <h1 class="text-2xl font-bold text-white">Usage Dashboard</h1>
          <p class="text-gray-400 text-sm mt-1">Track your API usage and costs</p>
        </div>
        
        <div class="flex flex-wrap items-center gap-3">
          <!-- Period Selector -->
          <div class="flex bg-gray-800 rounded-lg p-1 border border-gray-700">
            <button
              v-for="period in ['daily', 'weekly', 'monthly'] as TimePeriod[]"
              :key="period"
              @click="selectedPeriod = period"
              class="px-4 py-2 text-sm rounded-md transition-all duration-200"
              :class="selectedPeriod === period 
                ? 'bg-cyan-600 text-white shadow-sm' 
                : 'text-gray-400 hover:text-white'"
            >
              {{ period.charAt(0).toUpperCase() + period.slice(1) }}
            </button>
          </div>
          
          <!-- Export Button -->
          <a
            :href="exportUrl"
            download="usage.csv"
            class="flex items-center gap-2 px-4 py-2 bg-gray-800 hover:bg-gray-700 text-gray-300 hover:text-white border border-gray-700 rounded-lg text-sm transition-colors"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            Export CSV
          </a>
        </div>
      </div>
      
      <!-- Error State -->
      <div v-if="error" class="mb-6 p-4 bg-red-900/30 border border-red-700/50 rounded-lg text-red-400">
        <div class="flex items-center gap-2">
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{{ error }}</span>
        </div>
      </div>
      
      <!-- Summary Cards -->
      <UsageSummaryCards 
        :summary="summary" 
        :loading="loading" 
        class="mb-6"
      />
      
      <!-- Charts Row -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
        <!-- Provider Cost Chart -->
        <ProviderChart 
          :providers="providerData"
          :loading="loading"
          v-model:chartType="chartType"
        />
        
        <!-- Daily Trend Chart -->
        <DailyTrendChart 
          :daily-data="dailyData"
          :loading="loading"
        />
      </div>
      
      <!-- Model Usage Table -->
      <ModelUsageTable 
        :providers="providerData"
        :loading="loading"
      />
      
      <!-- Empty State -->
      <div 
        v-if="!loading && !summary?.total_requests" 
        class="mt-8 text-center py-12 bg-gray-900 border border-gray-800 rounded-xl"
      >
        <div class="text-5xl mb-4">📊</div>
        <h3 class="text-lg font-medium text-white mb-2">No usage data yet</h3>
        <p class="text-gray-400 text-sm max-w-md mx-auto">
          Start making API requests to see your usage statistics here. 
          Your token usage and costs will be tracked automatically.
        </p>
      </div>
    </div>
  </div>
</template>