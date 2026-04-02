export interface ChatMessage {
  sender: string
  content: string
  type: 'chat' | 'log' | 'diff'
  projectId: string
  timestamp: number
}

export function useWebSocket(apiKey?: string) {
  const config = useRuntimeConfig()
  const messages = ref<ChatMessage[]>([])
  const logLines = ref<string[]>([])
  const diffContent = ref('')
  const connected = ref(false)
  
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
  
  onMounted(() => connect())
  onUnmounted(() => disconnect())
  
  return { messages, logLines, diffContent, connected, send }
}