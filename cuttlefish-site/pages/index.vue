<template>
  <div class="min-h-screen bg-slate-950 text-white relative overflow-hidden">
    <!-- Subtle grid background -->
    <div class="absolute inset-0 bg-grid-pattern opacity-[0.03]" />
    
    <!-- Gradient overlay -->
    <div class="absolute inset-0 bg-gradient-to-b from-cyan-500/5 via-transparent to-transparent" />
    
    <!-- Hero Section -->
    <section class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-16 pb-24 lg:pt-24 lg:pb-32">
      <div class="grid lg:grid-cols-2 gap-12 lg:gap-16 items-center">
        <!-- Left: Content -->
        <div class="text-center lg:text-left">
          <!-- Logo/Emoji -->
          <div class="text-6xl mb-6 animate-float">🐙</div>
          
          <!-- Headline -->
          <h1 class="text-5xl sm:text-6xl lg:text-7xl font-bold mb-4 tracking-tight">
            <span class="bg-gradient-to-r from-white via-cyan-100 to-cyan-400 bg-clip-text text-transparent">
              Cuttlefish
            </span>
          </h1>
          
          <!-- Tagline -->
          <p class="text-2xl sm:text-3xl text-slate-300 font-medium mb-4">
            Multi-Agent AI Coding Platform
          </p>
          
          <!-- Subheadline -->
          <p class="text-lg sm:text-xl text-slate-400 mb-8 max-w-xl mx-auto lg:mx-0">
            Portable. Multi-model. Self-developing. Built in Rust.
          </p>
          
          <!-- CTA Buttons -->
          <div class="flex flex-col sm:flex-row gap-4 justify-center lg:justify-start">
            <NuxtLink 
              to="/install"
              class="inline-flex items-center justify-center px-6 py-3 bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-semibold rounded-lg transition-all duration-200 hover:shadow-lg hover:shadow-cyan-500/25 hover:-translate-y-0.5"
            >
              Get Started
              <svg class="w-5 h-5 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
            </NuxtLink>
            
            <a 
              href="https://github.com/JackTYM/cuttlefish-rs"
              target="_blank"
              rel="noopener noreferrer"
              class="inline-flex items-center justify-center px-6 py-3 border border-slate-700 hover:border-cyan-500/50 text-slate-300 hover:text-white font-semibold rounded-lg transition-all duration-200 hover:bg-slate-800/50"
            >
              <svg class="w-5 h-5 mr-2" fill="currentColor" viewBox="0 0 24 24">
                <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z"/>
              </svg>
              View Source
            </a>
          </div>
        </div>
        
        <!-- Right: Terminal Demo -->
        <div class="relative">
          <!-- Glow effect behind terminal -->
          <div class="absolute -inset-4 bg-cyan-500/10 blur-3xl rounded-full" />
          
          <!-- Terminal -->
          <TerminalDemo class="relative" />
        </div>
      </div>
    </section>
    
    <!-- Features Section -->
    <section id="features" class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-24">
      <!-- Section header -->
      <div class="text-center mb-16">
        <h2 class="text-3xl sm:text-4xl font-bold mb-4">
          Why <span class="text-cyan-400">Cuttlefish</span>?
        </h2>
        <p class="text-slate-400 text-lg max-w-2xl mx-auto">
          Nature's ultimate adapter, now in code. Multiple specialized agents working in concert, 
          each with its own expertise and optimal model.
        </p>
      </div>
      
      <!-- Feature cards grid -->
      <div class="grid sm:grid-cols-2 lg:grid-cols-4 gap-6">
        <!-- Multi-Agent System -->
        <div class="feature-card group p-6 rounded-xl bg-slate-900/50 border border-slate-800 hover:border-cyan-500/30 transition-all duration-300 hover:-translate-y-1">
          <div class="text-4xl mb-4">🤖</div>
          <h3 class="text-xl font-semibold mb-2 text-white group-hover:text-cyan-400 transition-colors">
            Multi-Agent System
          </h3>
          <p class="text-slate-400 text-sm leading-relaxed">
            Orchestrator → Coder → Critic workflow. Each agent has its own expertise, tools, and optimal model.
          </p>
        </div>
        
        <!-- Multi-Model Routing -->
        <div class="feature-card group p-6 rounded-xl bg-slate-900/50 border border-slate-800 hover:border-cyan-500/30 transition-all duration-300 hover:-translate-y-1">
          <div class="text-4xl mb-4">🎯</div>
          <h3 class="text-xl font-semibold mb-2 text-white group-hover:text-cyan-400 transition-colors">
            Multi-Model Routing
          </h3>
          <p class="text-slate-400 text-sm leading-relaxed">
            Right model for each task. Fast models for quick tasks, powerful models for deep reasoning.
          </p>
        </div>
        
        <!-- Docker Sandboxes -->
        <div class="feature-card group p-6 rounded-xl bg-slate-900/50 border border-slate-800 hover:border-cyan-500/30 transition-all duration-300 hover:-translate-y-1">
          <div class="text-4xl mb-4">📦</div>
          <h3 class="text-xl font-semibold mb-2 text-white group-hover:text-cyan-400 transition-colors">
            Docker Sandboxes
          </h3>
          <p class="text-slate-400 text-sm leading-relaxed">
            Isolated, secure execution. Every project runs in its own container—no cross-contamination.
          </p>
        </div>
        
        <!-- Self-Developing -->
        <div class="feature-card group p-6 rounded-xl bg-slate-900/50 border border-slate-800 hover:border-cyan-500/30 transition-all duration-300 hover:-translate-y-1">
          <div class="text-4xl mb-4">🔄</div>
          <h3 class="text-xl font-semibold mb-2 text-white group-hover:text-cyan-400 transition-colors">
            Self-Developing
          </h3>
          <p class="text-slate-400 text-sm leading-relaxed">
            Updates itself via GitHub Actions. True self-evolving software that fixes its own bugs.
          </p>
        </div>
      </div>
    </section>
    
    <!-- Philosophy Section -->
    <section class="relative z-10 max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-24 border-t border-slate-800/50">
      <div class="grid lg:grid-cols-2 gap-12 items-center">
        <div>
          <h2 class="text-3xl sm:text-4xl font-bold mb-6">
            The <span class="text-cyan-400">Cuttlefish</span> Way
          </h2>
          <div class="space-y-4 text-slate-300">
            <p>
              The cuttlefish is nature's ultimate adapter. With millions of chromatophores in their skin, 
              cuttlefish can change color, pattern, and texture in milliseconds.
            </p>
            <p>
              <strong class="text-white">Multi-Colored = Multi-Agent.</strong> Just as a cuttlefish displays 
              many colors simultaneously through specialized chromatophores, Cuttlefish deploys multiple 
              specialized agents working in concert.
            </p>
            <p>
              <strong class="text-white">Rapid Adaptation = Dynamic Model Routing.</strong> A cuttlefish 
              switches from camouflage to warning display instantly. Cuttlefish switches between fast models 
              for quick tasks and powerful models for deep reasoning.
            </p>
          </div>
        </div>
        
        <!-- ASCII Art Diagram -->
        <div class="bg-slate-900/50 rounded-xl border border-slate-800 p-6 font-mono text-sm text-slate-400 overflow-x-auto">
          <pre class="whitespace-pre">
    🐙 The Cuttlefish Way
    
    Task arrives → Orchestrator assesses → Routes to optimal agent
                                              ↓
    ┌─────────────┬─────────────┬─────────────┬─────────────┐
    │   Planner   │    Coder    │   Critic    │   DevOps    │
    │  ultrabrain │    deep     │    high     │    high     │
    │  "strategy" │   "build"   │  "review"   │  "deploy"   │
    └─────────────┴─────────────┴─────────────┴─────────────┘
                              ↓
    Different models, different strengths, one unified system</pre>
        </div>
      </div>
    </section>
    
    <!-- CTA Section -->
    <section class="relative z-10 max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-24 text-center">
      <h2 class="text-3xl sm:text-4xl font-bold mb-4">
        Ready to dive in?
      </h2>
      <p class="text-slate-400 text-lg mb-8">
        Get started with Cuttlefish in minutes.
      </p>
      <NuxtLink 
        to="/install"
        class="inline-flex items-center justify-center px-8 py-4 bg-cyan-500 hover:bg-cyan-400 text-slate-950 font-semibold rounded-lg transition-all duration-200 hover:shadow-lg hover:shadow-cyan-500/25 hover:-translate-y-0.5 text-lg"
      >
        Get Started
        <svg class="w-5 h-5 ml-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
        </svg>
      </NuxtLink>
    </section>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'default'
})
</script>

<style scoped>
/* Grid background pattern */
.bg-grid-pattern {
  background-image: 
    linear-gradient(to right, rgba(255, 255, 255, 0.05) 1px, transparent 1px),
    linear-gradient(to bottom, rgba(255, 255, 255, 0.05) 1px, transparent 1px);
  background-size: 40px 40px;
}

/* Floating animation for emoji */
@keyframes float {
  0%, 100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

.animate-float {
  animation: float 3s ease-in-out infinite;
}

/* Feature card hover glow */
.feature-card:hover {
  box-shadow: 0 0 40px -10px rgba(34, 211, 238, 0.15);
}

/* Smooth scroll */
html {
  scroll-behavior: smooth;
}
</style>