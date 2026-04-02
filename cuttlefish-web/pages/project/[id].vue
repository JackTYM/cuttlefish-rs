<template>
  <div class="flex flex-col h-full">
    <!-- Project Header with Tabs -->
    <div class="bg-gray-900 border-b border-gray-800 px-6 py-3 flex items-center gap-4 shrink-0">
      <NuxtLink to="/" class="text-cyan-400 hover:text-cyan-300 transition-colors">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
        </svg>
      </NuxtLink>
      <span class="text-lg font-semibold">{{ route.params.id }}</span>
    </div>

    <div class="bg-gray-900 border-b border-gray-800 px-6 flex gap-1 shrink-0">
      <button
        v-for="tab in tabs"
        :key="tab"
        @click="activeTab = tab"
        class="px-4 py-2 text-sm font-medium transition-colors"
        :class="activeTab === tab ? 'text-cyan-400 border-b-2 border-cyan-400' : 'text-gray-400 hover:text-white'"
      >
        {{ tab }}
      </button>
    </div>

    <div v-if="activeTab === 'Chat'" class="flex flex-col flex-1 overflow-hidden">
      <div ref="chatEl" class="flex-1 overflow-y-auto p-6 space-y-4">
        <div v-for="(msg, i) in projectMessages" :key="i" class="flex gap-3">
          <div
            class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold shrink-0"
            :class="{
              'bg-cyan-700': msg.sender === 'user',
              'bg-purple-700': msg.sender === 'orchestrator',
              'bg-yellow-700': msg.sender === 'coder',
              'bg-red-700': msg.sender === 'critic',
              'bg-gray-700': !['user','orchestrator','coder','critic'].includes(msg.sender)
            }"
          >{{ msg.sender[0]?.toUpperCase() }}</div>
          <div class="flex-1">
            <div class="text-xs text-gray-500 mb-1">{{ msg.sender }}</div>
            <div class="prose prose-invert prose-sm max-w-none text-gray-200" v-html="renderMarkdown(msg.content)" />
          </div>
        </div>
      </div>
      <div class="border-t border-gray-800 p-4 flex gap-3 shrink-0">
        <input
          v-model="input"
          @keyup.enter="sendMessage"
          placeholder="Describe what you want to build..."
          class="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm focus:outline-none focus:border-cyan-500"
        />
        <button @click="sendMessage" class="bg-cyan-600 hover:bg-cyan-500 text-white px-4 py-2 rounded-lg text-sm transition-colors">Send</button>
      </div>
    </div>

    <div v-if="activeTab === 'Build Log'" class="flex-1 overflow-y-auto p-6 font-mono text-sm bg-black">
      <div v-for="(line, i) in logLines" :key="i"
        class="leading-5"
        :class="{
          'text-red-400': line.includes('error') || line.includes('FAILED'),
          'text-yellow-400': line.includes('warning'),
          'text-green-400': line.includes('ok') || line.includes('PASSED'),
          'text-gray-300': !line.includes('error') && !line.includes('warning') && !line.includes('ok'),
        }"
      >{{ line }}</div>
      <div v-if="!logLines.length" class="text-gray-500 text-center py-8">No build logs yet</div>
    </div>

    <div v-if="activeTab === 'Diff'" class="flex-1 overflow-y-auto p-6 font-mono text-sm">
      <div v-for="(line, i) in diffLines" :key="i"
        class="leading-5"
        :class="{
          'text-green-400 bg-green-950/30': line.startsWith('+') && !line.startsWith('+++'),
          'text-red-400 bg-red-950/30': line.startsWith('-') && !line.startsWith('---'),
          'text-cyan-400': line.startsWith('@@'),
          'text-gray-300': !line.startsWith('+') && !line.startsWith('-') && !line.startsWith('@@'),
        }"
      >{{ line }}</div>
      <div v-if="!diffContent" class="text-gray-500 text-center py-8">No diff yet</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { marked } from 'marked'

const route = useRoute()
const tabs = ['Chat', 'Build Log', 'Diff']
const activeTab = ref('Chat')
const input = ref('')
const chatEl = ref<HTMLElement>()

const { messages, logLines, diffContent, connected, send } = useWebSocket()

const projectId = computed(() => route.params.id as string)
const projectMessages = computed(() => messages.value.filter(m => !m.projectId || m.projectId === projectId.value))
const diffLines = computed(() => diffContent.value.split('\n'))

const renderMarkdown = (text: string) => {
  try { return marked(text) } catch { return text }
}

const sendMessage = () => {
  if (!input.value.trim()) return
  send(projectId.value, input.value)
  input.value = ''
}

watch(() => projectMessages.value.length, async () => {
  await nextTick()
  if (chatEl.value) chatEl.value.scrollTop = chatEl.value.scrollHeight
})
</script>