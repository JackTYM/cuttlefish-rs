/**
 * useSafetyApi - Composable for safety-related API calls
 * 
 * Provides methods for:
 * - Fetching pending actions
 * - Approving/rejecting actions
 * - Getting diff previews
 * - Managing checkpoints
 * - Gate configuration
 */

export interface PendingAction {
  id: string
  projectId: string
  actionType: string
  description: string
  path?: string
  command?: string
  confidence: number
  confidenceReasoning: string
  riskFactors?: RiskFactor[]
  createdAt: string
  timeoutSecs: number
  hasDiff: boolean
}

export interface RiskFactor {
  type: string
  description: string
}

export interface DiffPreview {
  actionId: string
  path: string
  isNewFile: boolean
  isDeletion: boolean
  language?: string
  stats: {
    linesAdded: number
    linesRemoved: number
    hunks: number
  }
  unifiedDiff: string
}

export interface Checkpoint {
  id: string
  projectId: string
  createdAt: string
  description: string
  trigger: string
  gitRef: string
  containerSnapshotId: string
  memoryBackupPath: string
}

export interface GateConfig {
  autoApproveThreshold: number
  promptThreshold: number
}

export interface CreateCheckpointRequest {
  description: string
  gitRef?: string
  containerSnapshotId?: string
}

export interface UpdateGateConfigRequest {
  autoApproveThreshold?: number
  promptThreshold?: number
}

export function useSafetyApi() {
  const config = useRuntimeConfig()
  
  // Get API key from storage
  const getApiKey = () => {
    if (typeof window === 'undefined') return ''
    return localStorage.getItem('apiKey') || ''
  }
  
  // Common headers
  const headers = () => ({
    'Content-Type': 'application/json',
    'Authorization': `Bearer ${getApiKey()}`,
  })
  
  // State
  const pendingActions = ref<PendingAction[]>([])
  const loading = ref(false)
  const error = ref<string | null>(null)
  
  // Fetch pending actions for a project
  const fetchPendingActions = async (projectId: string) => {
    loading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/projects/${projectId}/actions/pending`,
        { headers: headers() }
      )
      if (!response.ok) throw new Error('Failed to fetch pending actions')
      const data = await response.json()
      pendingActions.value = data
      return data
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Unknown error'
      return []
    } finally {
      loading.value = false
    }
  }
  
  // Approve an action
  const approveAction = async (actionId: string) => {
    loading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/actions/${actionId}/approve`,
        {
          method: 'POST',
          headers: headers(),
        }
      )
      if (!response.ok) throw new Error('Failed to approve action')
      const data = await response.json()
      // Remove from local state
      pendingActions.value = pendingActions.value.filter(a => a.id !== actionId)
      return { success: true, data }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Unknown error'
      return { success: false, error: error.value }
    } finally {
      loading.value = false
    }
  }
  
  // Reject an action
  const rejectAction = async (actionId: string) => {
    loading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/actions/${actionId}/reject`,
        {
          method: 'POST',
          headers: headers(),
        }
      )
      if (!response.ok) throw new Error('Failed to reject action')
      const data = await response.json()
      // Remove from local state
      pendingActions.value = pendingActions.value.filter(a => a.id !== actionId)
      return { success: true, data }
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Unknown error'
      return { success: false, error: error.value }
    } finally {
      loading.value = false
    }
  }
  
  // Get diff preview for an action
  const getActionDiff = async (actionId: string): Promise<DiffPreview | null> => {
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/actions/${actionId}/diff`,
        { headers: headers() }
      )
      if (!response.ok) throw new Error('Failed to fetch diff')
      return await response.json()
    } catch (err) {
      console.error('Failed to fetch diff:', err)
      return null
    }
  }
  
  // Get gate configuration
  const getGateConfig = async (projectId: string): Promise<GateConfig | null> => {
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/projects/${projectId}/safety/config`,
        { headers: headers() }
      )
      if (!response.ok) throw new Error('Failed to fetch gate config')
      return await response.json()
    } catch (err) {
      console.error('Failed to fetch gate config:', err)
      return null
    }
  }
  
  // Update gate configuration
  const updateGateConfig = async (projectId: string, updates: UpdateGateConfigRequest): Promise<GateConfig | null> => {
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/projects/${projectId}/safety/config`,
        {
          method: 'PUT',
          headers: headers(),
          body: JSON.stringify(updates),
        }
      )
      if (!response.ok) throw new Error('Failed to update gate config')
      return await response.json()
    } catch (err) {
      console.error('Failed to update gate config:', err)
      return null
    }
  }
  
  // List checkpoints
  const listCheckpoints = async (projectId: string, page = 1, perPage = 20): Promise<Checkpoint[]> => {
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/projects/${projectId}/checkpoints?page=${page}&per_page=${perPage}`,
        { headers: headers() }
      )
      if (!response.ok) throw new Error('Failed to list checkpoints')
      return await response.json()
    } catch (err) {
      console.error('Failed to list checkpoints:', err)
      return []
    }
  }
  
  // Create checkpoint
  const createCheckpoint = async (projectId: string, request: CreateCheckpointRequest): Promise<Checkpoint | null> => {
    loading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/projects/${projectId}/checkpoints`,
        {
          method: 'POST',
          headers: headers(),
          body: JSON.stringify(request),
        }
      )
      if (!response.ok) throw new Error('Failed to create checkpoint')
      return await response.json()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Unknown error'
      return null
    } finally {
      loading.value = false
    }
  }
  
  // Restore checkpoint (rollback)
  const restoreCheckpoint = async (projectId: string, checkpointId: string, createSafetyCheckpoint = true) => {
    loading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/projects/${projectId}/checkpoints/${checkpointId}/restore`,
        {
          method: 'POST',
          headers: headers(),
          body: JSON.stringify({ create_safety_checkpoint: createSafetyCheckpoint }),
        }
      )
      if (!response.ok) throw new Error('Failed to restore checkpoint')
      return await response.json()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Unknown error'
      return null
    } finally {
      loading.value = false
    }
  }
  
  // Delete checkpoint
  const deleteCheckpoint = async (projectId: string, checkpointId: string): Promise<boolean> => {
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/projects/${projectId}/checkpoints/${checkpointId}`,
        {
          method: 'DELETE',
          headers: headers(),
        }
      )
      return response.ok
    } catch (err) {
      console.error('Failed to delete checkpoint:', err)
      return false
    }
  }
  
  // Undo operations
  const undoOperations = async (projectId: string, count = 1) => {
    loading.value = true
    error.value = null
    try {
      const response = await fetch(
        `${config.public.apiBase}/api/projects/${projectId}/undo`,
        {
          method: 'POST',
          headers: headers(),
          body: JSON.stringify({ count }),
        }
      )
      if (!response.ok) throw new Error('Failed to undo operations')
      return await response.json()
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Unknown error'
      return null
    } finally {
      loading.value = false
    }
  }
  
  return {
    // State
    pendingActions,
    loading,
    error,
    
    // Actions
    fetchPendingActions,
    approveAction,
    rejectAction,
    getActionDiff,
    getGateConfig,
    updateGateConfig,
    listCheckpoints,
    createCheckpoint,
    restoreCheckpoint,
    deleteCheckpoint,
    undoOperations,
  }
}