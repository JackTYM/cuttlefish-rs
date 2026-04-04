export interface ChatMessage {
  sender: string
  content: string
  type: 'chat' | 'log' | 'diff'
  projectId: string
  timestamp: number
}

export interface PendingApprovalEvent {
  id: string
  projectId: string
  actionType: string
  description: string
  path?: string
  command?: string
  confidence: number
  confidenceReasoning: string
  riskFactors?: { type: string; description: string }[]
  createdAt: string
  timeoutSecs: number
  hasDiff: boolean
}

export function useWebSocket(apiKey?: string) {
  const config = useRuntimeConfig()
  const messages = ref<ChatMessage[]>([])
  const logLines = ref<string[]>([])
  const diffContent = ref('')
  const connected = ref(false)
  const pendingApprovals = ref<PendingApprovalEvent[]>([])
  
  let ws: WebSocket | null = null
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  
  const connect = () => {
    if (ws) return
    try {
      const url = `${config.public.wsUrl}${apiKey ? `?key=${apiKey}` : ''}`
      ws = new WebSocket(url)
      
      ws.onopen = () => { connected.value = true }
      ws.onclose = () => {
        connected.value = false
        ws = null
        reconnectTimer = setTimeout(connect, 3000)
      }
      ws.onmessage = (event) => {
        try {
          const msg = JSON.parse(event.data)
          if (msg.type === 'response') {
            messages.value.push({
              sender: msg.agent || 'system',
              content: msg.content,
              type: 'chat',
              projectId: msg.project_id || '',
              timestamp: Date.now(),
            })
          } else if (msg.type === 'build_log') {
            logLines.value.push(msg.line)
            if (logLines.value.length > 2000) logLines.value.splice(0, 100)
          } else if (msg.type === 'diff') {
            diffContent.value = msg.patch
          } else if (msg.type === 'pending_approval') {
            pendingApprovals.value.push({
              id: msg.action_id,
              projectId: msg.project_id,
              actionType: msg.action_type,
              description: msg.description,
              path: msg.path,
              command: msg.command,
              confidence: msg.confidence,
              confidenceReasoning: msg.confidence_reasoning,
              riskFactors: msg.risk_factors,
              createdAt: msg.created_at,
              timeoutSecs: msg.timeout_secs,
              hasDiff: msg.has_diff,
            })
          } else if (msg.type === 'approval_resolved') {
            pendingApprovals.value = pendingApprovals.value.filter(
              p => p.id !== msg.action_id
            )
          }
        } catch {}
      }
    } catch (e) {
      reconnectTimer = setTimeout(connect, 3000)
    }
  }
  
  const send = (projectId: string, content: string) => {
    if (ws?.readyState === WebSocket.OPEN) {
      messages.value.push({ sender: 'user', content, type: 'chat', projectId, timestamp: Date.now() })
      ws.send(JSON.stringify({ type: 'chat', project_id: projectId, content }))
    }
  }
  
  const disconnect = () => {
    if (reconnectTimer) clearTimeout(reconnectTimer)
    ws?.close()
    ws = null
  }
  
  const removePendingApproval = (actionId: string) => {
    pendingApprovals.value = pendingApprovals.value.filter(p => p.id !== actionId)
  }
  
  onMounted(() => connect())
  onUnmounted(() => disconnect())
  
  return { 
    messages, 
    logLines, 
    diffContent, 
    connected, 
    send,
    pendingApprovals,
    removePendingApproval,
  }
}