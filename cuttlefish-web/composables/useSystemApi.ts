/**
 * useSystemApi - Composable for system configuration and status
 * Fetches real settings from the server API instead of using mock data
 */

export interface ProviderConfig {
  id: string
  name: string
  type: string
  enabled: boolean
  apiKey: string
  model: string
  models: string[]
  connected: boolean | null
}

export interface AgentSettings {
  orchestrator: { category: string }
  coder: { category: string }
  critic: { category: string }
  planner: { category: string }
  explorer: { category: string }
  librarian: { category: string }
}

export interface SandboxConfig {
  memoryLimitMb: number
  cpuLimit: number
  diskLimitGb: number
  maxConcurrent: number
}

export interface SystemConfig {
  apiKey: string
  providers: ProviderConfig[]
  agentSettings: AgentSettings
  sandbox: SandboxConfig
  tunnelConnected: boolean
  version: string
}

export interface SystemStatus {
  version: string
  uptime: number
  connected: boolean
}

export function useSystemApi() {
  const config = useRuntimeConfig()
  const loading = ref(false)
  const error = ref<string | null>(null)

  /**
   * Fetch system configuration from server
   */
  const fetchConfig = async (): Promise<SystemConfig | null> => {
    loading.value = true
    error.value = null
    try {
      const data = await $fetch<SystemConfig>(`${config.public.apiBase}/api/system/config`)
      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch system config'
      return null
    } finally {
      loading.value = false
    }
  }

  /**
   * Save system configuration to server
   */
  const saveConfig = async (systemConfig: Partial<SystemConfig>): Promise<boolean> => {
    loading.value = true
    error.value = null
    try {
      await $fetch(`${config.public.apiBase}/api/system/config`, {
        method: 'PUT',
        body: systemConfig,
      })
      return true
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to save config'
      return false
    } finally {
      loading.value = false
    }
  }

  /**
   * Test a provider connection
   */
  const testProvider = async (providerId: string): Promise<boolean> => {
    try {
      const data = await $fetch<{ connected: boolean }>(`${config.public.apiBase}/api/system/providers/${providerId}/test`, {
        method: 'POST',
      })
      return data?.connected ?? false
    } catch {
      return false
    }
  }

  /**
   * Get system status (version, uptime, etc.)
   */
  const fetchStatus = async (): Promise<SystemStatus | null> => {
    try {
      const data = await $fetch<SystemStatus>(`${config.public.apiBase}/api/system/status`)
      return data
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to fetch status'
      return null
    }
  }

  /**
   * Regenerate API key
   */
  const regenerateApiKey = async (): Promise<string | null> => {
    try {
      const data = await $fetch<{ apiKey: string }>(`${config.public.apiBase}/api/system/api-key/regenerate`, {
        method: 'POST',
      })
      return data?.apiKey ?? null
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Failed to regenerate API key'
      return null
    }
  }

  return {
    loading,
    error,
    fetchConfig,
    saveConfig,
    testProvider,
    fetchStatus,
    regenerateApiKey,
  }
}
