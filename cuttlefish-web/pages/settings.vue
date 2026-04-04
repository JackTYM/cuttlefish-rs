<script setup lang="ts">
/**
 * Settings Page - Configuration interface for Cuttlefish instance
 * Manages API keys, model providers, agent settings, and preferences
 * Persists to localStorage until API is ready
 */

// =============================================================================
// Types
// =============================================================================

interface ProviderConfig {
  id: string
  name: string
  type: string
  enabled: boolean
  apiKey: string
  model: string
  models: string[]
  connected: boolean | null
}

interface AgentSettings {
  orchestrator: { category: string }
  coder: { category: string }
  critic: { category: string }
  planner: { category: string }
  explorer: { category: string }
  librarian: { category: string }
}

interface NotificationSettings {
  projectCreated: boolean
  projectCompleted: boolean
  agentErrors: boolean
  weeklyDigest: boolean
}

interface Settings {
  apiKey: string
  providers: ProviderConfig[]
  agentSettings: AgentSettings
  notifications: NotificationSettings
  sandbox: {
    memoryLimitMb: number
    cpuLimit: number
    diskLimitGb: number
    maxConcurrent: number
  }
  tunnelConnected: boolean
  version: string
}

// =============================================================================
// Default Settings
// =============================================================================

const defaultSettings: Settings = {
  apiKey: '',
  providers: [
    {
      id: 'anthropic',
      name: 'Anthropic',
      type: 'anthropic',
      enabled: true,
      apiKey: '',
      model: 'claude-sonnet-4-6',
      models: ['claude-opus-4-6', 'claude-sonnet-4-6', 'claude-haiku-4-5'],
      connected: null,
    },
    {
      id: 'openai',
      name: 'OpenAI',
      type: 'openai',
      enabled: false,
      apiKey: '',
      model: 'gpt-5.4',
      models: ['gpt-5.4', 'gpt-5-nano', 'gpt-4o'],
      connected: null,
    },
    {
      id: 'google',
      name: 'Google Gemini',
      type: 'google',
      enabled: false,
      apiKey: '',
      model: 'gemini-2.0-flash',
      models: ['gemini-2.0-flash', 'gemini-1.5-pro'],
      connected: null,
    },
    {
      id: 'bedrock',
      name: 'AWS Bedrock',
      type: 'bedrock',
      enabled: false,
      apiKey: '',
      model: 'anthropic.claude-sonnet-4-6-20260101-v1:0',
      models: [
        'anthropic.claude-opus-4-6-20260101-v1:0',
        'anthropic.claude-sonnet-4-6-20260101-v1:0',
        'anthropic.claude-haiku-4-5-20260101-v1:0',
      ],
      connected: null,
    },
    {
      id: 'ollama',
      name: 'Ollama (Local)',
      type: 'ollama',
      enabled: false,
      apiKey: '',
      model: 'llama3.1',
      models: ['llama3.1', 'llama3.2', 'codellama', 'mistral'],
      connected: null,
    },
  ],
  agentSettings: {
    orchestrator: { category: 'deep' },
    coder: { category: 'deep' },
    critic: { category: 'unspecified-high' },
    planner: { category: 'ultrabrain' },
    explorer: { category: 'quick' },
    librarian: { category: 'quick' },
  },
  notifications: {
    projectCreated: true,
    projectCompleted: true,
    agentErrors: true,
    weeklyDigest: false,
  },
  sandbox: {
    memoryLimitMb: 2048,
    cpuLimit: 2.0,
    diskLimitGb: 10,
    maxConcurrent: 5,
  },
  tunnelConnected: false,
  version: '0.1.0',
}

const categoryOptions = [
  { value: 'ultrabrain', label: 'Ultrabrain (Hardest Logic)' },
  { value: 'deep', label: 'Deep (Complex Work)' },
  { value: 'quick', label: 'Quick (Fast Tasks)' },
  { value: 'visual', label: 'Visual (Frontend/UI)' },
  { value: 'unspecified-high', label: 'Unspecified High' },
  { value: 'unspecified-low', label: 'Unspecified Low' },
]

// =============================================================================
// State
// =============================================================================

const settings = ref<Settings>(JSON.parse(JSON.stringify(defaultSettings)))
const saving = ref(false)
const showSuccess = ref(false)
const showError = ref(false)
const errorMessage = ref('')
const copied = ref<string | null>(null)
const testingProvider = ref<string | null>(null)
const showRegenerateConfirm = ref(false)
const showResetConfirm = ref(false)
const hasUnsavedChanges = ref(false)

// Computed for masked API key display
const maskedApiKey = computed(() => {
  const key = settings.value.apiKey
  if (!key || key.length < 8) return key
  return key.slice(0, 3) + '****' + key.slice(-4)
})

// =============================================================================
// Persistence
// =============================================================================

const STORAGE_KEY = 'cuttlefish-settings'

const loadSettings = () => {
  if (typeof window === 'undefined') return
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored)
      // Merge with defaults to handle new fields
      settings.value = {
        ...defaultSettings,
        ...parsed,
        sandbox: { ...defaultSettings.sandbox, ...parsed.sandbox },
        notifications: { ...defaultSettings.notifications, ...parsed.notifications },
        agentSettings: { ...defaultSettings.agentSettings, ...parsed.agentSettings },
      }
    }
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
}

const saveSettings = async () => {
  saving.value = true
  showError.value = false
  
  try {
    // Simulate API call delay
    await new Promise(r => setTimeout(r, 800))
    
    // Save to localStorage
    if (typeof window !== 'undefined') {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(settings.value))
    }
    
    hasUnsavedChanges.value = false
    showSuccess.value = true
    setTimeout(() => showSuccess.value = false, 3000)
  } catch (e) {
    showError.value = true
    errorMessage.value = e instanceof Error ? e.message : 'Failed to save settings'
  } finally {
    saving.value = false
  }
}

// =============================================================================
// API Key Management
// =============================================================================

const copyApiKey = async () => {
  try {
    await navigator.clipboard.writeText(settings.value.apiKey)
    copied.value = 'main'
    setTimeout(() => copied.value = null, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

const regenerateApiKey = () => {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789'
  let newKey = 'sk-'
  for (let i = 0; i < 32; i++) {
    newKey += chars.charAt(Math.floor(Math.random() * chars.length))
  }
  settings.value.apiKey = newKey
  hasUnsavedChanges.value = true
  showRegenerateConfirm.value = false
}

// =============================================================================
// Provider Management
// =============================================================================

const maskProviderKey = (key: string) => {
  if (!key || key.length < 8) return key
  return key.slice(0, 4) + '****' + key.slice(-4)
}

const copyProviderKey = async (providerId: string) => {
  const provider = settings.value.providers.find(p => p.id === providerId)
  if (!provider) return
  try {
    await navigator.clipboard.writeText(provider.apiKey)
    copied.value = providerId
    setTimeout(() => copied.value = null, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

const testProviderConnection = async (providerId: string) => {
  const provider = settings.value.providers.find(p => p.id === providerId)
  if (!provider) return
  
  testingProvider.value = providerId
  provider.connected = null
  
  try {
    // Simulate connection test
    await new Promise(r => setTimeout(r, 1500))
    
    // Mock: 80% success rate for demo
    provider.connected = Math.random() > 0.2
  } catch (err) {
    provider.connected = false
  } finally {
    testingProvider.value = null
  }
}

const toggleProvider = (providerId: string) => {
  const provider = settings.value.providers.find(p => p.id === providerId)
  if (provider) {
    provider.enabled = !provider.enabled
    hasUnsavedChanges.value = true
  }
}

// =============================================================================
// Validation
// =============================================================================

const validateNumber = (value: number, min: number, max: number): boolean => {
  return !isNaN(value) && value >= min && value <= max
}

const memoryValid = computed(() => 
  validateNumber(settings.value.sandbox.memoryLimitMb, 512, 16384)
)
const cpuValid = computed(() => 
  validateNumber(settings.value.sandbox.cpuLimit, 0.5, 16)
)
const diskValid = computed(() => 
  validateNumber(settings.value.sandbox.diskLimitGb, 1, 100)
)
const concurrentValid = computed(() => 
  validateNumber(settings.value.sandbox.maxConcurrent, 1, 20)
)

const formValid = computed(() => 
  memoryValid.value && cpuValid.value && diskValid.value && concurrentValid.value
)

// =============================================================================
// Reset
// =============================================================================

const resetToDefaults = () => {
  settings.value = JSON.parse(JSON.stringify(defaultSettings))
  hasUnsavedChanges.value = true
  showResetConfirm.value = false
}

// =============================================================================
// Lifecycle
// =============================================================================

onMounted(() => {
  loadSettings()
})

// Track changes
watch(settings, () => {
  hasUnsavedChanges.value = true
}, { deep: true })
</script>

<template>
  <div class="p-4 sm:p-6 max-w-4xl mx-auto">
    <!-- Header -->
    <header class="mb-6 sm:mb-8">
      <h1 class="text-xl sm:text-2xl font-bold text-white mb-2">Settings</h1>
      <p class="text-gray-400 text-sm sm:text-base">Configure your Cuttlefish instance</p>
    </header>
    
    <!-- Success Toast -->
    <Transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="opacity-0 transform -translate-y-2"
      enter-to-class="opacity-100 transform translate-y-0"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="opacity-100 transform translate-y-0"
      leave-to-class="opacity-0 transform -translate-y-2"
    >
      <div 
        v-if="showSuccess" 
        class="fixed top-4 right-4 z-50 p-4 bg-green-900/90 border border-green-700 rounded-lg text-green-400 flex items-center gap-2 shadow-lg"
        role="alert"
      >
        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <span class="text-sm font-medium">Settings saved successfully</span>
      </div>
    </Transition>
    
    <!-- Error Toast -->
    <Transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="opacity-0 transform -translate-y-2"
      enter-to-class="opacity-100 transform translate-y-0"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="opacity-100 transform translate-y-0"
      leave-to-class="opacity-0 transform -translate-y-2"
    >
      <div 
        v-if="showError" 
        class="fixed top-4 right-4 z-50 p-4 bg-red-900/90 border border-red-700 rounded-lg text-red-400 flex items-center gap-2 shadow-lg"
        role="alert"
      >
        <svg class="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
        <span class="text-sm font-medium">{{ errorMessage || 'Failed to save settings' }}</span>
      </div>
    </Transition>
    
    <!-- Unsaved Changes Indicator -->
    <div 
      v-if="hasUnsavedChanges" 
      class="mb-4 p-3 bg-yellow-900/30 border border-yellow-700/50 rounded-lg text-yellow-400 text-sm flex items-center gap-2"
      role="status"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
      </svg>
      You have unsaved changes
    </div>
    
    <!-- API Key Section -->
    <section class="mb-6 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl" aria-labelledby="api-keys-heading">
      <h2 id="api-keys-heading" class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
        </svg>
        API Key
      </h2>
      <p class="text-sm text-gray-500 mb-4">Use this key to authenticate API requests from WebUI and TUI clients</p>
      
      <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
        <div class="relative flex-1">
          <input 
            type="text" 
            :value="maskedApiKey" 
            readonly
            class="w-full px-4 py-3 sm:py-2.5 min-h-[44px] bg-gray-800 border border-gray-700 rounded-lg text-gray-400 font-mono text-sm focus:outline-none"
            aria-label="API key (masked)"
          />
          <span class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-gray-600">read-only</span>
        </div>
        <div class="flex gap-2">
          <button 
            @click="copyApiKey" 
            class="flex-1 sm:flex-none px-4 py-3 sm:py-2.5 min-h-[44px] rounded-lg transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
            :class="copied === 'main' 
              ? 'bg-green-900/50 text-green-400 border border-green-700' 
              : 'bg-gray-800 text-gray-300 hover:bg-gray-700 border border-gray-700'"
          >
            <span v-if="copied === 'main'" class="flex items-center justify-center gap-1.5">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              Copied!
            </span>
            <span v-else class="flex items-center justify-center gap-1.5">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
              </svg>
              Copy
            </span>
          </button>
          <button 
            @click="showRegenerateConfirm = true" 
            class="flex-1 sm:flex-none px-4 py-3 sm:py-2.5 min-h-[44px] bg-red-900/30 hover:bg-red-900/50 text-red-400 border border-red-900/50 rounded-lg transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400"
          >
            <span class="flex items-center justify-center gap-1.5">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
              </svg>
              Regenerate
            </span>
          </button>
        </div>
      </div>
    </section>
    
    <!-- Provider Configuration -->
    <section class="mb-6 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl" aria-labelledby="providers-heading">
      <h2 id="providers-heading" class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
        </svg>
        Model Providers
      </h2>
      <p class="text-sm text-gray-500 mb-4">Configure AI model providers and their API keys</p>
      
      <div class="space-y-4">
        <div 
          v-for="provider in settings.providers" 
          :key="provider.id"
          class="p-4 bg-gray-800/50 border border-gray-700/50 rounded-lg"
          :class="{ 'opacity-60': !provider.enabled }"
        >
          <!-- Provider Header -->
          <div class="flex items-center justify-between mb-3">
            <div class="flex items-center gap-3">
              <button
                @click="toggleProvider(provider.id)"
                class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-800"
                :class="provider.enabled ? 'bg-cyan-600' : 'bg-gray-600'"
                :aria-pressed="provider.enabled"
                :aria-label="`Toggle ${provider.name}`"
              >
                <span 
                  class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
                  :class="provider.enabled ? 'translate-x-6' : 'translate-x-1'"
                />
              </button>
              <span class="font-medium text-white">{{ provider.name }}</span>
              <span class="text-xs text-gray-500 font-mono">{{ provider.type }}</span>
            </div>
            
            <!-- Connection Status -->
            <div class="flex items-center gap-2">
              <template v-if="provider.connected === true">
                <span class="w-2 h-2 rounded-full bg-green-400" aria-hidden="true" />
                <span class="text-xs text-green-400">Connected</span>
              </template>
              <template v-else-if="provider.connected === false">
                <span class="w-2 h-2 rounded-full bg-red-400" aria-hidden="true" />
                <span class="text-xs text-red-400">Failed</span>
              </template>
            </div>
          </div>
          
          <!-- Provider Config (only when enabled) -->
          <div v-if="provider.enabled" class="space-y-3">
            <!-- API Key -->
            <div v-if="provider.type !== 'ollama'">
              <label :for="`provider-key-${provider.id}`" class="block text-xs text-gray-400 mb-1.5">API Key</label>
              <div class="flex gap-2">
                <input
                  :id="`provider-key-${provider.id}`"
                  :type="provider.apiKey ? 'password' : 'text'"
                  v-model="provider.apiKey"
                  :placeholder="`Enter ${provider.name} API key`"
                  class="flex-1 px-3 py-2 min-h-[40px] bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:border-cyan-500 transition-colors"
                />
                <button
                  v-if="provider.apiKey"
                  @click="copyProviderKey(provider.id)"
                  class="px-3 py-2 min-h-[40px] bg-gray-700 hover:bg-gray-600 text-gray-300 rounded-lg transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                  :aria-label="`Copy ${provider.name} API key`"
                >
                  <svg v-if="copied === provider.id" class="w-4 h-4 text-green-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                  </svg>
                  <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
              </div>
            </div>
            
            <!-- Model Selection -->
            <div>
              <label :for="`provider-model-${provider.id}`" class="block text-xs text-gray-400 mb-1.5">Default Model</label>
              <select
                :id="`provider-model-${provider.id}`"
                v-model="provider.model"
                class="w-full px-3 py-2 min-h-[40px] bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:border-cyan-500 transition-colors"
              >
                <option v-for="model in provider.models" :key="model" :value="model">{{ model }}</option>
              </select>
            </div>
            
            <!-- Test Connection -->
            <button
              @click="testProviderConnection(provider.id)"
              :disabled="testingProvider === provider.id || (provider.type !== 'ollama' && !provider.apiKey)"
              class="w-full px-3 py-2 min-h-[40px] bg-gray-700 hover:bg-gray-600 disabled:opacity-50 disabled:cursor-not-allowed text-gray-300 rounded-lg text-sm transition-colors flex items-center justify-center gap-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
            >
              <svg v-if="testingProvider === provider.id" class="w-4 h-4 animate-spin" fill="none" viewBox="0 0 24 24" aria-hidden="true">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
              </svg>
              <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
              </svg>
              {{ testingProvider === provider.id ? 'Testing...' : 'Test Connection' }}
            </button>
          </div>
        </div>
      </div>
    </section>
    
    <!-- Agent Settings -->
    <section class="mb-6 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl" aria-labelledby="agents-heading">
      <h2 id="agents-heading" class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
        </svg>
        Agent Settings
      </h2>
      <p class="text-sm text-gray-500 mb-4">Assign task categories to agents for optimal model routing</p>
      
      <div class="space-y-3">
        <div 
          v-for="(config, agent) in settings.agentSettings" 
          :key="agent"
          class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4 p-3 bg-gray-800/30 rounded-lg"
        >
          <label :for="`agent-${agent}`" class="sm:w-28 text-gray-300 capitalize text-sm font-medium">
            {{ agent }}
          </label>
          <select
            :id="`agent-${agent}`"
            v-model="settings.agentSettings[agent as keyof AgentSettings].category"
            class="w-full sm:flex-1 px-3 py-2.5 sm:py-2 min-h-[40px] bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:border-cyan-500 transition-colors"
          >
            <option v-for="opt in categoryOptions" :key="opt.value" :value="opt.value">
              {{ opt.label }}
            </option>
          </select>
        </div>
      </div>
    </section>
    
    <!-- Sandbox Limits -->
    <section class="mb-6 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl" aria-labelledby="sandbox-heading">
      <h2 id="sandbox-heading" class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
        </svg>
        Sandbox Limits
      </h2>
      <p class="text-sm text-gray-500 mb-4">Resource constraints for Docker containers</p>
      
      <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div>
          <label for="memory-limit" class="block text-gray-400 mb-2 text-sm">Memory (MB)</label>
          <input
            id="memory-limit"
            type="number" 
            v-model.number="settings.sandbox.memoryLimitMb"
            class="w-full px-3 py-2.5 sm:py-2 min-h-[40px] bg-gray-800 border rounded-lg text-white text-sm focus:outline-none transition-colors"
            :class="memoryValid ? 'border-gray-700 focus:border-cyan-500' : 'border-red-700 focus:border-red-500'"
            min="512"
            max="16384"
          />
          <p v-if="!memoryValid" class="text-xs text-red-400 mt-1">Range: 512-16384 MB</p>
        </div>
        <div>
          <label for="cpu-limit" class="block text-gray-400 mb-2 text-sm">CPU Cores</label>
          <input
            id="cpu-limit"
            type="number" 
            v-model.number="settings.sandbox.cpuLimit"
            step="0.5"
            class="w-full px-3 py-2.5 sm:py-2 min-h-[40px] bg-gray-800 border rounded-lg text-white text-sm focus:outline-none transition-colors"
            :class="cpuValid ? 'border-gray-700 focus:border-cyan-500' : 'border-red-700 focus:border-red-500'"
            min="0.5"
            max="16"
          />
          <p v-if="!cpuValid" class="text-xs text-red-400 mt-1">Range: 0.5-16 cores</p>
        </div>
        <div>
          <label for="disk-limit" class="block text-gray-400 mb-2 text-sm">Disk (GB)</label>
          <input
            id="disk-limit"
            type="number" 
            v-model.number="settings.sandbox.diskLimitGb"
            class="w-full px-3 py-2.5 sm:py-2 min-h-[40px] bg-gray-800 border rounded-lg text-white text-sm focus:outline-none transition-colors"
            :class="diskValid ? 'border-gray-700 focus:border-cyan-500' : 'border-red-700 focus:border-red-500'"
            min="1"
            max="100"
          />
          <p v-if="!diskValid" class="text-xs text-red-400 mt-1">Range: 1-100 GB</p>
        </div>
        <div>
          <label for="max-concurrent" class="block text-gray-400 mb-2 text-sm">Max Concurrent</label>
          <input
            id="max-concurrent"
            type="number" 
            v-model.number="settings.sandbox.maxConcurrent"
            class="w-full px-3 py-2.5 sm:py-2 min-h-[40px] bg-gray-800 border rounded-lg text-white text-sm focus:outline-none transition-colors"
            :class="concurrentValid ? 'border-gray-700 focus:border-cyan-500' : 'border-red-700 focus:border-red-500'"
            min="1"
            max="20"
          />
          <p v-if="!concurrentValid" class="text-xs text-red-400 mt-1">Range: 1-20</p>
        </div>
      </div>
    </section>
    
    <!-- Notification Preferences -->
    <section class="mb-6 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl" aria-labelledby="notifications-heading">
      <h2 id="notifications-heading" class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
        </svg>
        Notifications
      </h2>
      <p class="text-sm text-gray-500 mb-4">Configure how you receive updates</p>
      
      <div class="space-y-3">
        <label 
          v-for="(enabled, key) in settings.notifications" 
          :key="key"
          class="flex items-center justify-between p-3 bg-gray-800/30 rounded-lg cursor-pointer hover:bg-gray-800/50 transition-colors"
        >
          <span class="text-gray-300 text-sm capitalize">
            {{ key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase()) }}
          </span>
          <button
            @click.prevent="settings.notifications[key as keyof NotificationSettings] = !settings.notifications[key as keyof NotificationSettings]"
            class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-800"
            :class="settings.notifications[key as keyof NotificationSettings] ? 'bg-cyan-600' : 'bg-gray-600'"
            :aria-pressed="settings.notifications[key as keyof NotificationSettings]"
          >
            <span 
              class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform"
              :class="settings.notifications[key as keyof NotificationSettings] ? 'translate-x-6' : 'translate-x-1'"
            />
          </button>
        </label>
      </div>
    </section>
    
    <!-- Tunnel Status -->
    <section class="mb-6 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl" aria-labelledby="tunnel-heading">
      <h2 id="tunnel-heading" class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
        </svg>
        Tunnel
      </h2>
      <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
        <div class="flex items-center gap-3">
          <div 
            class="w-3 h-3 rounded-full transition-colors duration-300"
            :class="settings.tunnelConnected ? 'bg-green-400' : 'bg-red-400'"
            aria-hidden="true"
          />
          <span class="text-white">{{ settings.tunnelConnected ? 'Connected' : 'Disconnected' }}</span>
        </div>
        <button 
          v-if="!settings.tunnelConnected" 
          class="px-4 py-2.5 sm:py-2 min-h-[40px] bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
        >
          Generate Link Code
        </button>
      </div>
      <p class="text-xs text-gray-500 mt-3">Connect external services via secure tunnel</p>
    </section>
    
    <!-- About -->
    <section class="mb-6 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl" aria-labelledby="about-heading">
      <h2 id="about-heading" class="text-lg font-semibold text-white mb-4 flex items-center gap-2">
        <svg class="w-5 h-5 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        About
      </h2>
      <div class="space-y-2 text-gray-400 text-sm">
        <p>Version: <span class="text-white font-mono">{{ settings.version }}</span></p>
        <p class="flex flex-col sm:flex-row sm:items-center gap-1 sm:gap-2">
          <a href="https://github.com/JackTYM/cuttlefish-rs" target="_blank" rel="noopener noreferrer" class="text-cyan-400 hover:text-cyan-300 transition-colors inline-flex items-center gap-1">
            GitHub
            <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
          </a>
          <span class="hidden sm:inline text-gray-700">•</span>
          <NuxtLink to="/docs" class="text-cyan-400 hover:text-cyan-300 transition-colors">Documentation</NuxtLink>
        </p>
      </div>
    </section>
    
    <!-- Action Buttons -->
    <div class="flex flex-col sm:flex-row justify-between gap-3 pt-4 border-t border-gray-800">
      <button 
        @click="showResetConfirm = true"
        class="px-4 py-3 sm:py-2.5 min-h-[44px] bg-gray-800 hover:bg-gray-700 text-gray-400 hover:text-white border border-gray-700 rounded-lg text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
      >
        Reset to Defaults
      </button>
      <button 
        @click="saveSettings"
        :disabled="saving || !formValid"
        class="px-6 py-3 sm:py-2.5 min-h-[44px] bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
      >
        <svg v-if="saving" class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24" aria-hidden="true">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        {{ saving ? 'Saving...' : 'Save Changes' }}
      </button>
    </div>
    
    <!-- Regenerate Confirmation Modal -->
    <Teleport to="body">
      <div 
        v-if="showRegenerateConfirm" 
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60"
        role="dialog"
        aria-modal="true"
        aria-labelledby="regenerate-title"
      >
        <div class="bg-gray-900 border border-gray-700 rounded-xl p-6 max-w-md w-full shadow-2xl">
          <h3 id="regenerate-title" class="text-lg font-semibold text-white mb-3">Regenerate API Key?</h3>
          <p class="text-gray-400 text-sm mb-6">
            This will invalidate your current key. Any applications using the old key will lose access until updated.
          </p>
          <div class="flex justify-end gap-3">
            <button 
              @click="showRegenerateConfirm = false"
              class="px-4 py-2 min-h-[40px] bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
            >
              Cancel
            </button>
            <button 
              @click="regenerateApiKey"
              class="px-4 py-2 min-h-[40px] bg-red-600 hover:bg-red-500 text-white rounded-lg text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400"
            >
              Regenerate
            </button>
          </div>
        </div>
      </div>
    </Teleport>
    
    <!-- Reset Confirmation Modal -->
    <Teleport to="body">
      <div 
        v-if="showResetConfirm" 
        class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60"
        role="dialog"
        aria-modal="true"
        aria-labelledby="reset-title"
      >
        <div class="bg-gray-900 border border-gray-700 rounded-xl p-6 max-w-md w-full shadow-2xl">
          <h3 id="reset-title" class="text-lg font-semibold text-white mb-3">Reset to Defaults?</h3>
          <p class="text-gray-400 text-sm mb-6">
            This will reset all settings to their default values. Your current configuration will be lost.
          </p>
          <div class="flex justify-end gap-3">
            <button 
              @click="showResetConfirm = false"
              class="px-4 py-2 min-h-[40px] bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg text-sm transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
            >
              Cancel
            </button>
            <button 
              @click="resetToDefaults"
              class="px-4 py-2 min-h-[40px] bg-red-600 hover:bg-red-500 text-white rounded-lg text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400"
            >
              Reset
            </button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>