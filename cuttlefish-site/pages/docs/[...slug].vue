<template>
  <div class="flex min-h-screen bg-slate-950">
    <!-- Sidebar -->
    <aside class="w-64 bg-slate-900 p-4 border-r border-slate-800 sticky top-0 h-screen overflow-y-auto">
      <nav class="space-y-2">
        <NuxtLink 
          v-for="doc in navigation" 
          :key="doc._path"
          :to="doc._path"
          class="block px-3 py-2 rounded text-slate-300 hover:bg-slate-800 hover:text-white transition-colors"
          :class="{ 'bg-slate-800 text-white': isActive(doc._path) }"
        >
          {{ doc.title }}
        </NuxtLink>
      </nav>
    </aside>
    
    <!-- Content -->
    <main class="flex-1 p-8 max-w-4xl">
      <ContentDoc class="prose prose-invert max-w-none" />
    </main>
  </div>
</template>

<script setup>
const route = useRoute()

const { data: navigation } = await useAsyncData('navigation', () => 
  queryContent('docs').only(['title', '_path']).find()
)

const isActive = (path) => {
  return route.path === path
}
</script>

<style scoped>
:deep(.prose) {
  --tw-prose-body: rgb(226 232 240);
  --tw-prose-headings: rgb(248 250 252);
  --tw-prose-links: rgb(34 211 238);
  --tw-prose-code: rgb(226 232 240);
  --tw-prose-pre-bg: rgb(15 23 42);
  --tw-prose-pre-code: rgb(226 232 240);
}

:deep(.prose code) {
  background-color: rgb(30 41 59);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
}

:deep(.prose pre) {
  background-color: rgb(15 23 42);
  border: 1px solid rgb(51 65 85);
}

:deep(.prose a) {
  color: rgb(34 211 238);
  text-decoration: underline;
}

:deep(.prose a:hover) {
  color: rgb(6 182 212);
}
</style>
