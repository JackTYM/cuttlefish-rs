<template>
  <div class="min-h-screen bg-gray-950 text-white flex">
    <!-- Skip to main content link -->
    <a
      href="#main-content"
      class="sr-only focus:not-sr-only focus:absolute focus:z-[100] focus:top-4 focus:left-4 focus:px-4 focus:py-2 focus:bg-cyan-600 focus:text-white focus:rounded-lg focus:outline-none focus:ring-2 focus:ring-cyan-400"
    >
      Skip to main content
    </a>

    <!-- Desktop Sidebar -->
    <aside
      class="w-64 bg-gray-900 border-r border-gray-800 flex flex-col shrink-0 transition-transform duration-300 motion-reduce:transition-none"
      :class="{
        'translate-x-0': mobileMenuOpen || !isMobile,
        '-translate-x-full': isMobile && !mobileMenuOpen,
        'fixed inset-y-0 left-0 z-50': isMobile,
        'relative': !isMobile
      }"
      :aria-hidden="isMobile && !mobileMenuOpen"
    >
      <!-- Logo -->
      <div class="p-4 border-b border-gray-800 flex items-center justify-between">
        <NuxtLink to="/" class="flex items-center gap-2 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 rounded">
          <span class="text-2xl" role="img" aria-label="Cuttlefish logo">🐙</span>
          <span class="text-xl font-bold text-cyan-400">Cuttlefish</span>
        </NuxtLink>
        <button
          v-if="isMobile"
          @click="mobileMenuOpen = false"
          class="p-1 text-gray-400 hover:text-white focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
          aria-label="Close navigation menu"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
      
      <!-- Navigation -->
      <nav class="flex-1 p-4 space-y-1" aria-label="Main navigation">
        <NuxtLink
          to="/"
          class="nav-item"
          :class="{ 'nav-item-active': route.path === '/' }"
          :aria-current="route.path === '/' ? 'page' : undefined"
          @click="mobileMenuOpen = false"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          <span>Projects</span>
        </NuxtLink>
        
        <NuxtLink
          to="/templates"
          class="nav-item"
          :class="{ 'nav-item-active': route.path.startsWith('/templates') }"
          :aria-current="route.path.startsWith('/templates') ? 'page' : undefined"
          @click="mobileMenuOpen = false"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
          </svg>
          <span>Templates</span>
        </NuxtLink>
        
        <NuxtLink
          to="/logs"
          class="nav-item"
          :class="{ 'nav-item-active': route.path.startsWith('/logs') }"
          :aria-current="route.path.startsWith('/logs') ? 'page' : undefined"
          @click="mobileMenuOpen = false"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <span>Logs</span>
        </NuxtLink>
        
        <NuxtLink
          to="/settings"
          class="nav-item"
          :class="{ 'nav-item-active': route.path.startsWith('/settings') }"
          :aria-current="route.path.startsWith('/settings') ? 'page' : undefined"
          @click="mobileMenuOpen = false"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          <span>Settings</span>
        </NuxtLink>
      </nav>
      
      <!-- Connection Status -->
      <div class="p-4 border-t border-gray-800">
        <div class="flex items-center gap-2" role="status" aria-live="polite">
          <span
            class="w-2 h-2 rounded-full transition-colors duration-300 motion-reduce:transition-none"
            :class="connected ? 'bg-green-400' : 'bg-red-400'"
            aria-hidden="true"
          />
          <span class="text-sm text-gray-400">
            {{ connected ? 'Connected' : 'Disconnected' }}
          </span>
        </div>
      </div>
    </aside>
    
    <!-- Mobile Menu Overlay -->
    <div
      v-if="isMobile && mobileMenuOpen"
      class="fixed inset-0 bg-black/60 z-40 transition-opacity duration-300 motion-reduce:transition-none"
      @click="mobileMenuOpen = false"
      aria-hidden="true"
    />
    
    <!-- Main Content Area -->
    <div class="flex-1 flex flex-col min-w-0">
      <!-- Mobile Header -->
      <header
        v-if="isMobile"
        class="bg-gray-900 border-b border-gray-800 px-4 py-3 flex items-center gap-3 shrink-0"
      >
        <button
          @click="mobileMenuOpen = true"
          class="p-1.5 text-gray-400 hover:text-white transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
          aria-label="Open navigation menu"
          aria-expanded="false"
          aria-controls="mobile-nav"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
        </button>
        <span class="text-lg font-semibold text-cyan-400" role="img" aria-label="Cuttlefish logo">🐙</span>
        <span class="text-lg font-semibold text-cyan-400">Cuttlefish</span>
      </header>
      
      <!-- Page Content -->
      <main id="main-content" class="flex-1 overflow-auto" tabindex="-1">
        <slot />
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
const route = useRoute()
const { connected } = useWebSocket()

const mobileMenuOpen = ref(false)
const isMobile = ref(false)

// Handle responsive behavior
const checkMobile = () => {
  isMobile.value = window.innerWidth < 768
}

onMounted(() => {
  checkMobile()
  window.addEventListener('resize', checkMobile)
})

onUnmounted(() => {
  window.removeEventListener('resize', checkMobile)
})

// Close mobile menu on route change
watch(() => route.path, () => {
  mobileMenuOpen.value = false
})
</script>

<style scoped>
.nav-item {
  @apply flex items-center gap-3 px-3 py-2.5 rounded-lg text-gray-400 hover:text-white hover:bg-gray-800 transition-all duration-150 motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900;
}

.nav-item-active {
  @apply text-cyan-400 bg-cyan-950/30 hover:bg-cyan-950/50;
}

/* Ensure focus is visible for keyboard navigation */
:focus-visible {
  outline: 2px solid theme('colors.cyan.400');
  outline-offset: 2px;
}
</style>