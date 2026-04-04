<script setup lang="ts">
import { Line } from 'vue-chartjs'
import {
  Chart as ChartJS,
  CategoryScale,
  LinearScale,
  PointElement,
  LineElement,
  Title,
  Tooltip,
  Legend,
  Filler,
} from 'chart.js'
import type { DailyUsage } from '~/composables/useUsageApi'

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend, Filler)

const props = defineProps<{
  dailyData: DailyUsage[]
  loading?: boolean
}>()

const chartData = computed(() => {
  const labels = props.dailyData.map(d => {
    const date = new Date(d.date)
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' })
  })
  
  const costs = props.dailyData.map(d => d.estimated_cost)
  const inputTokens = props.dailyData.map(d => d.input_tokens)
  const outputTokens = props.dailyData.map(d => d.output_tokens)
  
  return {
    labels,
    datasets: [
      {
        label: 'Cost ($)',
        data: costs,
        borderColor: 'rgba(34, 211, 238, 1)',
        backgroundColor: 'rgba(34, 211, 238, 0.1)',
        fill: true,
        tension: 0.4,
        pointRadius: 3,
        pointHoverRadius: 6,
        pointBackgroundColor: 'rgba(34, 211, 238, 1)',
        pointBorderColor: 'rgba(17, 24, 39, 1)',
        pointBorderWidth: 2,
        yAxisID: 'y',
      },
      {
        label: 'Input Tokens',
        data: inputTokens,
        borderColor: 'rgba(168, 85, 247, 1)',
        backgroundColor: 'transparent',
        borderDash: [5, 5],
        tension: 0.4,
        pointRadius: 2,
        pointHoverRadius: 5,
        yAxisID: 'y1',
      },
      {
        label: 'Output Tokens',
        data: outputTokens,
        borderColor: 'rgba(74, 222, 128, 1)',
        backgroundColor: 'transparent',
        borderDash: [5, 5],
        tension: 0.4,
        pointRadius: 2,
        pointHoverRadius: 5,
        yAxisID: 'y1',
      },
    ],
  }
})

const chartOptions = {
  responsive: true,
  maintainAspectRatio: false,
  interaction: {
    mode: 'index' as const,
    intersect: false,
  },
  plugins: {
    legend: {
      position: 'top' as const,
      labels: {
        color: '#9ca3af',
        font: {
          family: 'JetBrains Mono',
          size: 11,
        },
        padding: 16,
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
        label: (context: { dataset: { label: string }; raw: number }) => {
          const value = context.raw
          if (context.dataset.label === 'Cost ($)') {
            return ` ${context.dataset.label}: $${value.toFixed(4)}`
          }
          return ` ${context.dataset.label}: ${value.toLocaleString()}`
        },
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
        maxRotation: 45,
      },
    },
    y: {
      type: 'linear' as const,
      display: true,
      position: 'left' as const,
      grid: {
        color: 'rgba(55, 65, 81, 0.3)',
      },
      ticks: {
        color: '#22d3ee',
        font: {
          family: 'JetBrains Mono',
          size: 10,
        },
        callback: (value: number) => `$${value.toFixed(3)}`,
      },
      title: {
        display: true,
        text: 'Cost (USD)',
        color: '#22d3ee',
        font: {
          family: 'JetBrains Mono',
          size: 11,
        },
      },
    },
    y1: {
      type: 'linear' as const,
      display: true,
      position: 'right' as const,
      grid: {
        drawOnChartArea: false,
      },
      ticks: {
        color: '#a855f7',
        font: {
          family: 'JetBrains Mono',
          size: 10,
        },
        callback: (value: number) => {
          if (value >= 1000000) return `${(value / 1000000).toFixed(1)}M`
          if (value >= 1000) return `${(value / 1000).toFixed(0)}K`
          return value.toString()
        },
      },
      title: {
        display: true,
        text: 'Tokens',
        color: '#a855f7',
        font: {
          family: 'JetBrains Mono',
          size: 11,
        },
      },
    },
  },
}
</script>

<template>
  <div class="bg-gray-900 border border-gray-800 rounded-xl p-5">
    <h3 class="text-lg font-semibold text-white mb-4">Daily Usage Trend</h3>
    
    <div v-if="loading" class="h-72 flex items-center justify-center">
      <div class="w-16 h-16 border-2 border-cyan-500/30 border-t-cyan-500 rounded-full animate-spin" />
    </div>
    
    <div v-else-if="dailyData.length === 0" class="h-72 flex items-center justify-center text-gray-500">
      <div class="text-center">
        <svg class="w-12 h-12 mx-auto mb-2 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z" />
        </svg>
        <p>No usage data available</p>
      </div>
    </div>
    
    <div v-else class="h-72">
      <Line :data="chartData" :options="chartOptions" />
    </div>
  </div>
</template>