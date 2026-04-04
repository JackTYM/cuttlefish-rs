<script setup lang="ts">
import { Doughnut, Bar } from 'vue-chartjs'
import {
  Chart as ChartJS,
  ArcElement,
  CategoryScale,
  LinearScale,
  BarElement,
  Title,
  Tooltip,
  Legend,
} from 'chart.js'
import type { ProviderUsage } from '~/composables/useUsageApi'

ChartJS.register(ArcElement, CategoryScale, LinearScale, BarElement, Title, Tooltip, Legend)

const props = withDefaults(defineProps<{
  providers: ProviderUsage[]
  loading?: boolean
  chartType?: 'pie' | 'bar'
}>(), {
  chartType: 'pie',
})

const chartColors = [
  'rgba(34, 211, 238, 0.8)',   // cyan-400
  'rgba(168, 85, 247, 0.8)',   // purple-500
  'rgba(74, 222, 128, 0.8)',   // green-400
  'rgba(251, 191, 36, 0.8)',   // amber-400
  'rgba(251, 113, 133, 0.8)',  // rose-400
  'rgba(96, 165, 250, 0.8)',   // blue-400
  'rgba(244, 114, 182, 0.8)',  // pink-400
  'rgba(45, 212, 191, 0.8)',   // teal-400
]

const borderColors = [
  'rgba(34, 211, 238, 1)',
  'rgba(168, 85, 247, 1)',
  'rgba(74, 222, 128, 1)',
  'rgba(251, 191, 36, 1)',
  'rgba(251, 113, 133, 1)',
  'rgba(96, 165, 250, 1)',
  'rgba(244, 114, 182, 1)',
  'rgba(45, 212, 191, 1)',
]

const chartData = computed(() => {
  const labels = props.providers.map(p => p.provider)
  const data = props.providers.map(p => p.estimated_cost)
  
  return {
    labels,
    datasets: [{
      data,
      backgroundColor: chartColors.slice(0, labels.length),
      borderColor: borderColors.slice(0, labels.length),
      borderWidth: 1,
    }],
  }
})

const pieOptions = {
  responsive: true,
  maintainAspectRatio: false,
  plugins: {
    legend: {
      position: 'right' as const,
      labels: {
        color: '#9ca3af',
        font: {
          family: 'JetBrains Mono',
          size: 11,
        },
        padding: 12,
        usePointStyle: true,
        pointStyle: 'circle',
      },
    },
    tooltip: {
      backgroundColor: 'rgba(17, 24, 39, 0.95)',
      titleColor: '#f3f4f6',
      bodyColor: '#d1d5db',
      borderColor: 'rgba(55, 65, 81, 0.5)',
      borderWidth: 1,
      padding: 12,
      callbacks: {
        label: (context: { label: string; raw: number }) => {
          const value = context.raw
          return ` ${context.label}: $${value.toFixed(4)}`
        },
      },
    },
  },
}

const barOptions = {
  responsive: true,
  maintainAspectRatio: false,
  indexAxis: 'y' as const,
  plugins: {
    legend: {
      display: false,
    },
    tooltip: {
      backgroundColor: 'rgba(17, 24, 39, 0.95)',
      titleColor: '#f3f4f6',
      bodyColor: '#d1d5db',
      borderColor: 'rgba(55, 65, 81, 0.5)',
      borderWidth: 1,
      padding: 12,
      callbacks: {
        label: (context: { raw: number }) => `$${context.raw.toFixed(4)}`,
      },
    },
  },
  scales: {
    x: {
      grid: {
        color: 'rgba(55, 65, 81, 0.3)',
      },
      ticks: {
        color: '#9ca3af',
        font: {
          family: 'JetBrains Mono',
          size: 10,
        },
        callback: (value: number) => `$${value.toFixed(3)}`,
      },
    },
    y: {
      grid: {
        display: false,
      },
      ticks: {
        color: '#d1d5db',
        font: {
          family: 'JetBrains Mono',
          size: 11,
        },
      },
    },
  },
}

const formatCost = (cost: number): string => {
  if (cost >= 1) return `$${cost.toFixed(2)}`
  if (cost >= 0.01) return `$${cost.toFixed(3)}`
  return `$${cost.toFixed(4)}`
}
</script>

<template>
  <div class="bg-gray-900 border border-gray-800 rounded-xl p-5">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-white">Cost by Provider</h3>
      <div class="flex gap-2">
        <button
          @click="$emit('update:chartType', 'pie')"
          class="px-3 py-1.5 text-xs rounded-lg transition-colors"
          :class="chartType === 'pie' 
            ? 'bg-cyan-900/50 text-cyan-400 border border-cyan-700/50' 
            : 'bg-gray-800 text-gray-400 hover:bg-gray-700 border border-gray-700'"
        >
          Pie
        </button>
        <button
          @click="$emit('update:chartType', 'bar')"
          class="px-3 py-1.5 text-xs rounded-lg transition-colors"
          :class="chartType === 'bar' 
            ? 'bg-cyan-900/50 text-cyan-400 border border-cyan-700/50' 
            : 'bg-gray-800 text-gray-400 hover:bg-gray-700 border border-gray-700'"
        >
          Bar
        </button>
      </div>
    </div>
    
    <div v-if="loading" class="h-64 flex items-center justify-center">
      <div class="w-32 h-32 border-2 border-cyan-500/30 border-t-cyan-500 rounded-full animate-spin" />
    </div>
    
    <div v-else-if="providers.length === 0" class="h-64 flex items-center justify-center text-gray-500">
      <div class="text-center">
        <svg class="w-12 h-12 mx-auto mb-2 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
        </svg>
        <p>No usage data yet</p>
      </div>
    </div>
    
    <div v-else class="h-64">
      <Doughnut v-if="chartType === 'pie'" :data="chartData" :options="pieOptions" />
      <Bar v-else :data="chartData" :options="barOptions" />
    </div>
    
    <!-- Provider List -->
    <div v-if="providers.length > 0" class="mt-4 space-y-2">
      <div 
        v-for="(provider, idx) in providers" 
        :key="provider.provider"
        class="flex items-center justify-between py-2 px-3 rounded-lg bg-gray-800/50"
      >
        <div class="flex items-center gap-2">
          <span 
            class="w-3 h-3 rounded-full" 
            :style="{ backgroundColor: chartColors[idx % chartColors.length] }"
          />
          <span class="text-sm text-gray-300">{{ provider.provider }}</span>
        </div>
        <span class="text-sm font-mono text-white">{{ formatCost(provider.estimated_cost) }}</span>
      </div>
    </div>
  </div>
</template>