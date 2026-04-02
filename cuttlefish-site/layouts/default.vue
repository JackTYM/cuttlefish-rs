<template>
  <div class="min-h-screen bg-slate-950 text-white">
    <!-- Sticky Header -->
    <header class="sticky top-0 z-50 bg-slate-950/95 backdrop-blur border-b border-slate-800">
      <nav class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center justify-between">
        <!-- Logo -->
        <div class="flex items-center gap-2 text-2xl font-bold">
          <span>🐙</span>
          <span>Cuttlefish</span>
        </div>

        <!-- Desktop Nav Items -->
        <div class="hidden md:flex items-center gap-8">
          <NuxtLink to="/features" class="hover:text-cyan-400 transition">Features</NuxtLink>
          <NuxtLink to="/install" class="hover:text-cyan-400 transition">Install</NuxtLink>
          <NuxtLink to="/docs" class="hover:text-cyan-400 transition">Docs</NuxtLink>
          <NuxtLink to="/marketplace" class="hover:text-cyan-400 transition">Marketplace</NuxtLink>
        </div>

        <!-- Desktop CTA Button -->
        <a
          href="https://app.cuttlefish.dev"
          target="_blank"
          rel="noopener noreferrer"
          class="hidden md:inline-flex bg-cyan-600 hover:bg-cyan-700 px-4 py-2 rounded font-semibold transition"
        >
          Launch App →
        </a>

        <!-- Mobile Menu Button -->
        <button
          @click="mobileMenuOpen = !mobileMenuOpen"
          class="md:hidden p-2 text-gray-400 hover:text-white transition-colors"
          aria-label="Toggle menu"
        >
          <svg v-if="!mobileMenuOpen" class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
          <svg v-else class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </nav>

      <!-- Mobile Menu -->
      <Transition
        enter-active-class="transition ease-out duration-200"
        enter-from-class="opacity-0 -translate-y-2"
        enter-to-class="opacity-100 translate-y-0"
        leave-active-class="transition ease-in duration-150"
        leave-from-class="opacity-100 translate-y-0"
        leave-to-class="opacity-0 -translate-y-2"
      >
        <div v-if="mobileMenuOpen" class="md:hidden border-t border-slate-800 bg-slate-950">
          <div class="px-4 py-4 space-y-3">
            <NuxtLink 
              to="/features" 
              class="block py-2 text-gray-300 hover:text-cyan-400 transition"
              @click="mobileMenuOpen = false"
            >
              Features
            </NuxtLink>
            <NuxtLink 
              to="/install" 
              class="block py-2 text-gray-300 hover:text-cyan-400 transition"
              @click="mobileMenuOpen = false"
            >
              Install
            </NuxtLink>
            <NuxtLink 
              to="/docs" 
              class="block py-2 text-gray-300 hover:text-cyan-400 transition"
              @click="mobileMenuOpen = false"
            >
              Docs
            </NuxtLink>
            <NuxtLink 
              to="/marketplace" 
              class="block py-2 text-gray-300 hover:text-cyan-400 transition"
              @click="mobileMenuOpen = false"
            >
              Marketplace
            </NuxtLink>
            <div class="pt-3 border-t border-slate-800">
              <a
                href="https://app.cuttlefish.dev"
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center justify-center w-full bg-cyan-600 hover:bg-cyan-700 px-4 py-3 rounded font-semibold transition"
              >
                Launch App →
              </a>
            </div>
          </div>
        </div>
      </Transition>
    </header>

    <!-- Page Content -->
    <main>
      <slot />
    </main>
  </div>
</template>

<script setup lang="ts">
const mobileMenuOpen = ref(false)

// Close menu on route change
const route = useRoute()
watch(() => route.path, () => {
  mobileMenuOpen.value = false
})
</script>

<style scoped>
/* Layout styles */
</style>
