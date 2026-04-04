<template>
  <div class="min-h-screen bg-slate-950 text-white">
    <!-- Skip to main content link -->
    <a
      href="#main-content"
      class="sr-only focus:not-sr-only focus:absolute focus:z-[100] focus:top-4 focus:left-4 focus:px-4 focus:py-2 focus:bg-cyan-600 focus:text-white focus:rounded-lg focus:outline-none focus:ring-2 focus:ring-cyan-400"
    >
      Skip to main content
    </a>

    <!-- Sticky Header -->
    <header class="sticky top-0 z-50 bg-slate-950/95 backdrop-blur border-b border-slate-800" role="banner">
      <nav class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 flex items-center justify-between" aria-label="Main navigation">
        <!-- Logo -->
        <NuxtLink to="/" class="flex items-center gap-2 text-2xl font-bold focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950 rounded">
          <span role="img" aria-label="Cuttlefish logo">🐙</span>
          <span>Cuttlefish</span>
        </NuxtLink>

        <!-- Desktop Nav Items -->
        <div class="hidden md:flex items-center gap-8">
          <NuxtLink to="/features" class="hover:text-cyan-400 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded">Features</NuxtLink>
          <NuxtLink to="/install" class="hover:text-cyan-400 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded">Install</NuxtLink>
          <NuxtLink to="/docs" class="hover:text-cyan-400 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded">Docs</NuxtLink>
          <NuxtLink to="/marketplace" class="hover:text-cyan-400 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded">Marketplace</NuxtLink>
        </div>

        <!-- Desktop CTA Button -->
        <a
          href="https://app.cuttlefish.dev"
          target="_blank"
          rel="noopener noreferrer"
          class="hidden md:inline-flex bg-cyan-600 hover:bg-cyan-700 px-4 py-2 rounded font-semibold transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-950"
        >
          Launch App →
          <span class="sr-only">(opens in new tab)</span>
        </a>

        <!-- Mobile Menu Button -->
        <button
          @click="mobileMenuOpen = !mobileMenuOpen"
          class="md:hidden p-2 text-gray-400 hover:text-white transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
          :aria-expanded="mobileMenuOpen"
          aria-controls="mobile-menu"
          aria-label="Toggle navigation menu"
        >
          <svg v-if="!mobileMenuOpen" class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
          </svg>
          <svg v-else class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
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
        <div v-if="mobileMenuOpen" id="mobile-menu" class="md:hidden border-t border-slate-800 bg-slate-950">
          <nav class="px-4 py-4 space-y-3" aria-label="Mobile navigation">
            <NuxtLink 
              to="/features" 
              class="block py-2 text-gray-300 hover:text-cyan-400 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
              @click="mobileMenuOpen = false"
            >
              Features
            </NuxtLink>
            <NuxtLink 
              to="/install" 
              class="block py-2 text-gray-300 hover:text-cyan-400 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
              @click="mobileMenuOpen = false"
            >
              Install
            </NuxtLink>
            <NuxtLink 
              to="/docs" 
              class="block py-2 text-gray-300 hover:text-cyan-400 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
              @click="mobileMenuOpen = false"
            >
              Docs
            </NuxtLink>
            <NuxtLink 
              to="/marketplace" 
              class="block py-2 text-gray-300 hover:text-cyan-400 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
              @click="mobileMenuOpen = false"
            >
              Marketplace
            </NuxtLink>
            <div class="pt-3 border-t border-slate-800">
              <a
                href="https://app.cuttlefish.dev"
                target="_blank"
                rel="noopener noreferrer"
                class="inline-flex items-center justify-center w-full bg-cyan-600 hover:bg-cyan-700 px-4 py-3 rounded font-semibold transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
              >
                Launch App →
                <span class="sr-only">(opens in new tab)</span>
              </a>
            </div>
          </nav>
        </div>
      </Transition>
    </header>

    <!-- Page Content -->
    <main id="main-content" tabindex="-1">
      <slot />
    </main>

    <!-- Footer -->
    <footer class="border-t border-slate-800 py-8 px-4" role="contentinfo">
      <div class="max-w-7xl mx-auto text-center text-slate-500 text-sm">
        <p>© {{ new Date().getFullYear() }} Cuttlefish. Open source under MIT License.</p>
      </div>
    </footer>
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

/* Screen reader only utility */
.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border-width: 0;
}

/* Ensure focus is visible for keyboard navigation */
:focus-visible {
  outline: 2px solid theme('colors.cyan.400');
  outline-offset: 2px;
}

/* Prefers reduced motion support */
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
</style>
