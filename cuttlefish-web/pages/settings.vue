<script setup lang="ts">
/**
 * Settings Page - Configuration interface for Cuttlefish instance
 * Manages API keys, model routing, sandbox limits, and tunnel status
 */

// State
const settings = ref({
  apiKey: 'sk-****************************abcd',
  modelCategory: {
    deep: 'claude-3-5-sonnet',
    quick: 'claude-3-haiku',
    ultrabrain: 'claude-3-opus',
  },
  sandbox: {
    memoryLimit: 2048,
    cpuLimit: 2,
    diskLimit: 10,
  },
  tunnelConnected: false,
})

const saving = ref(false)
const showSuccess = ref(false)
const copied = ref(false)

const modelOptions = [
  'claude-3-5-sonnet',
  'claude-3-opus',
  'claude-3-haiku',
]

const save = async () => {
  saving.value = true
  // Simulate save
  await new Promise(r => setTimeout(r, 1000))
  saving.value = false
  showSuccess.value = true
  setTimeout(() => showSuccess.value = false, 3000)
}

const regenerateApiKey = () => {
  if (confirm('Are you sure? This will invalidate your current key.')) {
    settings.value.apiKey = 'sk-****************************' + Math.random().toString(36).slice(-4)
  }
}

const copyApiKey = async () => {
  try {
    await navigator.clipboard.writeText(settings.value.apiKey)
    copied.value = true
    setTimeout(() => copied.value = false, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

// Form validation helpers
const validateNumber = (value: number, min: number, max: number): boolean => {
  return !isNaN(value) && value >= min && value <= max
}

const memoryValid = computed(() => validateNumber(settings.value.sandbox.memoryLimit, 512, 16384))
const cpuValid = computed(() => validateNumber(settings.value.sandbox.cpuLimit, 0.5, 16))
const diskValid = computed(() => validateNumber(settings.value.sandbox.diskLimit, 1, 100))
</script>

<template>
  <div class="p-6 max-w-4xl">
    <!-- Header -->
    <div class="mb-8">
      <h1 class="text-2xl font-bold text-white mb-2">Settings</h1>
      <p class="text-gray-400">Configure your Cuttlefish instance</p>
    </div>
    
    <!-- Success toast -->
    <Transition
      enter-active-class="transition ease-out duration-300"
      enter-from-class="opacity-0 transform -translate-y-2"
      enter-to-class="opacity-100 transform translate-y-0"
      leave-active-class="transition ease-in duration-200"
      leave-from-class="opacity-100 transform translate-y-0"
      leave-to-class="opacity-0 transform -translate-y-2"
    >
      <div v-if="showSuccess" class="mb-6 p-4 bg-green-900/50 border border-green-700 rounded-lg text-green-400 flex items-center gap-2">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        Settings saved successfully
      </div>
    </Transition>
    
    <!-- API Keys Section -->
    <section class="mb-8 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl">
      <h2 class="text-lg font-semibold text-white mb-4">API Keys</h2>
      <div class="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
        <input 
          type="text" 
          :value="settings.apiKey" 
          readonly
          class="flex-1 px-4 py-3 sm:py-2 min-h-[44px] bg-gray-800 border border-gray-700 rounded-lg text-gray-400 font-mono text-sm focus:outline-none"
        />
        <div class="flex gap-2">
          <button 
            @click="copyApiKey" 
            class="flex-1 sm:flex-none px-4 py-3 sm:py-2 min-h-[44px] rounded-lg transition-all duration-200"
            :class="copied 
              ? 'bg-green-900/50 text-green-400 border border-green-700' 
              : 'bg-gray-800 text-gray-300 hover:bg-gray-700 border border-gray-700'"
          >
            {{ copied ? '✓ Copied!' : 'Copy' }}
          </button>
          <button 
            @click="regenerateApiKey" 
            class="flex-1 sm:flex-none px-4 py-3 sm:py-2 min-h-[44px] bg-red-900/30 hover:bg-red-900/50 text-red-400 border border-red-900/50 rounded-lg transition-colors"
          >
            Regenerate
          </button>
        </div>
      </div>
      <p class="text-xs text-gray-500 mt-3">Use this key to authenticate API requests</p>
    </section>
    
<!-- Model Configuration -->
    <section class="mb-8 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl">
      <h2 class="text-lg font-semibold text-white mb-4">Model Configuration</h2>
      <p class="text-sm text-gray-500 mb-4">Assign models to task categories for optimal routing</p>
      <div class="space-y-4">
        <div v-for="(model, category) in settings.modelCategory" :key="category" class="flex flex-col sm:flex-row sm:items-center gap-2 sm:gap-4">
          <label class="sm:w-32 text-gray-400 capitalize text-sm font-medium">{{ category }}</label>
          <select
            v-model="settings.modelCategory[category]"
            class="w-full sm:flex-1 px-4 py-3 sm:py-2 min-h-[44px] bg-gray-800 border border-gray-700 rounded-lg text-white text-sm focus:outline-none focus:border-cyan-500 transition-colors"
          >
            <option v-for="opt in modelOptions" :key="opt" :value="opt">{{ opt }}</option>
          </select>
        </div>
      </div>
    </section>
    
<!-- Sandbox Limits -->
    <section class="mb-8 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl">
      <h2 class="text-lg font-semibold text-white mb-4">Sandbox Limits</h2>
      <p class="text-sm text-gray-500 mb-4">Resource constraints for Docker containers</p>
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div>
          <label class="block text-gray-400 mb-2 text-sm">Memory (MB)</label>
          <input
            type="number" 
            v-model.number="settings.sandbox.memoryLimit"
            class="w-full px-4 py-3 sm:py-2 min-h-[44px] bg-gray-800 border rounded-lg text-white text-sm focus:outline-none transition-colors"
            :class="memoryValid ? 'border-gray-700 focus:border-cyan-500' : 'border-red-700 focus:border-red-500'"
            min="512"
            max="16384"
          />
          <p v-if="!memoryValid" class="text-xs text-red-400 mt-1">Range: 512-16384 MB</p>
        </div>
        <div>
          <label class="block text-gray-400 mb-2 text-sm">CPU Cores</label>
          <input
            type="number" 
            v-model.number="settings.sandbox.cpuLimit"
            step="0.5"
            class="w-full px-4 py-3 sm:py-2 min-h-[44px] bg-gray-800 border rounded-lg text-white text-sm focus:outline-none transition-colors"
            :class="cpuValid ? 'border-gray-700 focus:border-cyan-500' : 'border-red-700 focus:border-red-500'"
            min="0.5"
            max="16"
          />
          <p v-if="!cpuValid" class="text-xs text-red-400 mt-1">Range: 0.5-16 cores</p>
        </div>
        <div>
          <label class="block text-gray-400 mb-2 text-sm">Disk (GB)</label>
          <input
            type="number" 
            v-model.number="settings.sandbox.diskLimit"
            class="w-full px-4 py-3 sm:py-2 min-h-[44px] bg-gray-800 border rounded-lg text-white text-sm focus:outline-none transition-colors"
            :class="diskValid ? 'border-gray-700 focus:border-cyan-500' : 'border-red-700 focus:border-red-500'"
            min="1"
            max="100"
          />
          <p v-if="!diskValid" class="text-xs text-red-400 mt-1">Range: 1-100 GB</p>
        </div>
      </div>
    </section>
    
    <!-- Tunnel Status -->
    <section class="mb-8 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl">
      <h2 class="text-lg font-semibold text-white mb-4">Tunnel</h2>
      <div class="flex flex-col sm:flex-row sm:items-center gap-3 sm:gap-4">
        <div class="flex items-center gap-3">
          <div 
            class="w-3 h-3 rounded-full transition-colors duration-300"
            :class="settings.tunnelConnected ? 'bg-green-400' : 'bg-red-400'"
          />
          <span class="text-white">{{ settings.tunnelConnected ? 'Connected' : 'Disconnected' }}</span>
        </div>
        <button 
          v-if="!settings.tunnelConnected" 
          class="px-4 py-3 sm:py-2 min-h-[44px] bg-cyan-600 hover:bg-cyan-500 text-white rounded-lg text-sm font-medium transition-colors"
        >
          Generate Link Code
        </button>
      </div>
      <p class="text-xs text-gray-500 mt-3">Connect external services via secure tunnel</p>
    </section>
    
    <!-- About -->
    <section class="mb-8 p-4 sm:p-6 bg-gray-900 border border-gray-800 rounded-xl">
      <h2 class="text-lg font-semibold text-white mb-4">About</h2>
      <div class="space-y-2 text-gray-400 text-sm">
        <p>Version: <span class="text-white font-mono">0.1.0</span></p>
        <p class="flex flex-col sm:flex-row sm:items-center gap-1 sm:gap-2">
          <a href="https://github.com/JackTYM/cuttlefish-rs" class="text-cyan-400 hover:text-cyan-300 transition-colors">GitHub</a>
          <span class="hidden sm:inline text-gray-700">•</span>
          <NuxtLink to="/docs" class="text-cyan-400 hover:text-cyan-300 transition-colors">Documentation</NuxtLink>
        </p>
      </div>
    </section>
    
    <!-- Save Button -->
    <div class="flex justify-end">
      <button 
        @click="save"
        :disabled="saving || !memoryValid || !cpuValid || !diskValid"
        class="w-full sm:w-auto px-6 py-3 sm:py-3 min-h-[44px] bg-cyan-600 hover:bg-cyan-500 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg font-medium transition-colors flex items-center justify-center gap-2"
      >
        <svg v-if="saving" class="animate-spin w-4 h-4" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        {{ saving ? 'Saving...' : 'Save Changes' }}
      </button>
    </div>
  </div>
</template>