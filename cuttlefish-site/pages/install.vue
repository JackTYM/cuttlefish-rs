<template>
  <div class="min-h-screen bg-slate-950 text-white">
    <!-- Hero Section -->
    <section class="py-16 px-4 border-b border-slate-800" aria-labelledby="install-heading">
      <div class="max-w-4xl mx-auto text-center">
        <h1 id="install-heading" class="text-5xl font-bold mb-4 bg-gradient-to-r from-cyan-400 to-purple-400 bg-clip-text text-transparent">
          Installation
        </h1>
        <p class="text-xl text-slate-400">
          Get Cuttlefish running in minutes with our guided installer
        </p>
      </div>
    </section>

    <!-- Quick Start Section -->
    <section class="py-12 px-4 border-b border-slate-800" aria-labelledby="quickstart-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="quickstart-heading" class="text-2xl font-bold mb-6 text-cyan-400 flex items-center gap-3">
          <span class="text-3xl" aria-hidden="true">⚡</span>
          Quick Start
        </h2>
        <p class="text-slate-300 mb-6">
          The fastest way to get started. Run this one-liner to launch the guided installer:
        </p>
        
        <!-- One-liner Install Command -->
        <div class="relative group mb-8">
          <div class="bg-slate-950 border border-cyan-900/50 rounded-lg overflow-hidden">
            <div class="flex items-center gap-2 px-4 py-2 bg-slate-900 border-b border-slate-700">
              <div class="flex gap-1.5">
                <div class="w-3 h-3 rounded-full bg-red-500/80"></div>
                <div class="w-3 h-3 rounded-full bg-yellow-500/80"></div>
                <div class="w-3 h-3 rounded-full bg-green-500/80"></div>
              </div>
              <span class="text-xs text-slate-500 ml-2">terminal</span>
            </div>
            <div class="p-4 font-mono text-sm flex items-center gap-3">
              <span class="text-green-400 select-none">$</span>
              <code class="flex-1 text-cyan-300">curl -sSL https://raw.githubusercontent.com/JackTYM/cuttlefish-rs/master/install.sh | bash</code>
            </div>
          </div>
          <button
            @click="copyInstallCommand"
            class="absolute top-10 right-2 sm:right-4 px-3 py-2.5 sm:py-1.5 min-h-[44px] sm:min-h-0 rounded bg-cyan-600 hover:bg-cyan-500 text-sm font-medium transition-all flex items-center gap-2"
            :title="installCopied ? 'Copied!' : 'Copy command'"
          >
            <svg v-if="!installCopied" class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
            </svg>
            <svg v-else class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
            </svg>
            {{ installCopied ? 'Copied!' : 'Copy' }}
          </button>
        </div>

        <!-- What the installer does -->
        <div class="bg-slate-900/50 border border-slate-800 rounded-lg p-6">
          <h3 class="text-lg font-semibold mb-4 text-white">The installer will:</h3>
          <ul class="grid md:grid-cols-2 gap-3" role="list">
            <li class="flex items-start gap-3">
              <span class="text-cyan-400 mt-0.5" aria-hidden="true">✓</span>
              <span class="text-slate-300">Detect your platform (Linux, macOS, WSL)</span>
            </li>
            <li class="flex items-start gap-3">
              <span class="text-cyan-400 mt-0.5" aria-hidden="true">✓</span>
              <span class="text-slate-300">Install dependencies (Rust, Docker, Git)</span>
            </li>
            <li class="flex items-start gap-3">
              <span class="text-cyan-400 mt-0.5" aria-hidden="true">✓</span>
              <span class="text-slate-300">Guide provider selection & API key setup</span>
            </li>
            <li class="flex items-start gap-3">
              <span class="text-cyan-400 mt-0.5" aria-hidden="true">✓</span>
              <span class="text-slate-300">Configure server, database, sandbox</span>
            </li>
            <li class="flex items-start gap-3">
              <span class="text-cyan-400 mt-0.5" aria-hidden="true">✓</span>
              <span class="text-slate-300">Build and install from source</span>
            </li>
            <li class="flex items-start gap-3">
              <span class="text-cyan-400 mt-0.5" aria-hidden="true">✓</span>
              <span class="text-slate-300">Set up systemd service (optional)</span>
            </li>
          </ul>
        </div>
      </div>
    </section>

    <!-- Prerequisites Section -->
    <section class="py-12 px-4 border-b border-slate-800" aria-labelledby="prerequisites-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="prerequisites-heading" class="text-2xl font-bold mb-6 text-cyan-400 flex items-center gap-3">
          <span class="text-3xl" aria-hidden="true">📋</span>
          Prerequisites
        </h2>
        <div class="grid md:grid-cols-2 gap-4" role="list">
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-4 flex items-start gap-3" role="listitem">
            <span class="text-2xl" aria-hidden="true">🦀</span>
            <div>
              <h3 class="font-semibold">Rust 1.94.0+</h3>
              <p class="text-sm text-slate-400">Install via <code class="text-cyan-400">rustup</code></p>
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
            <span class="text-2xl" aria-hidden="true">📦</span>
            <div>
              <h3 class="font-semibold">Git</h3>
              <p class="text-sm text-slate-400">For cloning the repository</p>
            </div>
          </div>
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-4 flex items-start gap-3" role="listitem">
            <span class="text-2xl" aria-hidden="true">🔑</span>
            <div>
              <h3 class="font-semibold">API Key</h3>
              <p class="text-sm text-slate-400">From any supported provider</p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Platform Tabs -->
    <section class="py-12 px-4 border-b border-slate-800" aria-labelledby="platform-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="platform-heading" class="text-2xl font-bold mb-6 text-cyan-400 flex items-center gap-3">
          <span class="text-3xl" aria-hidden="true">🖥️</span>
          Platform-Specific Instructions
        </h2>
        
        <!-- Tab Navigation -->
        <div class="flex flex-wrap gap-2 mb-6 border-b border-slate-800 pb-4" role="tablist" aria-label="Platform selection">
          <button
            v-for="tab in platformTabs"
            :key="tab.id"
            @click="activePlatform = tab.id"
            :class="[
              'px-4 py-2 rounded-t font-medium transition-all motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
              activePlatform === tab.id
                ? 'bg-slate-800 text-cyan-400 border-t border-l border-r border-slate-700'
                : 'text-slate-400 hover:text-white hover:bg-slate-900'
            ]"
            role="tab"
            :id="`tab-${tab.id}`"
            :aria-selected="activePlatform === tab.id"
            :aria-controls="`panel-${tab.id}`"
            :tabindex="activePlatform === tab.id ? 0 : -1"
          >
            {{ tab.label }}
          </button>
        </div>

        <!-- Tab Content -->
        <div class="bg-slate-900 border border-slate-800 rounded-lg p-6">
          <!-- Linux Tab -->
          <div v-if="activePlatform === 'linux'" class="space-y-6" role="tabpanel" id="panel-linux" aria-labelledby="tab-linux">
            <p class="text-slate-300 mb-4">
              Linux is the primary platform for Cuttlefish. The guided installer works out of the box.
            </p>
            
            <div class="space-y-4">
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">1</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Install Rust</h4>
                  <TerminalBlock
                    command="curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
                    comment="Download and run rustup installer"
                  />
                </div>
              </div>
              
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">2</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Install Docker</h4>
                  <TerminalBlock
                    command="curl -fsSL https://get.docker.com | sudo bash"
                    comment="Install Docker Engine"
                  />
                  <TerminalBlock
                    command="sudo usermod -aG docker $USER"
                    comment="Add user to docker group (log out/in required)"
                  />
                </div>
              </div>
              
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">3</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Run the Installer</h4>
                  <TerminalBlock
                    command="curl -sSL https://raw.githubusercontent.com/JackTYM/cuttlefish-rs/master/install.sh | bash"
                    comment="Launch guided installer"
                  />
                </div>
              </div>
            </div>
            
            <div class="mt-6 p-4 bg-slate-800 rounded border border-slate-700" role="note">
              <p class="text-sm text-slate-300">
                <strong class="text-cyan-400">Systemd:</strong> The installer can set up Cuttlefish as a systemd service for 24/7 operation.
              </p>
            </div>
          </div>

          <!-- macOS Tab -->
          <div v-if="activePlatform === 'macos'" class="space-y-6" role="tabpanel" id="panel-macos" aria-labelledby="tab-macos">
            <p class="text-slate-300 mb-4">
              macOS works well with Homebrew for dependency installation.
            </p>
            
            <div class="space-y-4">
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">1</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Install Homebrew (if not installed)</h4>
                  <TerminalBlock
                    command="/bin/bash -c &quot;$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)&quot;"
                    comment="Install Homebrew package manager"
                  />
                </div>
              </div>
              
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">2</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Install Dependencies</h4>
                  <TerminalBlock
                    command="brew install rustup docker git"
                    comment="Install Rust, Docker, and Git"
                  />
                  <TerminalBlock
                    command="rustup-init -y"
                    comment="Initialize Rust toolchain"
                  />
                </div>
              </div>
              
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">3</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Start Docker Desktop</h4>
                  <p class="text-slate-400 text-sm mb-2">Open Docker Desktop from Applications and wait for it to start.</p>
                </div>
              </div>
              
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">4</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Run the Installer</h4>
                  <TerminalBlock
                    command="curl -sSL https://raw.githubusercontent.com/JackTYM/cuttlefish-rs/master/install.sh | bash"
                    comment="Launch guided installer"
                  />
                </div>
              </div>
            </div>
            
            <div class="mt-6 p-4 bg-slate-800 rounded border border-slate-700" role="note">
              <p class="text-sm text-slate-300">
                <strong class="text-cyan-400">Note:</strong> Docker Desktop for Mac uses a VM, so sandbox isolation works differently than on Linux.
              </p>
            </div>
          </div>

          <!-- Windows/WSL Tab -->
          <div v-if="activePlatform === 'windows'" class="space-y-6" role="tabpanel" id="panel-windows" aria-labelledby="tab-windows">
            <p class="text-slate-300 mb-4">
              Windows requires WSL 2 (Windows Subsystem for Linux) for Docker compatibility.
            </p>
            
            <div class="space-y-4">
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">1</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Enable WSL 2</h4>
                  <TerminalBlock
                    command="wsl --install"
                    comment="Run in PowerShell as Administrator"
                  />
                  <p class="text-slate-400 text-sm mt-2">Restart your computer when prompted.</p>
                </div>
              </div>
              
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">2</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Install Docker Desktop</h4>
                  <p class="text-slate-400 text-sm mb-2">
                    Download from <a href="https://www.docker.com/products/docker-desktop" target="_blank" rel="noopener" class="text-cyan-400 hover:underline">docker.com</a> and enable WSL 2 integration in settings.
                  </p>
                </div>
              </div>
              
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">3</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Open WSL Terminal</h4>
                  <TerminalBlock
                    command="wsl"
                    comment="Enter WSL environment"
                  />
                </div>
              </div>
              
              <div class="flex items-start gap-4">
                <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">4</span>
                <div class="flex-1">
                  <h4 class="font-semibold mb-2">Run the Installer</h4>
                  <TerminalBlock
                    command="curl -sSL https://raw.githubusercontent.com/JackTYM/cuttlefish-rs/master/install.sh | bash"
                    comment="Launch guided installer"
                  />
                </div>
              </div>
            </div>
            
            <div class="mt-6 p-4 bg-slate-800 rounded border border-slate-700" role="note">
              <p class="text-sm text-slate-300">
                <strong class="text-cyan-400">Important:</strong> All Cuttlefish commands must be run inside WSL. The Docker socket is shared from Windows.
              </p>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Manual Installation Section -->
    <section class="py-12 px-4 border-b border-slate-800" aria-labelledby="manual-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="manual-heading" class="text-2xl font-bold mb-6 text-cyan-400 flex items-center gap-3">
          <span class="text-3xl" aria-hidden="true">🔧</span>
          Manual Installation
        </h2>
        <p class="text-slate-300 mb-6">
          For developers who prefer full control over the installation process.
        </p>
        
        <div class="space-y-6">
          <!-- Step 1 -->
          <div class="flex items-start gap-4">
            <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">1</span>
            <div class="flex-1">
              <h3 class="font-semibold mb-2 text-white">Clone the Repository</h3>
              <TerminalBlock
                command="git clone https://github.com/JackTYM/cuttlefish-rs.git"
                comment="Clone from GitHub"
              />
              <TerminalBlock command="cd cuttlefish-rs" />
            </div>
          </div>
          
          <!-- Step 2 -->
          <div class="flex items-start gap-4">
            <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">2</span>
            <div class="flex-1">
              <h3 class="font-semibold mb-2 text-white">Build from Source</h3>
              <TerminalBlock
                command="cargo build --release"
                comment="Build optimized binary"
              />
            </div>
          </div>
          
          <!-- Step 3 -->
          <div class="flex items-start gap-4">
            <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">3</span>
            <div class="flex-1">
              <h3 class="font-semibold mb-2 text-white">Create Configuration</h3>
              <TerminalBlock
                command="cp cuttlefish.example.toml cuttlefish.toml"
                comment="Copy example config"
              />
              <p class="text-slate-400 text-sm mt-2">Edit <code class="text-cyan-400">cuttlefish.toml</code> with your settings.</p>
            </div>
          </div>
          
          <!-- Step 4 -->
          <div class="flex items-start gap-4">
            <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">4</span>
            <div class="flex-1">
              <h3 class="font-semibold mb-2 text-white">Set Environment Variables</h3>
              <TerminalBlock
                command="export CUTTLEFISH_API_KEY=&quot;your-secure-api-key&quot;"
                comment="API key for WebUI/TUI auth"
              />
              <TerminalBlock
                command="export ANTHROPIC_API_KEY=&quot;sk-ant-...&quot;"
                comment="For Anthropic provider"
              />
              <TerminalBlock
                command="export OPENAI_API_KEY=&quot;sk-...&quot;"
                comment="For OpenAI provider"
              />
              <TerminalBlock
                command="export GOOGLE_API_KEY=&quot;...&quot;"
                comment="For Google Gemini provider"
              />
            </div>
          </div>
          
          <!-- Step 5 -->
          <div class="flex items-start gap-4">
            <span class="flex-shrink-0 w-8 h-8 rounded-full bg-cyan-900/50 text-cyan-400 flex items-center justify-center font-bold">5</span>
            <div class="flex-1">
              <h3 class="font-semibold mb-2 text-white">Run the Server</h3>
              <TerminalBlock
                command="./target/release/cuttlefish-rs"
                comment="Start the server"
              />
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Configuration Example Section -->
    <section class="py-12 px-4 border-b border-slate-800" aria-labelledby="config-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="config-heading" class="text-2xl font-bold mb-6 text-cyan-400 flex items-center gap-3">
          <span class="text-3xl" aria-hidden="true">⚙️</span>
          Configuration Example
        </h2>
        <p class="text-slate-300 mb-6">
          Here's a sample <code class="text-cyan-400">cuttlefish.toml</code> with all available options:
        </p>
        
        <!-- Config File Display -->
        <div class="relative group mb-6">
          <div class="bg-slate-950 border border-slate-800 rounded-lg overflow-hidden">
            <div class="flex items-center justify-between px-4 py-2 bg-slate-900 border-b border-slate-700">
              <div class="flex items-center gap-2">
                <div class="flex gap-1.5">
                  <div class="w-3 h-3 rounded-full bg-red-500/80"></div>
                  <div class="w-3 h-3 rounded-full bg-yellow-500/80"></div>
                  <div class="w-3 h-3 rounded-full bg-green-500/80"></div>
                </div>
                <span class="text-xs text-slate-500 ml-2">cuttlefish.toml</span>
              </div>
              <button
                @click="copyConfig"
                class="px-3 py-2 sm:px-2 sm:py-1 min-h-[44px] sm:min-h-0 rounded bg-slate-800 hover:bg-slate-700 text-xs text-slate-400 transition-colors"
              >
                {{ configCopied ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <div class="p-4 font-mono text-sm overflow-x-auto">
              <pre class="text-slate-300"><code v-html="highlightedConfig"></code></pre>
            </div>
          </div>
        </div>
        
        <!-- Environment Variables Table -->
        <h3 id="env-vars-heading" class="text-lg font-semibold mb-4 mt-8">Environment Variables</h3>
        <div class="bg-slate-900 border border-slate-800 rounded-lg p-6 mb-6 overflow-x-auto">
          <table class="w-full text-sm" aria-labelledby="env-vars-heading">
            <thead>
              <tr class="border-b border-slate-700">
                <th scope="col" class="text-left py-2 pr-4 text-slate-400 font-medium">Variable</th>
                <th scope="col" class="text-left py-2 text-slate-400 font-medium">Description</th>
                <th scope="col" class="text-left py-2 text-slate-400 font-medium">Required</th>
              </tr>
            </thead>
            <tbody class="text-slate-300">
              <tr class="border-b border-slate-800">
                <td class="py-3 pr-4 font-mono text-cyan-400">CUTTLEFISH_API_KEY</td>
                <td class="py-3">API key for WebUI/TUI authentication</td>
                <td class="py-3"><span class="text-red-400">Yes</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="py-3 pr-4 font-mono text-cyan-400">ANTHROPIC_API_KEY</td>
                <td class="py-3">Anthropic API key</td>
                <td class="py-3"><span class="text-slate-500">If using Anthropic</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="py-3 pr-4 font-mono text-cyan-400">OPENAI_API_KEY</td>
                <td class="py-3">OpenAI API key</td>
                <td class="py-3"><span class="text-slate-500">If using OpenAI</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="py-3 pr-4 font-mono text-cyan-400">GOOGLE_API_KEY</td>
                <td class="py-3">Google Gemini API key</td>
                <td class="py-3"><span class="text-slate-500">If using Gemini</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="py-3 pr-4 font-mono text-cyan-400">AWS_ACCESS_KEY_ID</td>
                <td class="py-3">AWS access key for Bedrock</td>
                <td class="py-3"><span class="text-slate-500">If using Bedrock</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="py-3 pr-4 font-mono text-cyan-400">AWS_SECRET_ACCESS_KEY</td>
                <td class="py-3">AWS secret key for Bedrock</td>
                <td class="py-3"><span class="text-slate-500">If using Bedrock</span></td>
              </tr>
              <tr class="border-b border-slate-800">
                <td class="py-3 pr-4 font-mono text-cyan-400">AWS_DEFAULT_REGION</td>
                <td class="py-3">AWS region (e.g., us-east-1)</td>
                <td class="py-3"><span class="text-slate-500">If using Bedrock</span></td>
              </tr>
              <tr>
                <td class="py-3 pr-4 font-mono text-cyan-400">DISCORD_BOT_TOKEN</td>
                <td class="py-3">Discord bot token</td>
                <td class="py-3"><span class="text-slate-500">Optional</span></td>
              </tr>
            </tbody>
          </table>
        </div>
        
        <a
          href="/docs/configuration"
          class="inline-flex items-center gap-2 text-cyan-400 hover:text-cyan-300 transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded"
        >
          <span>View full configuration reference</span>
          <span aria-hidden="true">→</span>
        </a>
      </div>
    </section>

    <!-- Verification Section -->
    <section class="py-12 px-4 border-b border-slate-800" aria-labelledby="verify-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="verify-heading" class="text-2xl font-bold mb-6 text-cyan-400 flex items-center gap-3">
          <span class="text-3xl" aria-hidden="true">✅</span>
          Verify Installation
        </h2>
        <p class="text-slate-300 mb-6">
          Confirm Cuttlefish is running correctly with these checks:
        </p>
        
        <div class="space-y-6">
          <!-- Health Check -->
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <h3 class="font-semibold mb-3 text-white flex items-center gap-2">
              <span class="text-green-400">1.</span>
              Health Check
            </h3>
            <TerminalBlock
              command="curl http://localhost:8080/health"
              comment="Check server health"
            />
            <div class="mt-4 p-3 bg-slate-800 rounded border border-slate-700">
              <p class="text-sm text-slate-400 mb-1">Expected output:</p>
              <pre class="text-green-400 font-mono text-sm">{"status":"ok"}</pre>
            </div>
          </div>
          
          <!-- WebSocket Check -->
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <h3 class="font-semibold mb-3 text-white flex items-center gap-2">
              <span class="text-green-400">2.</span>
              WebSocket Connection
            </h3>
            <p class="text-slate-400 text-sm mb-3">
              Open <code class="text-cyan-400">ws://localhost:8080/ws</code> in a WebSocket client or the WebUI.
            </p>
          </div>
          
          <!-- API Check -->
          <div class="bg-slate-900 border border-slate-800 rounded-lg p-6">
            <h3 class="font-semibold mb-3 text-white flex items-center gap-2">
              <span class="text-green-400">3.</span>
              API Endpoints
            </h3>
            <TerminalBlock
              command="curl -H &quot;Authorization: Bearer $CUTTLEFISH_API_KEY&quot; http://localhost:8080/api/projects"
              comment="List projects (requires API key)"
            />
            <div class="mt-4 p-3 bg-slate-800 rounded border border-slate-700">
              <p class="text-sm text-slate-400 mb-1">Expected output:</p>
              <pre class="text-green-400 font-mono text-sm">{"projects": []}</pre>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- Troubleshooting FAQ -->
    <section class="py-12 px-4 border-b border-slate-800" aria-labelledby="troubleshooting-heading">
      <div class="max-w-4xl mx-auto">
        <h2 id="troubleshooting-heading" class="text-2xl font-bold mb-6 text-cyan-400 flex items-center gap-3">
          <span class="text-3xl" aria-hidden="true">🔍</span>
          Troubleshooting
        </h2>
        
        <div class="space-y-3">
          <FAQItem
            question="Docker socket permission denied"
            answer="Add your user to the docker group: sudo usermod -aG docker $USER, then log out and back in. Verify with: docker ps"
          />
          <FAQItem
            question="AWS credentials not found"
            answer="Ensure AWS_ACCESS_KEY_ID and AWS_SECRET_ACCESS_KEY are set in your environment. Verify with: echo $AWS_ACCESS_KEY_ID"
          />
          <FAQItem
            question="Port 8080 already in use"
            answer="Change the port in cuttlefish.toml under [server].port, or stop the conflicting service with: lsof -i :8080"
          />
          <FAQItem
            question="Build fails with Rust version error"
            answer="Install Rust 1.94.0+: rustup install 1.94.0 && rustup default 1.94.0"
          />
          <FAQItem
            question="Bedrock access denied"
            answer="Ensure your AWS account has Bedrock access enabled. Request access in the AWS console under Amazon Bedrock service."
          />
          <FAQItem
            question="API key authentication fails"
            answer="Make sure CUTTLEFISH_API_KEY is set and matches the key used in requests. Check for trailing whitespace or quotes."
          />
        </div>
      </div>
    </section>

    <!-- Bottom CTA -->
    <section class="py-16 px-4 bg-slate-900/50" aria-labelledby="help-heading">
      <div class="max-w-4xl mx-auto text-center">
        <h2 id="help-heading" class="text-2xl font-bold mb-4">Need Help?</h2>
        <p class="text-slate-400 mb-8">Get support from the community or the maintainers</p>
        <div class="flex flex-wrap justify-center gap-4">
          <a
            href="https://github.com/JackTYM/cuttlefish-rs/discussions"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-2 bg-cyan-600 hover:bg-cyan-500 px-6 py-3 rounded-lg font-semibold transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
          >
            <span aria-hidden="true">💬</span>
            <span>GitHub Discussions</span>
            <span class="sr-only">(opens in new tab)</span>
          </a>
          <a
            href="https://github.com/JackTYM/cuttlefish-rs/issues"
            target="_blank"
            rel="noopener noreferrer"
            class="inline-flex items-center gap-2 bg-slate-700 hover:bg-slate-600 px-6 py-3 rounded-lg font-semibold transition motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-slate-400 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
          >
            <span aria-hidden="true">🐛</span>
            <span>Report an Issue</span>
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
import { ref, computed } from 'vue'

definePageMeta({
  layout: 'default'
})

// Platform tabs
const platformTabs = [
  { id: 'linux', label: '🐧 Linux' },
  { id: 'macos', label: '🍎 macOS' },
  { id: 'windows', label: '🪟 Windows/WSL' }
]

const activePlatform = ref('linux')

// Copy states
const installCopied = ref(false)
const configCopied = ref(false)

// Copy functions
const copyInstallCommand = async () => {
  try {
    await navigator.clipboard.writeText('curl -sSL https://raw.githubusercontent.com/JackTYM/cuttlefish-rs/master/install.sh | bash')
    installCopied.value = true
    setTimeout(() => { installCopied.value = false }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}

// Configuration example with syntax highlighting
const configExample = `# Server Configuration
[server]
host = "127.0.0.1"      # Bind address
port = 8080             # HTTP/WebSocket port

# Database Configuration
[database]
path = "cuttlefish.db"  # SQLite database (WAL mode)

# Docker Sandbox Configuration
[sandbox]
docker_socket = "unix:///var/run/docker.sock"
memory_limit_mb = 2048  # Per-container memory
cpu_limit = 2.0         # Per-container CPU cores
disk_limit_gb = 10      # Per-container disk
max_concurrent = 5      # Max concurrent sandboxes

# Model Providers
[providers.claude]
provider_type = "bedrock"
model = "anthropic.claude-sonnet-4-6-20260101-v1:0"
region = "us-east-1"

[providers.claude-fast]
provider_type = "bedrock"
model = "anthropic.claude-haiku-4-5-20260101-v1:0"
region = "us-east-1"

# Agent Configuration
[agents.orchestrator]
category = "deep"

[agents.coder]
category = "deep"

[agents.critic]
category = "unspecified-high"

# Discord Bot (Optional)
[discord]
token_env_var = "DISCORD_BOT_TOKEN"
guild_ids = []`

const highlightedConfig = computed(() => {
  return configExample
    // Comments
    .replace(/(#.*)$/gm, '<span class="text-slate-500">$1</span>')
    // Section headers
    .replace(/^\[([^\]]+)\]/gm, '<span class="text-purple-400">[$1]</span>')
    // Keys
    .replace(/^(\w+)\s*=/gm, '<span class="text-cyan-400">$1</span> =')
    // String values
    .replace(/= "([^"]+)"/g, '= <span class="text-green-400">"$1"</span>')
    // Numbers
    .replace(/= (\d+\.?\d*)/g, '= <span class="text-yellow-400">$1</span>')
})

const copyConfig = async () => {
  try {
    await navigator.clipboard.writeText(configExample)
    configCopied.value = true
    setTimeout(() => { configCopied.value = false }, 2000)
  } catch (err) {
    console.error('Failed to copy:', err)
  }
}
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