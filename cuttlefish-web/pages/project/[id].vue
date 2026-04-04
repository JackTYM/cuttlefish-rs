<template>
  <div class="flex flex-col h-full">
    <!-- Project Header with Tabs -->
    <header class="bg-gray-900 border-b border-gray-800 px-4 sm:px-6 py-3 flex items-center gap-3 sm:gap-4 shrink-0">
      <NuxtLink 
        to="/" 
        class="text-cyan-400 hover:text-cyan-300 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded p-1" 
        aria-label="Back to projects"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
        </svg>
      </NuxtLink>
      
      <!-- Project Title -->
      <div class="flex items-center gap-2 min-w-0">
        <h1 class="text-base sm:text-lg font-semibold truncate" :class="{ 'text-gray-500 line-through': isArchived }">
          {{ projectInfo?.name || route.params.id }}
        </h1>
        <span v-if="projectInfo?.template" class="text-xs px-2 py-0.5 rounded-full bg-purple-900/50 text-purple-300 border border-purple-700/50 shrink-0">
          {{ projectInfo.template }}
        </span>
        <TerminalStatusBadge
          v-if="isArchived"
          status="pending"
          label="Archived"
          class="shrink-0"
        />
      </div>
      
      <!-- Connection Status -->
      <div class="hidden sm:flex items-center gap-2 ml-auto mr-4">
        <span 
          class="w-2 h-2 rounded-full transition-colors motion-reduce:transition-none"
          :class="connected ? 'bg-green-500' : 'bg-red-500 animate-pulse'"
          :aria-label="connected ? 'Connected' : 'Disconnected'"
        />
        <span class="text-xs text-gray-500">{{ connected ? 'Connected' : 'Reconnecting...' }}</span>
      </div>
      
      <!-- Action buttons -->
      <div class="flex items-center gap-2 ml-auto sm:ml-0">
        <button
          @click="toggleArchive"
          class="flex items-center gap-1.5 px-2 sm:px-3 py-1.5 text-sm rounded-lg transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
          :class="isArchived 
            ? 'bg-green-900/50 text-green-400 hover:bg-green-800/50 border border-green-700/50' 
            : 'bg-yellow-900/30 text-yellow-400 hover:bg-yellow-900/50 border border-yellow-700/30'"
          :aria-label="isArchived ? 'Restore project from archive' : 'Archive project'"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path v-if="isArchived" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
          </svg>
          <span class="hidden sm:inline">{{ isArchived ? 'Restore' : 'Archive' }}</span>
        </button>
        <button
          @click="showDeleteModal = true"
          class="flex items-center gap-1.5 px-2 sm:px-3 py-1.5 text-sm rounded-lg bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700/30 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
          aria-label="Delete project permanently"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          <span class="hidden sm:inline">Delete</span>
        </button>
      </div>
    </header>

    <!-- Tab Navigation -->
    <nav class="bg-gray-900 border-b border-gray-800 px-4 sm:px-6 flex gap-1 shrink-0 overflow-x-auto scrollbar-thin" aria-label="Project sections" role="tablist">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        @click="activeTab = tab.id"
        class="relative px-3 sm:px-4 py-3 sm:py-2.5 min-h-[44px] sm:min-h-0 text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 whitespace-nowrap flex items-center gap-2"
        :class="activeTab === tab.id ? 'text-cyan-400' : 'text-gray-400 hover:text-white'"
        :aria-selected="activeTab === tab.id"
        :tabindex="activeTab === tab.id ? 0 : -1"
        role="tab"
        :id="`tab-${tab.id}`"
        :aria-controls="`panel-${tab.id}`"
      >
        <component :is="tab.icon" class="w-4 h-4" />
        {{ tab.label }}
        <span 
          v-if="tab.badge && tab.badge > 0"
          class="absolute -top-1 -right-1 w-4 h-4 bg-cyan-500 text-white text-xs rounded-full flex items-center justify-center"
        >
          {{ tab.badge > 99 ? '99+' : tab.badge }}
        </span>
        <!-- Active indicator -->
        <span 
          v-if="activeTab === tab.id"
          class="absolute bottom-0 left-0 right-0 h-0.5 bg-cyan-400"
          aria-hidden="true"
        />
      </button>
    </nav>

    <!-- Loading State -->
    <div v-if="isLoading" class="flex-1 flex items-center justify-center">
      <div class="text-center">
        <svg class="w-8 h-8 text-cyan-500 animate-spin mx-auto mb-3" fill="none" viewBox="0 0 24 24" aria-hidden="true">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
        </svg>
        <p class="text-gray-500">Loading project...</p>
      </div>
    </div>

    <!-- Error State -->
    <div v-else-if="loadError" class="flex-1 flex items-center justify-center p-6">
      <div class="text-center max-w-md">
        <svg class="w-12 h-12 text-red-500 mx-auto mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <h3 class="text-lg font-semibold text-gray-200 mb-2">Failed to Load Project</h3>
        <p class="text-gray-500 mb-4">{{ loadError }}</p>
        <button
          @click="loadProject"
          class="bg-cyan-600 hover:bg-cyan-500 text-white px-4 py-2 rounded-lg text-sm transition-colors motion-reduce:transition-none"
        >
          Try Again
        </button>
      </div>
    </div>

    <!-- Tab Content -->
    <template v-else>
      <!-- Chat Tab -->
      <div v-show="activeTab === 'chat'" class="flex flex-col flex-1 overflow-hidden" role="tabpanel" id="panel-chat" aria-labelledby="tab-chat">
        <div ref="chatEl" class="flex-1 overflow-y-auto p-4 sm:p-6 space-y-4" aria-live="polite" aria-label="Chat messages">
          <!-- Empty Chat State -->
          <div v-if="!projectMessages.length" class="flex items-center justify-center h-full text-gray-500">
            <div class="text-center max-w-md">
              <div class="text-4xl mb-4">💬</div>
              <h3 class="text-lg font-medium text-gray-300 mb-2">Start a Conversation</h3>
              <p class="text-sm text-gray-500 mb-4">Describe what you want to build and the agents will help you create it.</p>
              <div class="flex flex-wrap gap-2 justify-center">
                <button
                  v-for="suggestion in chatSuggestions"
                  :key="suggestion"
                  @click="input = suggestion"
                  class="text-xs px-3 py-1.5 rounded-full bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-gray-300 border border-gray-700 transition-colors motion-reduce:transition-none"
                >
                  {{ suggestion }}
                </button>
              </div>
            </div>
          </div>
          
          <!-- Messages -->
          <TransitionGroup name="message-list">
            <div 
              v-for="(msg, i) in projectMessages" 
              :key="msg.timestamp" 
              class="flex gap-3 group"
              role="article" 
              :aria-label="`Message from ${msg.sender}`"
            >
              <!-- Avatar -->
              <div
                class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold shrink-0 ring-2 ring-gray-800"
                :class="{
                  'bg-cyan-700 ring-cyan-600': msg.sender === 'user',
                  'bg-purple-700 ring-purple-600': msg.sender === 'orchestrator',
                  'bg-yellow-700 ring-yellow-600': msg.sender === 'coder',
                  'bg-red-700 ring-red-600': msg.sender === 'critic',
                  'bg-gray-700 ring-gray-600': !['user','orchestrator','coder','critic'].includes(msg.sender)
                }"
                aria-hidden="true"
              >{{ msg.sender[0]?.toUpperCase() }}</div>
              
              <!-- Message Content -->
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1">
                  <span class="text-sm font-medium" :class="{
                    'text-cyan-400': msg.sender === 'user',
                    'text-purple-400': msg.sender === 'orchestrator',
                    'text-yellow-400': msg.sender === 'coder',
                    'text-red-400': msg.sender === 'critic',
                    'text-gray-400': !['user','orchestrator','coder','critic'].includes(msg.sender)
                  }">{{ msg.sender }}</span>
                  <span class="text-xs text-gray-600">{{ formatTime(msg.timestamp) }}</span>
                </div>
                <div class="prose prose-invert prose-sm max-w-none text-gray-200" v-html="renderMarkdown(msg.content)" />
              </div>
            </div>
          </TransitionGroup>
        </div>
        
        <!-- Chat Input -->
        <div class="border-t border-gray-800 p-4 flex gap-3 shrink-0 bg-gray-900/50">
          <label for="chat-input" class="sr-only">Message input</label>
          <div class="flex-1 relative">
            <textarea
              id="chat-input"
              ref="chatInputRef"
              v-model="input"
              @keydown.enter.exact.prevent="sendMessage"
              @keydown.enter.shift.exact="() => {}"
              @input="autoResize"
              placeholder="Describe what you want to build... (Shift+Enter for new line)"
              rows="1"
              class="w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 resize-none min-h-[44px] max-h-32 transition-colors motion-reduce:transition-none"
            />
          </div>
          <button 
            @click="sendMessage" 
            :disabled="!input.trim()"
            class="bg-cyan-600 hover:bg-cyan-500 disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed text-white px-4 py-2 rounded-lg text-sm transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 self-end min-h-[44px]"
            aria-label="Send message"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
            </svg>
          </button>
        </div>
      </div>

      <!-- Build Log Tab -->
      <div v-show="activeTab === 'build-log'" class="flex-1 overflow-hidden flex flex-col" role="tabpanel" id="panel-build-log" aria-labelledby="tab-build-log">
        <!-- Log Header -->
        <div class="bg-gray-900 border-b border-gray-800 px-4 sm:px-6 py-3 flex items-center justify-between shrink-0">
          <div class="flex items-center gap-3">
            <span class="text-sm font-medium text-gray-300">Build Output</span>
            <span v-if="logLines.length" class="text-xs text-gray-500">{{ logLines.length }} lines</span>
          </div>
          <div class="flex items-center gap-2">
            <button
              v-if="logLines.length"
              @click="copyLogs"
              class="text-xs px-3 py-1.5 rounded bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-gray-200 transition-colors motion-reduce:transition-none"
            >
              {{ logsCopied ? '✓ Copied!' : 'Copy Logs' }}
            </button>
            <button
              v-if="logLines.length"
              @click="logLines = []"
              class="text-xs px-3 py-1.5 rounded bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-gray-200 transition-colors motion-reduce:transition-none"
            >
              Clear
            </button>
          </div>
        </div>
        
        <!-- Log Content -->
        <div class="flex-1 overflow-y-auto font-mono text-xs sm:text-sm bg-black" aria-label="Build log output">
          <div v-if="!logLines.length" class="flex items-center justify-center h-full text-gray-500">
            <div class="text-center">
              <svg class="w-12 h-12 mx-auto mb-3 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
              </svg>
              <p>No build logs yet</p>
              <p class="text-xs text-gray-600 mt-1">Run a build to see output here</p>
            </div>
          </div>
          <div v-else class="p-4">
            <div
              v-for="(line, i) in logLines"
              :key="i"
              class="leading-relaxed py-0.5 hover:bg-gray-900/50 transition-colors motion-reduce:transition-none"
              :class="getLogLineClass(line)"
            >
              <span class="select-none text-gray-600 mr-3 inline-block w-8 text-right">{{ i + 1 }}</span>
              <span v-html="highlightLogLine(line)" />
            </div>
          </div>
        </div>
      </div>

      <!-- Diff Tab -->
      <div v-show="activeTab === 'diff'" class="flex-1 overflow-hidden flex flex-col" role="tabpanel" id="panel-diff" aria-labelledby="tab-diff">
        <!-- Diff Header -->
        <div class="bg-gray-900 border-b border-gray-800 px-4 sm:px-6 py-3 flex items-center justify-between shrink-0">
          <div class="flex items-center gap-3">
            <span class="text-sm font-medium text-gray-300">Changes</span>
            <span v-if="diffStats" class="text-xs text-gray-500" role="status" aria-label="Diff statistics">
              <span class="text-green-400">+{{ diffStats.additions }}</span>
              <span class="text-gray-600 mx-1">/</span>
              <span class="text-red-400">-{{ diffStats.deletions }}</span>
            </span>
          </div>
          <div class="flex items-center gap-2" role="group" aria-label="Diff view mode">
            <button
              @click="diffViewMode = 'unified'"
              class="px-3 py-1 text-xs rounded transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
              :class="diffViewMode === 'unified' ? 'bg-cyan-900/50 text-cyan-400' : 'text-gray-500 hover:text-gray-300'"
              :aria-pressed="diffViewMode === 'unified'"
              aria-label="Unified diff view"
            >
              Unified
            </button>
            <button
              @click="diffViewMode = 'split'"
              class="px-3 py-1 text-xs rounded transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
              :class="diffViewMode === 'split' ? 'bg-cyan-900/50 text-cyan-400' : 'text-gray-500 hover:text-gray-300'"
              :aria-pressed="diffViewMode === 'split'"
              aria-label="Split diff view"
            >
              Split
            </button>
          </div>
        </div>
        
        <!-- Diff Content -->
        <div class="flex-1 overflow-y-auto font-mono text-xs sm:text-sm bg-gray-950" aria-label="Diff content">
          <div v-if="!diffContent" class="flex items-center justify-center h-full text-gray-500">
            <div class="text-center">
              <svg class="w-12 h-12 mx-auto mb-3 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p>No changes yet</p>
              <p class="text-xs text-gray-600 mt-1">Changes will appear here after edits</p>
            </div>
          </div>
          
          <!-- Unified View -->
          <div v-else-if="diffViewMode === 'unified'" class="divide-y divide-gray-800">
            <div v-for="(hunk, hunkIdx) in diffHunks" :key="hunkIdx">
              <!-- Hunk Header -->
              <button
                @click="toggleHunk(hunkIdx)"
                class="w-full px-4 py-2 bg-gray-900 text-cyan-400 text-xs sticky top-0 border-b border-gray-800 flex items-center gap-2 hover:bg-gray-800 transition-colors motion-reduce:transition-none text-left"
              >
                <svg 
                  class="w-3 h-3 transition-transform motion-reduce:transition-none"
                  :class="expandedHunks.has(hunkIdx) ? 'rotate-90' : ''"
                  fill="none" 
                  stroke="currentColor" 
                  viewBox="0 0 24 24"
                  aria-hidden="true"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
                {{ hunk.header }}
              </button>
              <!-- Hunk Lines -->
              <div v-show="expandedHunks.has(hunkIdx)" class="relative">
                <div v-for="(line, i) in hunk.lines" :key="i" class="flex group">
                  <!-- Line Number -->
                  <div class="w-10 sm:w-12 shrink-0 text-right pr-2 sm:pr-3 py-0.5 text-xs select-none"
                    :class="{
                      'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                      'text-green-700 bg-green-950/30': line.isAddition,
                      'text-red-700 bg-red-950/30': line.isDeletion,
                    }"
                  >{{ line.oldLine || '' }}</div>
                  <div class="w-10 sm:w-12 shrink-0 text-right pr-2 sm:pr-3 py-0.5 text-xs select-none border-r border-gray-800"
                    :class="{
                      'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                      'text-green-700 bg-green-950/30': line.isAddition,
                      'text-red-700 bg-red-950/30': line.isDeletion,
                    }"
                  >{{ line.newLine || '' }}</div>
                  <!-- Diff Indicator -->
                  <div class="w-5 sm:w-6 shrink-0 text-center py-0.5 text-xs"
                    :class="{
                      'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                      'text-green-400 bg-green-950/50': line.isAddition,
                      'text-red-400 bg-red-950/50': line.isDeletion,
                    }"
                  >{{ line.isAddition ? '+' : line.isDeletion ? '-' : ' ' }}</div>
                  <!-- Line Content -->
                  <div class="flex-1 px-2 sm:px-3 py-0.5 whitespace-pre overflow-x-auto"
                    :class="{
                      'text-gray-300': !line.isAddition && !line.isDeletion,
                      'text-green-300 bg-green-950/20': line.isAddition,
                      'text-red-300 bg-red-950/20': line.isDeletion,
                    }"
                  ><span v-html="highlightSyntax(line.content, line.isAddition, line.isDeletion)" /></div>
                </div>
              </div>
            </div>
          </div>
          
          <!-- Split View -->
          <div v-else class="flex h-full">
            <!-- Left (Old) -->
            <div class="flex-1 border-r border-gray-800 overflow-y-auto">
              <div class="px-4 py-2 bg-gray-900 text-xs text-gray-500 border-b border-gray-800 sticky top-0">Original</div>
              <div v-for="(line, i) in splitDiffLines.left" :key="i" class="flex">
                <div class="w-10 sm:w-12 shrink-0 text-right pr-2 sm:pr-3 py-0.5 text-xs text-gray-600 bg-gray-900/30">{{ line.num || '' }}</div>
                <div class="flex-1 px-2 sm:px-3 py-0.5 whitespace-pre text-xs sm:text-sm"
                  :class="{
                    'text-gray-300': !line.isChange,
                    'text-red-300 bg-red-950/30': line.isChange,
                  }"
                >{{ line.content }}</div>
              </div>
            </div>
            <!-- Right (New) -->
            <div class="flex-1 overflow-y-auto">
              <div class="px-4 py-2 bg-gray-900 text-xs text-gray-500 border-b border-gray-800 sticky top-0">Modified</div>
              <div v-for="(line, i) in splitDiffLines.right" :key="i" class="flex">
                <div class="w-10 sm:w-12 shrink-0 text-right pr-2 sm:pr-3 py-0.5 text-xs text-gray-600 bg-gray-900/30">{{ line.num || '' }}</div>
                <div class="flex-1 px-2 sm:px-3 py-0.5 whitespace-pre text-xs sm:text-sm"
                  :class="{
                    'text-gray-300': !line.isChange,
                    'text-green-300 bg-green-950/30': line.isChange,
                  }"
                >{{ line.content }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Files Tab -->
      <div v-show="activeTab === 'files'" class="flex-1 overflow-hidden flex flex-col sm:flex-row" role="tabpanel" id="panel-files" aria-labelledby="tab-files">
        <!-- File Tree -->
        <nav class="w-full sm:w-56 md:w-64 bg-gray-900 border-r border-gray-800 overflow-y-auto shrink-0 sm:max-h-full" aria-label="File explorer">
          <div class="p-3 border-b border-gray-800 flex items-center justify-between sticky top-0 bg-gray-900">
            <span class="text-sm font-medium text-gray-300">Files</span>
            <button class="text-gray-500 hover:text-cyan-400 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded p-2 sm:p-1 min-h-[44px] sm:min-h-0 min-w-[44px] sm:min-w-0" aria-label="Add new file">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
            </button>
          </div>
          <div class="p-2" role="tree" aria-label="Project files">
            <template v-for="item in fileTree" :key="item.path">
              <!-- Folder -->
              <div v-if="item.type === 'folder'" class="mb-1" role="treeitem" :aria-expanded="expandedFolders.has(item.path)">
                <button
                  @click="toggleFolder(item.path)"
                  class="w-full flex items-center gap-2 px-2 py-2.5 sm:py-1.5 min-h-[44px] sm:min-h-0 rounded text-sm hover:bg-gray-800 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                  :class="expandedFolders.has(item.path) ? 'text-cyan-400' : 'text-gray-300'"
                  :aria-label="`${item.name} folder, ${expandedFolders.has(item.path) ? 'expanded' : 'collapsed'}`"
                >
                  <svg
                    class="w-4 h-4 transition-transform motion-reduce:transition-none"
                    :class="expandedFolders.has(item.path) ? 'rotate-90' : ''"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    aria-hidden="true"
                  >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                  </svg>
                  <svg class="w-4 h-4 text-yellow-500" fill="currentColor" viewBox="0 0 20 20" aria-hidden="true">
                    <path d="M2 6a2 2 0 012-2h5l2 2h5a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                  </svg>
                  <span class="truncate">{{ item.name }}</span>
                </button>
                <!-- Children -->
                <div v-if="expandedFolders.has(item.path)" class="ml-4" role="group">
                  <button
                    v-for="child in item.children"
                    :key="child.path"
                    @click="selectFile(child)"
                    class="w-full flex items-center gap-2 px-2 py-2.5 sm:py-1.5 min-h-[44px] sm:min-h-0 rounded text-sm hover:bg-gray-800 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                    :class="selectedFile?.path === child.path ? 'text-cyan-400 bg-cyan-950/30' : 'text-gray-400'"
                    role="treeitem"
                    :aria-selected="selectedFile?.path === child.path"
                    :aria-label="`${child.name} file`"
                  >
                    <FileIcon :filename="child.name" />
                    <span class="truncate">{{ child.name }}</span>
                  </button>
                </div>
              </div>
              <!-- Root-level file -->
              <button
                v-else
                @click="selectFile(item)"
                class="w-full flex items-center gap-2 px-2 py-2.5 sm:py-1.5 min-h-[44px] sm:min-h-0 rounded text-sm hover:bg-gray-800 transition-colors motion-reduce:transition-none mb-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                :class="selectedFile?.path === item.path ? 'text-cyan-400 bg-cyan-950/30' : 'text-gray-400'"
                role="treeitem"
                :aria-selected="selectedFile?.path === item.path"
                :aria-label="`${item.name} file`"
              >
                <FileIcon :filename="item.name" />
                <span class="truncate">{{ item.name }}</span>
              </button>
            </template>
          </div>
        </nav>
        
        <!-- File Content -->
        <main class="flex-1 overflow-y-auto bg-gray-950" aria-label="File content viewer">
          <div v-if="selectedFile" class="p-4">
            <div class="flex items-center justify-between mb-3 pb-3 border-b border-gray-800">
              <div class="flex items-center gap-2 min-w-0">
                <FileIcon :filename="selectedFile.name" />
                <span class="text-sm font-medium text-gray-200 truncate">{{ selectedFile.path }}</span>
              </div>
              <div class="flex items-center gap-3">
                <span class="text-xs text-gray-500">{{ selectedFile.size }}</span>
                <button
                  @click="copyFileContent"
                  class="text-xs px-2 py-1 rounded bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-gray-200 transition-colors motion-reduce:transition-none"
                >
                  {{ fileCopied ? '✓ Copied!' : 'Copy' }}
                </button>
              </div>
            </div>
            <TerminalCodeBlock
              :code="selectedFile.content || ''"
              :language="getFileLanguage(selectedFile.name)"
              :show-line-numbers="true"
            />
          </div>
          <div v-else class="flex items-center justify-center h-full text-gray-500">
            <div class="text-center">
              <svg class="w-12 h-12 mx-auto mb-3 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
              </svg>
              <p>Select a file to view its contents</p>
            </div>
          </div>
        </main>
      </div>

      <!-- Settings Tab -->
      <div v-show="activeTab === 'settings'" class="flex-1 overflow-y-auto p-4 sm:p-6" role="tabpanel" id="panel-settings" aria-labelledby="tab-settings">
        <div class="max-w-2xl mx-auto space-y-6">
          <!-- Project Info -->
          <section class="bg-gray-900 rounded-xl border border-gray-800 p-4 sm:p-5" aria-labelledby="project-info-heading">
            <h3 id="project-info-heading" class="text-sm font-semibold text-gray-300 mb-4 flex items-center gap-2">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Project Information
            </h3>
            <div class="space-y-4">
              <div>
                <label id="project-id-label" class="block text-xs text-gray-500 mb-1">Project ID</label>
                <div class="bg-gray-800 rounded-lg px-3 py-2 text-sm text-gray-400 font-mono" aria-labelledby="project-id-label">{{ route.params.id }}</div>
              </div>
              <div>
                <label for="project-name" class="block text-xs text-gray-500 mb-1">Project Name</label>
                <input
                  id="project-name"
                  v-model="settings.name"
                  class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors motion-reduce:transition-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                />
              </div>
              <div>
                <label for="project-description" class="block text-xs text-gray-500 mb-1">Description</label>
                <textarea
                  id="project-description"
                  v-model="settings.description"
                  rows="3"
                  class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm focus:outline-none focus:border-cyan-500 transition-colors motion-reduce:transition-none resize-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                />
              </div>
              <div class="flex items-center justify-between py-2">
                <div>
                  <div id="archived-label" class="text-sm font-medium text-gray-200">Archived</div>
                  <div id="archived-description" class="text-xs text-gray-500">Soft delete - hides from main list</div>
                </div>
                <button
                  @click="toggleArchive"
                  class="relative inline-flex h-6 w-11 items-center rounded-full transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                  :class="isArchived ? 'bg-yellow-600' : 'bg-gray-700'"
                  role="switch"
                  :aria-checked="isArchived"
                  aria-labelledby="archived-label"
                  aria-describedby="archived-description"
                >
                  <span
                    class="inline-block h-4 w-4 transform rounded-full bg-white transition-transform motion-reduce:transition-none"
                    :class="isArchived ? 'translate-x-6' : 'translate-x-1'"
                    aria-hidden="true"
                  />
                </button>
              </div>
            </div>
          </section>

          <!-- Sandbox Settings -->
          <section class="bg-gray-900 rounded-xl border border-gray-800 p-4 sm:p-5" aria-labelledby="sandbox-heading">
            <h3 id="sandbox-heading" class="text-sm font-semibold text-gray-300 mb-4 flex items-center gap-2">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 12h14M5 12a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v4a2 2 0 01-2 2M5 12a2 2 0 00-2 2v4a2 2 0 002 2h14a2 2 0 002-2v-4a2 2 0 00-2-2m-2-4h.01M17 16h.01" />
              </svg>
              Sandbox Configuration
            </h3>
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div>
                <label for="memory-limit" class="block text-xs text-gray-500 mb-1">Memory Limit (MB)</label>
                <input
                  id="memory-limit"
                  v-model.number="settings.memoryLimit"
                  type="number"
                  min="256"
                  max="8192"
                  class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-3 sm:py-2 min-h-[44px] sm:min-h-0 text-sm focus:outline-none focus:border-cyan-500 transition-colors motion-reduce:transition-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                />
              </div>
              <div>
                <label for="cpu-limit" class="block text-xs text-gray-500 mb-1">CPU Limit (cores)</label>
                <input
                  id="cpu-limit"
                  v-model.number="settings.cpuLimit"
                  type="number"
                  min="0.5"
                  max="8"
                  step="0.5"
                  class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-3 sm:py-2 min-h-[44px] sm:min-h-0 text-sm focus:outline-none focus:border-cyan-500 transition-colors motion-reduce:transition-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                />
              </div>
              <div>
                <label for="disk-limit" class="block text-xs text-gray-500 mb-1">Disk Limit (GB)</label>
                <input
                  id="disk-limit"
                  v-model.number="settings.diskLimit"
                  type="number"
                  min="1"
                  max="50"
                  class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-3 sm:py-2 min-h-[44px] sm:min-h-0 text-sm focus:outline-none focus:border-cyan-500 transition-colors motion-reduce:transition-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                />
              </div>
              <div>
                <label for="network-access" class="block text-xs text-gray-500 mb-1">Network Access</label>
                <select
                  id="network-access"
                  v-model="settings.networkAccess"
                  class="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-3 sm:py-2 min-h-[44px] sm:min-h-0 text-sm focus:outline-none focus:border-cyan-500 transition-colors motion-reduce:transition-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                >
                  <option value="enabled">Enabled</option>
                  <option value="disabled">Disabled</option>
                </select>
              </div>
            </div>
          </section>

          <!-- Agent Settings -->
          <section class="bg-gray-900 rounded-xl border border-gray-800 p-4 sm:p-5" aria-labelledby="agent-heading">
            <h3 id="agent-heading" class="text-sm font-semibold text-gray-300 mb-4 flex items-center gap-2">
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
              </svg>
              Agent Configuration
            </h3>
            <div class="space-y-3">
              <div v-for="(agent, index) in agentSettings" :key="agent.role" class="flex flex-col sm:flex-row sm:items-center justify-between py-2 border-b border-gray-800 last:border-0 gap-2 sm:gap-0">
                <div>
                  <div :id="`agent-${index}-label`" class="text-sm font-medium text-gray-200">{{ agent.role }}</div>
                  <div :id="`agent-${index}-description`" class="text-xs text-gray-500">{{ agent.description }}</div>
                </div>
                <label :for="`agent-model-${index}`" class="sr-only">Model for {{ agent.role }}</label>
                <select
                  :id="`agent-model-${index}`"
                  v-model="agent.model"
                  class="w-full sm:w-auto bg-gray-800 border border-gray-700 rounded-lg px-3 py-3 sm:py-1.5 min-h-[44px] sm:min-h-0 text-sm focus:outline-none focus:border-cyan-500 transition-colors motion-reduce:transition-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                  :aria-labelledby="`agent-${index}-label`"
                  :aria-describedby="`agent-${index}-description`"
                >
                  <option value="claude-3.5-sonnet">Claude 3.5 Sonnet</option>
                  <option value="claude-3-opus">Claude 3 Opus</option>
                  <option value="claude-3-haiku">Claude 3 Haiku</option>
                </select>
              </div>
            </div>
          </section>

          <!-- Actions -->
          <div class="flex flex-col-reverse sm:flex-row justify-end gap-3">
            <button
              @click="resetSettings"
              class="w-full sm:w-auto px-4 py-3 sm:py-2 min-h-[44px] sm:min-h-0 text-sm text-gray-400 hover:text-white transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 rounded-lg"
            >
              Reset to Defaults
            </button>
            <button
              @click="saveSettings"
              class="w-full sm:w-auto bg-cyan-600 hover:bg-cyan-500 text-white px-4 py-3 sm:py-2 min-h-[44px] sm:min-h-0 rounded-lg text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
            >
              Save Changes
            </button>
          </div>
        </div>
      </div>
    </template>

    <!-- Delete Confirmation Modal -->
    <Teleport to="body">
      <Transition name="fade">
        <div v-if="showDeleteModal" class="fixed inset-0 z-50 flex items-center justify-center p-4" role="dialog" aria-modal="true" aria-labelledby="delete-modal-title" aria-describedby="delete-modal-description">
          <!-- Backdrop -->
          <div 
            class="absolute inset-0 bg-black/70 backdrop-blur-sm"
            @click="showDeleteModal = false"
            aria-hidden="true"
          />
          <!-- Modal -->
          <div class="relative bg-gray-900 rounded-xl border border-gray-700 shadow-2xl max-w-md w-full overflow-hidden">
            <!-- Header -->
            <div class="px-6 py-4 border-b border-gray-800 flex items-center gap-3">
              <div class="w-10 h-10 rounded-full bg-red-900/50 flex items-center justify-center" aria-hidden="true">
                <svg class="w-5 h-5 text-red-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              </div>
              <div>
                <h3 id="delete-modal-title" class="text-lg font-semibold text-white">Delete Project</h3>
                <p class="text-sm text-gray-400">This action cannot be undone</p>
              </div>
            </div>
            <!-- Body -->
            <div class="px-6 py-4">
              <p id="delete-modal-description" class="text-gray-300">
                Are you sure you want to delete <span class="font-semibold text-white">{{ route.params.id }}</span>? 
                All files, history, and configuration will be permanently removed.
              </p>
              <div class="mt-4 p-3 bg-red-950/30 border border-red-900/50 rounded-lg">
                <p class="text-sm text-red-300">
                  Type the project ID to confirm:
                </p>
                <label for="delete-confirmation-input" class="sr-only">Type project ID to confirm deletion</label>
                <input
                  id="delete-confirmation-input"
                  v-model="deleteConfirmation"
                  type="text"
                  class="mt-2 w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm font-mono focus:outline-none focus:border-red-500 focus-visible:ring-2 focus-visible:ring-red-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                  :placeholder="String(route.params.id)"
                  :aria-describedby="deleteConfirmation !== route.params.id ? 'delete-hint' : undefined"
                />
                <p v-if="deleteConfirmation && deleteConfirmation !== route.params.id" id="delete-hint" class="sr-only">Project ID does not match</p>
              </div>
            </div>
            <!-- Footer -->
            <div class="px-6 py-4 bg-gray-900/50 border-t border-gray-800 flex justify-end gap-3">
              <button
                @click="showDeleteModal = false"
                class="px-4 py-2 text-sm text-gray-400 hover:text-white transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900 rounded-lg"
              >
                Cancel
              </button>
              <button
                @click="deleteProject"
                :disabled="deleteConfirmation !== route.params.id"
                class="px-4 py-2 text-sm rounded-lg transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
                :class="deleteConfirmation === route.params.id
                  ? 'bg-red-600 hover:bg-red-500 text-white'
                  : 'bg-gray-800 text-gray-500 cursor-not-allowed'"
                :aria-disabled="deleteConfirmation !== route.params.id"
              >
                Delete Permanently
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Toast Notification -->
    <Transition name="slide-up">
      <div
        v-if="toast"
        class="fixed bottom-4 right-4 rounded-lg px-4 py-3 shadow-lg flex items-center gap-3 z-50"
        :class="{
          'bg-green-900/90 border border-green-700': toast.type === 'success',
          'bg-red-900/90 border border-red-700': toast.type === 'error',
          'bg-cyan-900/90 border border-cyan-700': toast.type === 'info',
        }"
        role="alert"
      >
        <svg v-if="toast.type === 'success'" class="w-5 h-5 text-green-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
        <svg v-else-if="toast.type === 'error'" class="w-5 h-5 text-red-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <span class="text-sm" :class="{
          'text-green-200': toast.type === 'success',
          'text-red-200': toast.type === 'error',
          'text-cyan-200': toast.type === 'info',
        }">{{ toast.message }}</span>
        <button
          @click="toast = null"
          class="shrink-0 transition-colors motion-reduce:transition-none"
          :class="{
            'text-green-400 hover:text-green-300': toast.type === 'success',
            'text-red-400 hover:text-red-300': toast.type === 'error',
            'text-cyan-400 hover:text-cyan-300': toast.type === 'info',
          }"
          aria-label="Dismiss notification"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { marked } from 'marked'
import { h, defineComponent, type Component } from 'vue'

const route = useRoute()

// Tab configuration with icons
const ChatIcon = defineComponent({
  render() {
    return h('svg', { class: 'w-4 h-4', fill: 'none', stroke: 'currentColor', viewBox: '0 0 24 24', 'aria-hidden': true }, [
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z' })
    ])
  }
})

const BuildIcon = defineComponent({
  render() {
    return h('svg', { class: 'w-4 h-4', fill: 'none', stroke: 'currentColor', viewBox: '0 0 24 24', 'aria-hidden': true }, [
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2' })
    ])
  }
})

const DiffIcon = defineComponent({
  render() {
    return h('svg', { class: 'w-4 h-4', fill: 'none', stroke: 'currentColor', viewBox: '0 0 24 24', 'aria-hidden': true }, [
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z' })
    ])
  }
})

const FilesIcon = defineComponent({
  render() {
    return h('svg', { class: 'w-4 h-4', fill: 'none', stroke: 'currentColor', viewBox: '0 0 24 24', 'aria-hidden': true }, [
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z' })
    ])
  }
})

const SettingsIcon = defineComponent({
  render() {
    return h('svg', { class: 'w-4 h-4', fill: 'none', stroke: 'currentColor', viewBox: '0 0 24 24', 'aria-hidden': true }, [
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z' }),
      h('path', { 'stroke-linecap': 'round', 'stroke-linejoin': 'round', 'stroke-width': '2', d: 'M15 12a3 3 0 11-6 0 3 3 0 016 0z' })
    ])
  }
})

const tabs: { id: string; label: string; icon: Component; badge?: number }[] = [
  { id: 'chat', label: 'Chat', icon: ChatIcon },
  { id: 'build-log', label: 'Build Log', icon: BuildIcon },
  { id: 'diff', label: 'Diff', icon: DiffIcon },
  { id: 'files', label: 'Files', icon: FilesIcon },
  { id: 'settings', label: 'Settings', icon: SettingsIcon },
]

const activeTab = ref('chat')
const input = ref('')
const chatEl = ref<HTMLElement>()
const chatInputRef = ref<HTMLTextAreaElement>()

const { messages, logLines, diffContent, connected, send } = useWebSocket()

const projectId = computed(() => route.params.id as string)
const projectMessages = computed(() => messages.value.filter(m => !m.projectId || m.projectId === projectId.value))

// Loading and error states
const isLoading = ref(true)
const loadError = ref<string | null>(null)

// Project info (would be fetched from API)
const projectInfo = ref<{ name?: string; template?: string } | null>(null)

// Archive and delete state
const isArchived = ref(false)
const showDeleteModal = ref(false)
const deleteConfirmation = ref('')

// Toast notification
const toast = ref<{ type: 'success' | 'error' | 'info'; message: string } | null>(null)

// Chat suggestions
const chatSuggestions = [
  'Create a new REST API endpoint',
  'Add unit tests for the main module',
  'Refactor the configuration system',
  'Set up CI/CD pipeline',
]

// Diff view state
const diffViewMode = ref<'unified' | 'split'>('unified')
const expandedHunks = ref(new Set([0])) // First hunk expanded by default

const diffStats = computed(() => {
  if (!diffContent.value) return null
  const lines = diffContent.value.split('\n')
  const additions = lines.filter(l => l.startsWith('+')).length
  const deletions = lines.filter(l => l.startsWith('-')).length
  return { additions, deletions }
})

interface DiffLine {
  content: string
  isAddition: boolean
  isDeletion: boolean
  oldLine?: number
  newLine?: number
}

interface DiffHunk {
  header: string
  lines: DiffLine[]
}

const diffHunks = computed<DiffHunk[]>(() => {
  if (!diffContent.value) return []
  // Simplified diff parsing - would be more robust in production
  return [{
    header: '@@ -1,10 +1,12 @@',
    lines: diffContent.value.split('\n').map((line, i) => ({
      content: line.replace(/^[+-]/, ''),
      isAddition: line.startsWith('+'),
      isDeletion: line.startsWith('-'),
      oldLine: line.startsWith('+') ? undefined : i + 1,
      newLine: line.startsWith('-') ? undefined : i + 1,
    }))
  }]
})

const splitDiffLines = computed(() => {
  const left: { num?: number; content: string; isChange: boolean }[] = []
  const right: { num?: number; content: string; isChange: boolean }[] = []
  
  if (!diffContent.value) return { left, right }
  
  diffContent.value.split('\n').forEach((line, i) => {
    if (line.startsWith('-')) {
      left.push({ num: i + 1, content: line.slice(1), isChange: true })
    } else if (line.startsWith('+')) {
      right.push({ num: i + 1, content: line.slice(1), isChange: true })
    } else {
      left.push({ num: i + 1, content: line, isChange: false })
      right.push({ num: i + 1, content: line, isChange: false })
    }
  })
  
  return { left, right }
})

const highlightSyntax = (content: string, _isAddition: boolean, _isDeletion: boolean) => {
  // Simple syntax highlighting - would use a proper library in production
  return content
    .replace(/\b(fn|let|const|var|if|else|for|while|return|import|export|from|async|await|function|class|interface|type)\b/g, '<span class="text-purple-400">$1</span>')
    .replace(/\b(string|number|boolean|void|null|undefined|any|never)\b/g, '<span class="text-cyan-400">$1</span>')
    .replace(/"([^"]*)"/g, '<span class="text-green-400">"$1"</span>')
    .replace(/'([^']*)'/g, "<span class=\"text-green-400\">'$1'</span>")
}

const highlightLogLine = (line: string) => {
  // Highlight common patterns in build logs
  return line
    .replace(/\b(error|ERROR|failed|FAILED)\b/g, '<span class="text-red-400 font-semibold">$1</span>')
    .replace(/\b(warning|WARNING|warn|WARN)\b/g, '<span class="text-yellow-400 font-semibold">$1</span>')
    .replace(/\b(ok|OK|success|SUCCESS|passed|PASSED)\b/g, '<span class="text-green-400 font-semibold">$1</span>')
    .replace(/\b(info|INFO|debug|DEBUG)\b/g, '<span class="text-gray-400">$1</span>')
    .replace(/\[([^\]]+)\]/g, '<span class="text-cyan-400">[$1]</span>')
}

const getLogLineClass = (line: string) => {
  if (line.toLowerCase().includes('error') || line.includes('FAILED')) {
    return 'text-red-300 bg-red-950/20'
  }
  if (line.toLowerCase().includes('warning') || line.toLowerCase().includes('warn')) {
    return 'text-yellow-300 bg-yellow-950/20'
  }
  if (line.toLowerCase().includes('ok') || line.includes('PASSED') || line.toLowerCase().includes('success')) {
    return 'text-green-300'
  }
  return 'text-gray-300'
}

const toggleHunk = (idx: number) => {
  if (expandedHunks.value.has(idx)) {
    expandedHunks.value.delete(idx)
  } else {
    expandedHunks.value.add(idx)
  }
}

const toggleArchive = () => {
  isArchived.value = !isArchived.value
  showToast('success', isArchived.value ? 'Project archived' : 'Project restored')
}

const deleteProject = () => {
  if (deleteConfirmation.value === route.params.id) {
    // Would call API to delete
    showDeleteModal.value = false
    navigateTo('/')
  }
}

const showToast = (type: 'success' | 'error' | 'info', message: string) => {
  toast.value = { type, message }
  setTimeout(() => {
    toast.value = null
  }, 3000)
}

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp)
  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const autoResize = () => {
  const textarea = chatInputRef.value
  if (textarea) {
    textarea.style.height = 'auto'
    textarea.style.height = `${Math.min(textarea.scrollHeight, 128)}px`
  }
}

// File tree state
interface FileItem {
  name: string
  path: string
  type: 'file' | 'folder'
  size?: string
  content?: string
  children?: FileItem[]
}

const expandedFolders = ref(new Set<string>())
const selectedFile = ref<FileItem | null>(null)
const fileCopied = ref(false)
const logsCopied = ref(false)

// Placeholder file tree
const fileTree = ref<FileItem[]>([
  {
    name: 'src',
    path: 'src',
    type: 'folder',
    children: [
      { name: 'main.rs', path: 'src/main.rs', type: 'file', size: '2.1 KB', content: 'fn main() {\n    println!("Hello, Cuttlefish!");\n}' },
      { name: 'lib.rs', path: 'src/lib.rs', type: 'file', size: '4.5 KB', content: '// Library root\npub mod utils;\npub mod config;' },
      { name: 'config.rs', path: 'src/config.rs', type: 'file', size: '1.8 KB', content: 'pub struct Config {\n    pub name: String,\n}' },
    ]
  },
  {
    name: 'tests',
    path: 'tests',
    type: 'folder',
    children: [
      { name: 'integration_test.rs', path: 'tests/integration_test.rs', type: 'file', size: '3.2 KB', content: '#[test]\nfn test_example() {\n    assert!(true);\n}' },
    ]
  },
  { name: 'Cargo.toml', path: 'Cargo.toml', type: 'file', size: '0.5 KB', content: '[package]\nname = "my-project"\nversion = "0.1.0"\nedition = "2021"' },
  { name: 'README.md', path: 'README.md', type: 'file', size: '0.3 KB', content: '# My Project\n\nA Cuttlefish project.' },
])

// Settings state - populated from API
const settings = ref({
  name: '',
  description: '',
  memoryLimit: 2048,
  cpuLimit: 2,
  diskLimit: 10,
  networkAccess: 'enabled',
})

const agentSettings = ref([
  { role: 'Orchestrator', description: 'Task decomposition and coordination', model: 'claude-3.5-sonnet' },
  { role: 'Coder', description: 'Code generation and execution', model: 'claude-3.5-sonnet' },
  { role: 'Critic', description: 'Code review and testing', model: 'claude-3.5-sonnet' },
])

// File icon component
const FileIcon = defineComponent({
  props: { filename: { type: String, required: true } },
  setup(props) {
    const getIconColor = () => {
      const ext = props.filename.split('.').pop()?.toLowerCase()
      const colors: Record<string, string> = {
        rs: 'text-orange-400',
        ts: 'text-blue-400',
        js: 'text-yellow-400',
        vue: 'text-green-400',
        json: 'text-yellow-300',
        md: 'text-gray-400',
        toml: 'text-purple-400',
        yaml: 'text-pink-400',
        yml: 'text-pink-400',
      }
      return colors[ext || ''] || 'text-gray-400'
    }
    return () => h('svg', {
      class: `w-4 h-4 ${getIconColor()}`,
      fill: 'none',
      stroke: 'currentColor',
      viewBox: '0 0 24 24',
      innerHTML: '<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />'
    })
  }
})

const getFileLanguage = (filename: string) => {
  const ext = filename.split('.').pop()?.toLowerCase()
  const languages: Record<string, string> = {
    rs: 'rust',
    ts: 'typescript',
    js: 'javascript',
    vue: 'vue',
    json: 'json',
    md: 'markdown',
    toml: 'toml',
    yaml: 'yaml',
    yml: 'yaml',
  }
  return languages[ext || ''] || 'text'
}

const toggleFolder = (path: string) => {
  if (expandedFolders.value.has(path)) {
    expandedFolders.value.delete(path)
  } else {
    expandedFolders.value.add(path)
  }
}

const selectFile = (file: FileItem) => {
  selectedFile.value = file
  fileCopied.value = false
}

const copyFileContent = async () => {
  if (selectedFile.value?.content) {
    await navigator.clipboard.writeText(selectedFile.value.content)
    fileCopied.value = true
    setTimeout(() => { fileCopied.value = false }, 2000)
  }
}

const copyLogs = async () => {
  await navigator.clipboard.writeText(logLines.value.join('\n'))
  logsCopied.value = true
  setTimeout(() => { logsCopied.value = false }, 2000)
}

const renderMarkdown = (text: string) => {
  try { return marked(text) } catch { return text }
}

const sendMessage = () => {
  if (!input.value.trim()) return
  send(projectId.value, input.value)
  input.value = ''
  if (chatInputRef.value) {
    chatInputRef.value.style.height = 'auto'
  }
}

const saveSettings = () => {
  // Would save to API - implementation pending
  showToast('success', 'Settings saved successfully')
}

const resetSettings = () => {
  if (projectInfo.value) {
    settings.value = {
      name: projectInfo.value.name,
      description: '',
      memoryLimit: 2048,
      cpuLimit: 2,
      diskLimit: 10,
      networkAccess: 'enabled',
    }
  }
  agentSettings.value.forEach(a => a.model = 'claude-3.5-sonnet')
  showToast('info', 'Settings reset to defaults')
}

const loadProject = async () => {
  isLoading.value = true
  loadError.value = null
  
  try {
    const config = useRuntimeConfig()
    const res = await $fetch<{ name: string; description?: string; template?: string; isArchived?: boolean }>(
      `${config.public.apiBase}/api/projects/${projectId.value}`
    )
    projectInfo.value = res
    isArchived.value = res.isArchived || false
    settings.value.name = res.name
    settings.value.description = res.description || ''
  } catch (e) {
    console.error('Failed to load project', e)
    loadError.value = 'Could not load project details. Please try again.'
  } finally {
    isLoading.value = false
  }
}

// Load project on mount
onMounted(loadProject)

// Auto-scroll chat
watch(() => projectMessages.value.length, async () => {
  await nextTick()
  if (chatEl.value) chatEl.value.scrollTop = chatEl.value.scrollHeight
})

// Keyboard shortcuts
onMounted(() => {
  const handleKeydown = (e: KeyboardEvent) => {
    // Ctrl/Cmd + 1-5 to switch tabs
    if ((e.ctrlKey || e.metaKey) && ['1', '2', '3', '4', '5'].includes(e.key)) {
      e.preventDefault()
      const tabIdx = parseInt(e.key) - 1
      if (tabs[tabIdx]) {
        activeTab.value = tabs[tabIdx].id
      }
    }
    // Escape to close modal
    if (e.key === 'Escape' && showDeleteModal.value) {
      showDeleteModal.value = false
    }
  }
  window.addEventListener('keydown', handleKeydown)
  onUnmounted(() => window.removeEventListener('keydown', handleKeydown))
})
</script>

<style scoped>
/* Transitions */
.message-list-enter-active,
.message-list-leave-active {
  transition: all 0.3s ease;
}

.message-list-enter-from,
.message-list-leave-to {
  opacity: 0;
  transform: translateY(10px);
}

.slide-up-enter-active,
.slide-up-leave-active {
  transition: all 0.3s ease;
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
  transform: translateY(20px);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
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

/* Focus visible styles for keyboard navigation */
.focus-visible:focus-visible,
button:focus-visible,
a:focus-visible,
input:focus-visible,
select:focus-visible,
textarea:focus-visible {
  outline: 2px solid #00d4aa;
  outline-offset: 2px;
}

/* Remove default focus outline when using focus-visible */
button:focus:not(:focus-visible),
a:focus:not(:focus-visible),
input:focus:not(:focus-visible),
select:focus:not(:focus-visible),
textarea:focus:not(:focus-visible) {
  outline: none;
}

/* Scrollbar styling */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: transparent;
}

::-webkit-scrollbar-thumb {
  background: #374151;
  border-radius: 4px;
}

::-webkit-scrollbar-thumb:hover {
  background: #4b5563;
}
</style>