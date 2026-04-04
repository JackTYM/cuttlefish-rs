/**
 * useUsageApi - Composable for fetching usage and cost data from the API
 * Provides typed interfaces and reactive state for usage dashboard components
 */

export interface UsageSummary {
  user_id: string
  period: string
  total_requests: number
  total_input_tokens: number
  total_output_tokens: number
  total_cost_usd: number
  by_provider: Record<string, number>
}

export interface DailyUsage {
  date: string
  input_tokens: number
  output_tokens: number
  request_count: number
  estimated_cost: number
}

export interface ProviderUsage {
  provider: string
  input_tokens: number
  output_tokens: number
  request_count: number
  estimated_cost: number
  models: ModelUsage[]
}

export interface ModelUsage {
  model: string
  input_tokens: number
  output_tokens: number
  request_count: number
  estimated_cost: number
}

export type TimePeriod = 'daily' | 'weekly' | 'monthly'

export function useUsageApi() {
  const config = useRuntimeConfig()
  const loading = ref(false)
  const error = ref<string | null>(null)

  /**
   * Fetch user's usage summary for a given time period
   */
  const fetchUsageSummary = async (period: TimePeriod = 'monthly'): Promise<UsageSummary | null> => {
    loading.value = true
    error.value = null
    try {
      const data = await $fetch<UsageSummary>(`${config.public.apiBase}/api/usage`, {
        params: { period },
      })
      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch usage summary'
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Fetch daily usage breakdown
   */
  const fetchDailyUsage = async (
    period: TimePeriod = 'monthly',
    projectId?: string
  ): Promise<DailyUsage[] | null> => {
    loading.value = true
    error.value = null
    try {
      const params: Record<string, string> = { period }
      if (projectId) params.project_id = projectId
      const data = await $fetch<{ days: DailyUsage[] }>(`${config.public.apiBase}/api/usage/daily`, {
        params,
      })
      return data?.days ?? []
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch daily usage'
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Fetch provider usage breakdown
   */
  const fetchProviderUsage = async (
    period: TimePeriod = 'monthly',
    projectId?: string
  ): Promise<ProviderUsage[] | null> => {
    loading.value = true
    error.value = null
    try {
      const params: Record<string, string> = { period }
      if (projectId) params.project_id = projectId
      const data = await $fetch<{ providers: ProviderUsage[] }>(`${config.public.apiBase}/api/usage/providers`, {
        params,
      })
      return data?.providers ?? []
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch provider usage'
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Get export URL for CSV download
   */
  const getExportUrl = (period: TimePeriod = 'monthly'): string => {
    return `${config.public.apiBase}/api/usage/export?period=${period}`
  }

  return {
    loading,
    error,
    fetchUsageSummary,
    fetchDailyUsage,
    fetchProviderUsage,
    getExportUrl,
  }
}