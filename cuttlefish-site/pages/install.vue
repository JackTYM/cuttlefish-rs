<template>
  <div class="min-h-screen bg-slate-950 text-white">
    <!-- Hero Section -->
    <section class="py-16 px-4 border-b border-slate-800" aria-labelledby="install-heading">
      <div class="max-w-4xl mx-auto text-center">
        <h1 id="install-heading" class="text-5xl font-bold mb-4 bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
          Installation
        </h1>
        <p class="text-xl text-slate-400">
          Get Cuttlefish running in minutes
        </p>
      </div>
    </section>

    <!-- Prerequisites Section -->
    <section class="py-12 px-4 border-b border-slate-800" aria-labelledby="prerequisites-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="prerequisites-heading" class="text-2xl font-bold mb-6 text-cyan-400">Prerequisites</h2>
        <div class="grid md:grid-cols-2 gap-4" role="list">
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-4 flex items-start gap-3" role="listitem">
            <span class="text-2xl" aria-hidden="true">🦀</span>
            <div>
              <h3 class="font-semibold">Rust 1.94.0+</h3>
              <p class="text-sm text-slate-400">Install via rustup</p>
            </div>
          </div>
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-4 flex items-start gap-3" role="listitem">
            <span class="text-2xl" aria-hidden="true">🐳</span>
            <div>
              <h3 class="font-semibold">Docker</h3>
              <p class="text-sm text-slate-400">Running daemon with socket access</p>
            </div>
          </div>
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-4 flex items-start gap-3" role="listitem">
            <span class="text-2xl" aria-hidden="true">☁️</span>
            <div>
              <h3 class="font-semibold">AWS Account</h3>
              <p class="text-sm text-slate-400">With Bedrock access (or Claude OAuth)</p>
            </div>
          </div>
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-4 flex items-start gap-3" role="listitem">
            <span class="text-2xl" aria-hidden="true">📦</span>
            <div>
              <h3 class="font-semibold">Git</h3>
              <p class="text-sm text-slate-400">For cloning the repository</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Installation Methods Tabs -->
    <section class="py-12 px-4" aria-labelledby="methods-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="methods-heading" class="text-2xl font-bold mb-6 text-cyan-400">Installation Methods</h2>
        
        <!-- Tab Navigation -->
        <div class="flex flex-wrap gap-2 mb-6 border-b border-slate-800 pb-4" role="tablist" aria-label="Installation method options">
          <button
            v-for="tab in tabs"
            :key="tab.id"
            @click="activeTab = tab.id"
            :class="[
              'px-4 py-2 rounded-t font-medium transition-all motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
              activeTab === tab.id
                ? 'bg-slate-800 text-cyan-400 border-t border-l border-r border-slate-700'
                : 'text-slate-400 hover:text-white hover:bg-slate-900'
            ]"
            role="tab"
            :id="`tab-${tab.id}`"
            :aria-selected="activeTab === tab.id"
            :aria-controls="`panel-${tab.id}`"
            :tabindex="activeTab === tab.id ? 0 : -1"
          >
            {{ tab.label }}
          </button>
        </div>

        <!-- Tab Content -->
        <div class="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <!-- Quick Start Tab -->
          <div v-if="activeTab === 'quickstart'" class="space-y-6" role="tabpanel" id="panel-quickstart" aria-labelledby="tab-quickstart">
            <p class="text-slate-300 mb-4">
              The fastest way to get started with Cuttlefish. Clone, build, and run in under 5 minutes.
            </p>
            
            <TerminalBlock
              command="git clone https://github.com/JackTYM/cuttlefish-rs.git"
              comment="Clone the repository"
            />
            <TerminalBlock command="cd cuttlefish-rs" />
            <TerminalBlock
              command="cargo build --release"
              comment="Build release binary"
            />
            <TerminalBlock
              command="cp cuttlefish.example.toml cuttlefish.toml"
              comment="Copy example config"
            />
            <TerminalBlock
              command="export CUTTLEFISH_API_KEY=&quot;your-api-key&quot;"
              comment="Set API key for authentication"
            />
            <TerminalBlock
              command="export AWS_ACCESS_KEY_ID=&quot;your-aws-key&quot;"
              comment="AWS credentials for Bedrock"
            />
            <TerminalBlock
              command="export AWS_SECRET_ACCESS_KEY=&quot;your-aws-secret&quot;"
            />
            <TerminalBlock
              command="export AWS_DEFAULT_REGION=&quot;us-east-1&quot;"
            />
            <TerminalBlock
              command="./target/release/cuttlefish-rs"
              comment="Run the server"
            />
          </div>

          <!-- Docker Tab -->
          <div v-if="activeTab === 'docker'" class="space-y-6" role="tabpanel" id="panel-docker" aria-labelledby="tab-docker">
            <p class="text-slate-300 mb-4">
              Run Cuttlefish in a container for isolated, reproducible deployments.
            </p>
            
            <TerminalBlock
              command="docker build -t cuttlefish ."
              comment="Build the Docker image"
            />
            <TerminalBlock
              command="docker run -d \
  -v /var/run/docker.sock:/var/run/docker.sock \
  -v $(pwd)/cuttlefish.toml:/etc/cuttlefish/cuttlefish.toml \
  -e CUTTLEFISH_API_KEY=&quot;your-key&quot; \
  -e AWS_ACCESS_KEY_ID=&quot;...&quot; \
  -e AWS_SECRET_ACCESS_KEY=&quot;...&quot; \
  -p 8080:8080 \
  cuttlefish"
              comment="Run with configuration"
              :multiline="true"
            />
            
            <div class="mt-6 p-4 bg-slate-800 rounded border border-slate-700" role="note">
              <p class="text-sm text-slate-300">
                <strong class="text-cyan-400">Note:</strong> The Docker socket mount is required for sandbox containers.
              </p>
            </div>
          </div>

          <!-- Guided Install Tab -->
          <div v-if="activeTab === 'guided'" class="space-y-6" role="tabpanel" id="panel-guided" aria-labelledby="tab-guided">
            <p class="text-slate-300 mb-4">
              Recommended for production deployments. The guided installer handles everything automatically.
            </p>
            
            <TerminalBlock
              command="curl -sSL https://raw.githubusercontent.com/JackTYM/cuttlefish-rs/master/install.sh | sudo bash"
              comment="Download and run the guided installer"
            />
            
            <div class="mt-8">
              <h3 class="text-lg font-semibold mb-4 text-white">The installer will:</h3>
              <ul class="space-y-3" role="list">
                <li class="flex items-start gap-3">
                  <span class="text-cyan-400 mt-1" aria-hidden="true">✓</span>
                  <span class="text-slate-300">Check and install dependencies (Rust, Docker, Git)</span>
                </li>
                <li class="flex items-start gap-3">
                  <span class="text-cyan-400 mt-1" aria-hidden="true">✓</span>
                  <span class="text-slate-300">Guide you through server, database, and sandbox configuration</span>
                </li>
                <li class="flex items-start gap-3">
                  <span class="text-cyan-400 mt-1" aria-hidden="true">✓</span>
                  <span class="text-slate-300">Set up AWS Bedrock credentials</span>
                </li>
                <li class="flex items-start gap-3">
                  <span class="text-cyan-400 mt-1" aria-hidden="true">✓</span>
                  <span class="text-slate-300">Optionally configure Discord bot integration</span>
                </li>
                <li class="flex items-start gap-3">
                  <span class="text-cyan-400 mt-1" aria-hidden="true">✓</span>
                  <span class="text-slate-300">Create a systemd service for 24/7 operation</span>
                </li>
                <li class="flex items-start gap-3">
                  <span class="text-cyan-400 mt-1" aria-hidden="true">✓</span>
                  <span class="text-slate-300">Generate secure API keys</span>
                </li>
              </ul>
            </div>
          </div>

          <!-- From Source Tab -->
          <div v-if="activeTab === 'source'" class="space-y-6" role="tabpanel" id="panel-source" aria-labelledby="tab-source">
            <p class="text-slate-300 mb-4">
              For developers who want to contribute or customize Cuttlefish.
            </p>
            
            <TerminalBlock
              command="git clone https://github.com/JackTYM/cuttlefish-rs.git"
              comment="Clone the repository"
            />
            <TerminalBlock command="cd cuttlefish-rs" />
            <TerminalBlock
              command="cargo build --workspace"
              comment="Build all crates"
            />
            <TerminalBlock
              command="cargo test --workspace"
              comment="Run tests"
            />
            <TerminalBlock
              command="cargo run"
              comment="Run in development mode"
            />
            
            <div class="mt-6 p-4 bg-slate-800 rounded border border-slate-700" role="note">
              <p class="text-sm text-slate-300">
                <strong class="text-cyan-400">Development:</strong> Use <code class="bg-slate-900 px-1 rounded">cargo clippy --workspace -- -D warnings</code> to check code quality.
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Configuration Section -->
    <section class="py-12 px-4 border-t border-slate-800" aria-labelledby="config-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="config-heading" class="text-2xl font-bold mb-6 text-cyan-400">Configuration</h2>
        
        <div class="bg-slate-900 border border-slate-800 rounded-lg p-6 mb-6">
          <h3 id="env-vars-heading" class="text-lg font-semibold mb-4">Environment Variables</h3>
          <div class="overflow-x-auto">
            <table class="w-full text-sm" aria-labelledby="env-vars-heading">
              <thead>
                <tr class="border-b border-slate-700">
                  <th scope="col" class="text-left py-2 pr-4 text-slate-400">Variable</th>
                  <th scope="col" class="text-left py-2 text-slate-400">Description</th>
                </tr>
              </thead>
              <tbody class="text-slate-300">
                <tr class="border-b border-slate-800">
                  <td class="py-3 pr-4 font-mono text-cyan-400">CUTTLEFISH_API_KEY</td>
                  <td class="py-3">API key for WebUI/TUI authentication (required)</td>
                </tr>
                <tr class="border-b border-slate-800">
                  <td class="py-3 pr-4 font-mono text-cyan-400">AWS_ACCESS_KEY_ID</td>
                  <td class="py-3">AWS access key for Bedrock</td>
                </tr>
                <tr class="border-b border-slate-800">
                  <td class="py-3 pr-4 font-mono text-cyan-400">AWS_SECRET_ACCESS_KEY</td>
                  <td class="py-3">AWS secret key for Bedrock</td>
                </tr>
                <tr class="border-b border-slate-800">
                  <td class="py-3 pr-4 font-mono text-cyan-400">AWS_DEFAULT_REGION</td>
                  <td class="py-3">AWS region (e.g., us-east-1)</td>
                </tr>
                <tr>
                  <td class="py-3 pr-4 font-mono text-cyan-400">DISCORD_BOT_TOKEN</td>
                  <td class="py-3">Discord bot token (optional)</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>

        <div class="flex items-center gap-4">
          <a
            href="/docs/configuration"
            class="inline-flex items-center gap-2 text-cyan-400 hover:text-cyan-300 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
          >
            <span>View full configuration reference</span>
            <span aria-hidden="true">→</span>
          </a>
        </div>
      </div>
    </section>

    <!-- Troubleshooting FAQ -->
    <section class="py-12 px-4 border-t border-slate-800" aria-labelledby="troubleshooting-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="troubleshooting-heading" class="text-2xl font-bold mb-6 text-cyan-400">Troubleshooting</h2>
        
        <div class="space-y-3">
          <FAQItem
            question="Docker socket permission denied"
            answer="Add your user to the docker group: sudo usermod -aG docker $USER, then log out and back in."
          />
          <FAQItem
            question="AWS credentials not found"
            answer="Ensure AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY are set in your environment. Verify with: echo $AWS_ACCESS_KEY_ID"
          />
          <FAQItem
            question="Port 8080 already in use"
            answer="Change the port in cuttlefish.toml under [server].port, or stop the conflicting service."
          />
          <FAQItem
            question="Build fails with Rust version error"
            answer="Install Rust 1.94.0+: rustup install 1.94.0 && rustup default 1.94.0"
          />
          <FAQItem
            question="Bedrock access denied"
            answer="Ensure your AWS account has Bedrock access enabled. Request access in the AWS console under Bedrock service."
          />
        </div>
      </div>
    </section>

    <!-- Bottom CTA -->
    <section class="py-16 px-4 border-t border-slate-800 bg-slate-900/50" aria-labelledby="help-heading">
      <div class="max-w-4xl mx-auto text-center">
        <h2 id="help-heading" class="text-2xl font-bold mb-4">Need Help?</h2>
        <p class="text-slate-400 mb-8">Get support from the community or the maintainers</p>
        <div class="flex flex-wrap justify-center gap-4">
          <a
            href="https://discord.gg/cuttlefish"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-2 bg-indigo-600 hover:bg-indigo-700 px-6 py-3 rounded-lg font-semibold transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
          >
            <span aria-hidden="true">💬</span>
            <span>Join Discord</span>
            <span class="sr-only">(opens in new tab)</span>
          </a>
          <a
            href="https://github.com/JackTYM/cuttlefish-rs/issues"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-2 bg-slate-700 hover:bg-slate-600 px-6 py-3 rounded-lg font-semibold transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
          >
            <span aria-hidden="true">🐛</span>
            <span>GitHub Issues</span>
            <span class="sr-only">(opens in new tab)</span>
          </a>
          <a
            href="/docs"
            class="inline-flex items-center gap-2 bg-slate-700 hover:bg-slate-600 px-6 py-3 rounded-lg font-semibold transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
          >
            <span aria-hidden="true">📚</span>
            <span>Documentation</span>
          </a>
        </div>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

definePageMeta({
  layout: 'default'
})

const tabs = [
  { id: 'quickstart', label: 'Quick Start' },
  { id: 'docker', label: 'Docker' },
  { id: 'guided', label: 'Guided Install' },
  { id: 'source', label: 'From Source' }
]

const activeTab = ref('quickstart')
</script>

<style scoped>
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