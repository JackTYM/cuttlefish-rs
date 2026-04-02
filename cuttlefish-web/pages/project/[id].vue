<template>
  <div class="flex flex-col h-full">
    <!-- Project Header with Tabs -->
    <header class="bg-gray-900 border-b border-gray-800 px-6 py-3 flex items-center gap-4 shrink-0">
      <NuxtLink to="/" class="text-cyan-400 hover:text-cyan-300 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded" aria-label="Back to projects">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 19l-7-7m0 0l7-7m-7 7h18" />
        </svg>
      </NuxtLink>
      <h1 class="text-lg font-semibold" :class="{ 'text-gray-500 line-through': isArchived }">{{ route.params.id }}</h1>
      <span v-if="projectInfo?.template" class="text-xs px-2 py-0.5 rounded-full bg-purple-900/50 text-purple-300 border border-purple-700/50">
        {{ projectInfo.template }}
      </span>
      <span v-if="isArchived" class="text-xs px-2 py-0.5 rounded-full bg-yellow-900/50 text-yellow-300 border border-yellow-700/50" role="status">
        Archived
      </span>
      <!-- Action buttons -->
      <div class="ml-auto flex items-center gap-2">
        <button
          @click="toggleArchive"
          class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
          :class="isArchived 
            ? 'bg-green-900/50 text-green-400 hover:bg-green-800/50 border border-green-700/50' 
            : 'bg-yellow-900/30 text-yellow-400 hover:bg-yellow-900/50 border border-yellow-700/30'"
          :aria-label="isArchived ? 'Restore project from archive' : 'Archive project'"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path v-if="isArchived" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            <path v-else stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
          </svg>
          {{ isArchived ? 'Restore' : 'Archive' }}
        </button>
        <button
          @click="showDeleteModal = true"
          class="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-lg bg-red-900/30 text-red-400 hover:bg-red-900/50 border border-red-700/30 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-red-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
          aria-label="Delete project permanently"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          Delete
        </button>
      </div>
    </header>

    <nav class="bg-gray-900 border-b border-gray-800 px-6 flex gap-1 shrink-0" aria-label="Project sections" role="tablist">
      <button
        v-for="tab in tabs"
        :key="tab"
        @click="activeTab = tab"
        class="px-4 py-2 text-sm font-medium transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
        :class="activeTab === tab ? 'text-cyan-400 border-b-2 border-cyan-400' : 'text-gray-400 hover:text-white'"
        :aria-selected="activeTab === tab"
        :tabindex="activeTab === tab ? 0 : -1"
        role="tab"
        :id="`tab-${tab.toLowerCase().replace(' ', '-')}`"
        :aria-controls="`panel-${tab.toLowerCase().replace(' ', '-')}`"
      >
        {{ tab }}
      </button>
    </nav>

    <!-- Chat Tab -->
    <div v-if="activeTab === 'Chat'" class="flex flex-col flex-1 overflow-hidden" role="tabpanel" id="panel-chat" aria-labelledby="tab-chat">
      <div ref="chatEl" class="flex-1 overflow-y-auto p-6 space-y-4" aria-live="polite" aria-label="Chat messages">
        <div v-for="(msg, i) in projectMessages" :key="i" class="flex gap-3" role="article" :aria-label="`Message from ${msg.sender}`">
          <div
            class="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold shrink-0"
            :class="{
              'bg-cyan-700': msg.sender === 'user',
              'bg-purple-700': msg.sender === 'orchestrator',
              'bg-yellow-700': msg.sender === 'coder',
              'bg-red-700': msg.sender === 'critic',
              'bg-gray-700': !['user','orchestrator','coder','critic'].includes(msg.sender)
            }"
            aria-hidden="true"
          >{{ msg.sender[0]?.toUpperCase() }}</div>
          <div class="flex-1">
            <div class="text-xs text-gray-500 mb-1">{{ msg.sender }}</div>
            <div class="prose prose-invert prose-sm max-w-none text-gray-200" v-html="renderMarkdown(msg.content)" />
          </div>
        </div>
      </div>
      <div class="border-t border-gray-800 p-4 flex gap-3 shrink-0">
        <label for="chat-input" class="sr-only">Message input</label>
        <input
          id="chat-input"
          v-model="input"
          @keyup.enter="sendMessage"
          placeholder="Describe what you want to build..."
          class="flex-1 bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm focus:outline-none focus:border-cyan-500 focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900"
        />
        <button @click="sendMessage" class="bg-cyan-600 hover:bg-cyan-500 text-white px-4 py-2 rounded-lg text-sm transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-gray-900" aria-label="Send message">Send</button>
      </div>
    </div>

    <!-- Build Log Tab -->
    <div v-if="activeTab === 'Build Log'" class="flex-1 overflow-y-auto p-6 font-mono text-sm bg-black" role="tabpanel" id="panel-build-log" aria-labelledby="tab-build-log" aria-label="Build log output">
      <div v-for="(line, i) in logLines" :key="i"
        class="leading-5"
        :class="{
          'text-red-400': line.includes('error') || line.includes('FAILED'),
          'text-yellow-400': line.includes('warning'),
          'text-green-400': line.includes('ok') || line.includes('PASSED'),
          'text-gray-300': !line.includes('error') && !line.includes('warning') && !line.includes('ok'),
        }"
      >{{ line }}</div>
      <div v-if="!logLines.length" class="text-gray-500 text-center py-8">No build logs yet</div>
    </div>

    <!-- Diff Tab -->
    <div v-if="activeTab === 'Diff'" class="flex-1 overflow-hidden flex flex-col" role="tabpanel" id="panel-diff" aria-labelledby="tab-diff">
      <!-- Diff Header -->
      <div class="bg-gray-900 border-b border-gray-800 px-6 py-3 flex items-center justify-between shrink-0">
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
      <div class="flex-1 overflow-y-auto font-mono text-sm bg-gray-950" aria-label="Diff content">
        <div v-if="!diffContent" class="flex items-center justify-center h-full text-gray-500">
          <div class="text-center">
            <svg class="w-12 h-12 mx-auto mb-3 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
            <p>No changes yet</p>
          </div>
        </div>
        <!-- Unified View -->
        <div v-else-if="diffViewMode === 'unified'" class="divide-y divide-gray-800">
          <div v-for="(hunk, hunkIdx) in diffHunks" :key="hunkIdx">
            <!-- Hunk Header -->
            <div class="px-4 py-2 bg-gray-900 text-cyan-400 text-xs sticky top-0 border-b border-gray-800">
              {{ hunk.header }}
            </div>
            <!-- Hunk Lines -->
            <div class="relative">
              <div v-for="(line, i) in hunk.lines" :key="i" class="flex group">
                <!-- Line Number -->
                <div class="w-12 shrink-0 text-right pr-3 py-0.5 text-xs select-none"
                  :class="{
                    'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                    'text-green-700 bg-green-950/30': line.isAddition,
                    'text-red-700 bg-red-950/30': line.isDeletion,
                  }"
                >{{ line.oldLine || '' }}</div>
                <div class="w-12 shrink-0 text-right pr-3 py-0.5 text-xs select-none border-r border-gray-800"
                  :class="{
                    'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                    'text-green-700 bg-green-950/30': line.isAddition,
                    'text-red-700 bg-red-950/30': line.isDeletion,
                  }"
                >{{ line.newLine || '' }}</div>
                <!-- Diff Indicator -->
                <div class="w-6 shrink-0 text-center py-0.5 text-xs"
                  :class="{
                    'text-gray-600 bg-gray-900/50': !line.isAddition && !line.isDeletion,
                    'text-green-400 bg-green-950/50': line.isAddition,
                    'text-red-400 bg-red-950/50': line.isDeletion,
                  }"
                >{{ line.isAddition ? '+' : line.isDeletion ? '-' : ' ' }}</div>
                <!-- Line Content -->
                <div class="flex-1 px-3 py-0.5 whitespace-pre"
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
              <div class="w-12 shrink-0 text-right pr-3 py-0.5 text-xs text-gray-600 bg-gray-900/30">{{ line.num || '' }}</div>
              <div class="flex-1 px-3 py-0.5 whitespace-pre text-sm"
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
              <div class="w-12 shrink-0 text-right pr-3 py-0.5 text-xs text-gray-600 bg-gray-900/30">{{ line.num || '' }}</div>
              <div class="flex-1 px-3 py-0.5 whitespace-pre text-sm"
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
    <div v-if="activeTab === 'Files'" class="flex-1 overflow-hidden flex" role="tabpanel" id="panel-files" aria-labelledby="tab-files">
      <!-- File Tree -->
      <nav class="w-64 bg-gray-900 border-r border-gray-800 overflow-y-auto shrink-0" aria-label="File explorer">
        <div class="p-3 border-b border-gray-800 flex items-center justify-between">
          <span class="text-sm font-medium text-gray-300">Files</span>
          <button class="text-gray-500 hover:text-cyan-400 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 rounded" aria-label="Add new file">
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
                class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-sm hover:bg-gray-800 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
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
                <span>{{ item.name }}</span>
              </button>
              <!-- Children -->
              <div v-if="expandedFolders.has(item.path)" class="ml-4" role="group">
                <button
                  v-for="child in item.children"
                  :key="child.path"
                  @click="selectFile(child)"
                  class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-sm hover:bg-gray-800 transition-colors motion-reduce:transition-none focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
                  :class="selectedFile?.path === child.path ? 'text-cyan-400 bg-cyan-950/30' : 'text-gray-400'"
                  role="treeitem"
                  :aria-selected="selectedFile?.path === child.path"
                  :aria-label="`${child.name} file`"
                >
                  <FileIcon :filename="child.name" />
                  <span>{{ child.name }}</span>
                </button>
              </div>
            </div>
            <!-- Root-level file -->
            <button
              v-else
              @click="selectFile(item)"
              class="w-full flex items-center gap-2 px-2 py-1.5 rounded text-sm hover:bg-gray-800 transition-colors motion-reduce:transition-none mb-1 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400"
              :class="selectedFile?.path === item.path ? 'text-cyan-400 bg-cyan-950/30' : 'text-gray-400'"
              role="treeitem"
              :aria-selected="selectedFile?.path === item.path"
              :aria-label="`${item.name} file`"
            >
              <FileIcon :filename="item.name" />
              <span>{{ item.name }}</span>
            </button>
          </template>
        </div>
      </nav>
      <!-- File Content -->
      <main class="flex-1 overflow-y-auto bg-gray-950" aria-label="File content viewer">
        <div v-if="selectedFile" class="p-4">
          <div class="flex items-center justify-between mb-3 pb-3 border-b border-gray-800">
            <div class="flex items-center gap-2">
              <FileIcon :filename="selectedFile.name" />
              <span class="text-sm font-medium text-gray-200">{{ selectedFile.path }}</span>
            </div>
            <span class="text-xs text-gray-500">{{ selectedFile.size }}</span>
          </div>
          <pre class="font-mono text-sm text-gray-300 whitespace-pre-wrap" aria-label="File contents">{{ selectedFile.content }}</pre>
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
    <div v-if="activeTab === 'Settings'" class="flex-1 overflow-y-auto p-6" role="tabpanel" id="panel-settings" aria-labelledby="tab-settings">
      <div class="max-w-2xl mx-auto space-y-6">
        <!-- Project Info -->
        <section class="bg-gray-900 rounded-xl border border-gray-800 p-5" aria-labelledby="project-info-heading">
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

    <!-- Delete Confirmation Modal -->
    <Teleport to="body">
      <div v-if="showDeleteModal" class="fixed inset-0 z-50 flex items-center justify-center" role="dialog" aria-modal="true" aria-labelledby="delete-modal-title" aria-describedby="delete-modal-description">
        <!-- Backdrop -->
        <div 
          class="absolute inset-0 bg-black/70 backdrop-blur-sm"
          @click="showDeleteModal = false"
          aria-hidden="true"
        />
        <!-- Modal -->
        <div class="relative bg-gray-900 rounded-xl border border-gray-700 shadow-2xl max-w-md w-full mx-4 overflow-hidden">
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
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { marked } from 'marked'
import { h, defineComponent } from 'vue'

const route = useRoute()
const tabs = ['Chat', 'Build Log', 'Diff', 'Files', 'Settings']
const activeTab = ref('Chat')
const input = ref('')
const chatEl = ref<HTMLElement>()

const { messages, logLines, diffContent, connected, send } = useWebSocket()

const projectId = computed(() => route.params.id as string)
const projectMessages = computed(() => messages.value.filter(m => !m.projectId || m.projectId === projectId.value))
const diffLines = computed(() => diffContent.value.split('\n'))

// Project info (would be fetched from API)
const projectInfo = ref<{ template?: string } | null>(null)

// Archive and delete state
const isArchived = ref(false)
const showDeleteModal = ref(false)
const deleteConfirmation = ref('')

// Diff view state
const diffViewMode = ref<'unified' | 'split'>('unified')
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
}

const toggleArchive = () => {
  isArchived.value = !isArchived.value
}

const deleteProject = () => {
  if (deleteConfirmation.value === route.params.id) {
    // Would call API to delete
    showDeleteModal.value = false
    navigateTo('/')
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

// Settings state
const settings = ref({
  name: 'my-project',
  description: 'A sample project',
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

const toggleFolder = (path: string) => {
  if (expandedFolders.value.has(path)) {
    expandedFolders.value.delete(path)
  } else {
    expandedFolders.value.add(path)
  }
}

const selectFile = (file: FileItem) => {
  selectedFile.value = file
}

const renderMarkdown = (text: string) => {
  try { return marked(text) } catch { return text }
}

const sendMessage = () => {
  if (!input.value.trim()) return
  send(projectId.value, input.value)
  input.value = ''
}

const saveSettings = () => {
  // Would save to API
  console.log('Saving settings:', settings.value, agentSettings.value)
}

const resetSettings = () => {
  settings.value = {
    name: 'my-project',
    description: 'A sample project',
    memoryLimit: 2048,
    cpuLimit: 2,
    diskLimit: 10,
    networkAccess: 'enabled',
  }
  agentSettings.value.forEach(a => a.model = 'claude-3.5-sonnet')
}

watch(() => projectMessages.value.length, async () => {
  await nextTick()
  if (chatEl.value) chatEl.value.scrollTop = chatEl.value.scrollHeight
})
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
</style>