export interface ChatMessage {
  sender: string
  content: string
  type: 'chat' | 'log' | 'diff'
  projectId: string
  timestamp: number
  isStreaming?: boolean
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

export interface LogEntry {
  id: string
  timestamp: string
  agent: string
  action: string
  level: 'info' | 'warn' | 'error'
  project: string
  context?: string
  stackTrace?: string
}

export function useWebSocket(apiKey?: string) {
  const config = useRuntimeConfig()
  const messages = ref<ChatMessage[]>([])
  const logLines = ref<string[]>([])
  const logEntries = ref<LogEntry[]>([])
  const diffContent = ref('')
  const connected = ref(false)
  const pendingApprovals = ref<PendingApprovalEvent[]>([])

  // Track streaming messages by project_id + agent
  const streamingMessages = new Map<string, number>() // key -> message index

  let ws: WebSocket | null = null
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null
  
  const connect = () => {
    if (ws) return
    try {
      const wsProtocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
      const wsBase = config.public.wsUrl || `${wsProtocol}//${window.location.host}`
      const url = `${wsBase}/ws${apiKey ? `?key=${apiKey}` : ''}`
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
          } else if (msg.type === 'log_entry') {
            logEntries.value.push({
              id: msg.id,
              timestamp: msg.timestamp,
              agent: msg.agent,
              action: msg.action,
              level: msg.level,
              project: msg.project,
              context: msg.context,
              stackTrace: msg.stack_trace,
            })
            // Keep max 1000 log entries to prevent memory issues
            if (logEntries.value.length > 1000) {
              logEntries.value.splice(0, 100)
            }
          } else if (msg.type === 'stream_chunk') {
            // Handle streaming chunks - accumulate into a message
            const streamKey = `${msg.project_id}:${msg.agent}`

            if (msg.done) {
              // Final chunk - mark message as complete
              const msgIdx = streamingMessages.get(streamKey)
              if (msgIdx !== undefined && messages.value[msgIdx]) {
                messages.value[msgIdx].isStreaming = false
              }
              streamingMessages.delete(streamKey)
            } else if (msg.content) {
              // Accumulate content
              const existingIdx = streamingMessages.get(streamKey)
              if (existingIdx !== undefined && messages.value[existingIdx]) {
                // Append to existing streaming message
                messages.value[existingIdx].content += msg.content
              } else {
                // Create new streaming message
                const newIdx = messages.value.length
                messages.value.push({
                  sender: msg.agent || 'assistant',
                  content: msg.content,
                  type: 'chat',
                  projectId: msg.project_id || '',
                  timestamp: Date.now(),
                  isStreaming: true,
                })
                streamingMessages.set(streamKey, newIdx)
              }
            }
          } else if (msg.type === 'error') {
            console.error('[WebSocket] Server error:', msg.message)
            // Could emit an event or store in a ref for UI display
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

  const approve = (actionId: string) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'approve', action_id: actionId }))
    }
  }

  const reject = (actionId: string, reason?: string) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'reject', action_id: actionId, reason }))
    }
  }

  const subscribe = (projectId: string) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'subscribe', project_id: projectId }))
    }
  }

  const unsubscribe = (projectId: string) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({ type: 'unsubscribe', project_id: projectId }))
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
    logEntries,
    diffContent,
    connected,
    send,
    approve,
    reject,
    subscribe,
    unsubscribe,
    pendingApprovals,
    removePendingApproval,
  }
}