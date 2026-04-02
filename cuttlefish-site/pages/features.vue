<template>
  <div class="min-h-screen bg-slate-950">
    <!-- Hero Section -->
    <section class="pt-24 pb-16 px-4">
      <div class="max-w-7xl mx-auto text-center">
        <h1 class="text-5xl md:text-6xl font-bold mb-6 bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
          Features
        </h1>
        <p class="text-xl md:text-2xl text-slate-400 max-w-3xl mx-auto">
          Everything you need to build with AI agents. Multi-model routing, isolated sandboxes, and full workflow automation.
        </p>
      </div>
    </section>

    <!-- Feature Grid -->
    <section class="py-16 px-4">
      <div class="max-w-7xl mx-auto">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Multi-Agent System -->
          <FeatureCard
            icon="🤖"
            title="Multi-Agent System"
            :description="[
              'Orchestrator → Coder → Critic workflow loop',
              'Specialized agents for planning, coding, and review',
              'Each agent uses its optimal model category'
            ]"
          >
            <CodeBlock :lines="[
              'Task arrives → Orchestrator assesses',
              '         ↓',
              '┌─────────┬─────────┬─────────┐',
              '│ Planner │  Coder  │ Critic  │',
              '│ultrabrain│  deep  │  high   │',
              '└─────────┴─────────┴─────────┘',
              '         ↓',
              'Different models, different strengths'
            ]" />
          </FeatureCard>

          <!-- Multi-Model Routing -->
          <FeatureCard
            icon="🎯"
            title="Category-Based Model Routing"
            :description="[
              'Different models for different tasks',
              'Fast models for quick tasks, powerful for complex reasoning',
              'Configure routing per-agent or per-category'
            ]"
          >
            <ModelTable />
          </FeatureCard>

          <!-- Docker Sandboxes -->
          <FeatureCard
            icon="📦"
            title="Docker Sandboxes"
            :description="[
              'Isolated execution environment per project',
              'Configurable CPU, memory, and disk limits',
              'Automatic cleanup of stale containers'
            ]"
          >
            <CodeBlock :lines="[
              '[sandbox]',
              'memory_limit_mb = 2048',
              'cpu_limit = 2.0',
              'disk_limit_gb = 10',
              'max_concurrent = 5'
            ]" language="toml" />
          </FeatureCard>

          <!-- Multi-Interface -->
          <FeatureCard
            icon="🖥️"
            title="Multi-Interface Access"
            :description="[
              'Discord Bot with slash commands',
              'Web UI with chat, logs, and diffs',
              'TUI for SSH sessions'
            ]"
          >
            <div class="flex gap-4 justify-center">
              <div class="bg-slate-800 px-4 py-2 rounded-lg border border-slate-700">
                <span class="text-purple-400 font-mono text-sm">Discord</span>
              </div>
              <div class="bg-slate-800 px-4 py-2 rounded-lg border border-slate-700">
                <span class="text-cyan-400 font-mono text-sm">WebUI</span>
              </div>
              <div class="bg-slate-800 px-4 py-2 rounded-lg border border-slate-700">
                <span class="text-green-400 font-mono text-sm">TUI</span>
              </div>
            </div>
          </FeatureCard>

          <!-- Hashline Editing -->
          <FeatureCard
            icon="✂️"
            title="Hashline Editing"
            :description="[
              'Surgical file edits using content-addressable hashes',
              '5-char xxHash for precise line identification',
              'No more line number drift'
            ]"
          >
            <CodeBlock :lines="hashlineLines" />
          </FeatureCard>

          <!-- GitHub Integration -->
          <FeatureCard
            icon="🐙"
            title="GitHub Integration"
            :description="[
              'Clone, branch, commit, push',
              'Create pull requests automatically',
              'Monitor GitHub Actions workflows'
            ]"
          >
            <CodeBlock :lines="[
              'git clone → branch → code → commit',
              '         ↓',
              '    Create PR',
              '         ↓',
              '    Monitor Actions',
              '         ↓',
              '    Merge on green'
            ]" />
          </FeatureCard>

          <!-- Self-Developing -->
          <FeatureCard
            icon="🔄"
            title="Self-Developing"
            :description="[
              'Updates itself via GitHub Actions',
              'Cuttlefish can fix its own bugs',
              'Pull new binaries and restart automatically'
            ]"
          >
            <div class="flex items-center justify-center gap-2 text-slate-400">
              <span class="text-2xl">🐛</span>
              <svg class="w-6 h-6 text-cyan-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 7l5 5m0 0l-5 5m5-5H6" />
              </svg>
              <span class="text-2xl">✅</span>
            </div>
          </FeatureCard>

          <!-- Template System -->
          <FeatureCard
            icon="🛒"
            title="Template Marketplace"
            :description="[
              'Start from proven templates',
              'Community-contributed project starters',
              'Markdown-based scaffolding'
            ]"
          >
            <div class="text-center text-slate-500 text-sm">
              Coming in v1.1
            </div>
          </FeatureCard>
        </div>
      </div>
    </section>

    <!-- Comparison Table -->
    <section class="py-16 px-4 bg-slate-900/50">
      <div class="max-w-5xl mx-auto">
        <h2 class="text-3xl md:text-4xl font-bold text-center mb-12">
          Why Cuttlefish?
        </h2>
        
        <div class="overflow-x-auto">
          <table class="w-full">
            <thead>
              <tr class="border-b border-slate-700">
                <th class="text-left py-4 px-4 text-slate-400 font-medium">Feature</th>
                <th class="py-4 px-4 text-cyan-400 font-semibold">Cuttlefish</th>
                <th class="py-4 px-4 text-slate-400">Cursor</th>
                <th class="py-4 px-4 text-slate-400">GitHub Copilot</th>
              </tr>
            </thead>
            <tbody class="text-center">
              <tr class="border-b border-slate-800">
                <td class="text-left py-4 px-4 text-slate-300">Multi-Agent System</td>
                <td class="py-4 px-4"><span class="text-green-400 text-xl">✓</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="text-left py-4 px-4 text-slate-300">Multi-Model Routing</td>
                <td class="py-4 px-4"><span class="text-green-400 text-xl">✓</span></td>
                <td class="py-4 px-4"><span class="text-yellow-400 text-sm">Partial</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="text-left py-4 px-4 text-slate-300">Self-Hosted</td>
                <td class="py-4 px-4"><span class="text-green-400 text-xl">✓</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="text-left py-4 px-4 text-slate-300">Discord Bot</td>
                <td class="py-4 px-4"><span class="text-green-400 text-xl">✓</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="text-left py-4 px-4 text-slate-300">Docker Sandboxes</td>
                <td class="py-4 px-4"><span class="text-green-400 text-xl">✓</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="text-left py-4 px-4 text-slate-300">Open Source</td>
                <td class="py-4 px-4"><span class="text-green-400 text-xl">✓</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
              </tr>
              <tr>
                <td class="text-left py-4 px-4 text-slate-300">Self-Developing</td>
                <td class="py-4 px-4"><span class="text-green-400 text-xl">✓</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
                <td class="py-4 px-4"><span class="text-slate-600">—</span></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </section>

    <!-- CTA Section -->
    <section class="py-24 px-4">
      <div class="max-w-3xl mx-auto text-center">
        <h2 class="text-3xl md:text-4xl font-bold mb-6">
          Ready to dive in?
        </h2>
        <p class="text-xl text-slate-400 mb-8">
          Get Cuttlefish running in minutes with our guided installer.
        </p>
        <NuxtLink 
          to="/install"
          class="inline-block bg-cyan-600 hover:bg-cyan-700 px-8 py-4 rounded-lg font-semibold text-lg transition transform hover:scale-105"
        >
          Get Started →
        </NuxtLink>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
definePageMeta({
  layout: 'default'
})

// Code block content for hashline editing
const hashlineLines = [
  '# Original file',
  'abcde│ fn main() {',
  'fghij│     println!("Hello");',
  '',
  '# Edit by hash',
  'replace fghij with:',
  '    println!("Hello, World!");'
]
</script>