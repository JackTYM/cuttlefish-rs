<script setup lang="ts">
/**
 * GlitchText - Text with CSS-only glitch effect on hover
 * Creates a cyberpunk-style glitch animation using clip-path and transforms
 */
const props = defineProps<{
  /** The text to display */
  text: string
  /** Glitch intensity (1-3, higher = more intense) */
  intensity?: 1 | 2 | 3
}>()

const intensityClass = computed(() => `glitch-intensity-${props.intensity || 2}`)
</script>

<template>
  <span 
    class="glitch-text relative inline-block" 
    :class="intensityClass"
  >
    <!-- Main text -->
    <span class="relative z-10">{{ text }}</span>
    
    <!-- Glitch layers (CSS pseudo-elements via class) -->
    <span 
      class="glitch-layer glitch-layer-1 absolute inset-0 text-cyan-400"
      aria-hidden="true"
    >{{ text }}</span>
    <span 
      class="glitch-layer glitch-layer-2 absolute inset-0 text-red-400"
      aria-hidden="true"
    >{{ text }}</span>
  </span>
</template>

<style scoped>
.glitch-text {
  --glitch-clip-1: inset(100% 0 0 0);
  --glitch-clip-2: inset(0 0 100% 0);
  --glitch-offset: 2px;
}

.glitch-text:hover .glitch-layer {
  animation: glitch 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94) both infinite;
}

.glitch-text:hover .glitch-layer-2 {
  animation-delay: 0.05s;
}

.glitch-layer {
  pointer-events: none;
  opacity: 0;
  transition: opacity 0.1s;
}

.glitch-text:hover .glitch-layer {
  opacity: 0.8;
}

/* Intensity variations */
.glitch-intensity-1 {
  --glitch-offset: 1px;
}
.glitch-intensity-1:hover .glitch-layer {
  animation-duration: 0.4s;
}

.glitch-intensity-2 {
  --glitch-offset: 2px;
}

.glitch-intensity-3 {
  --glitch-offset: 4px;
}
.glitch-intensity-3:hover .glitch-layer {
  animation-duration: 0.15s;
}

@keyframes glitch {
  0% {
    transform: translate(0);
    clip-path: var(--glitch-clip-1);
  }
  5% {
    transform: translate(calc(var(--glitch-offset) * -1), calc(var(--glitch-offset) * 0.5));
    clip-path: inset(40% 0 20% 0);
  }
  10% {
    transform: translate(var(--glitch-offset), calc(var(--glitch-offset) * -0.5));
    clip-path: inset(10% 0 60% 0);
  }
  15% {
    transform: translate(calc(var(--glitch-offset) * -0.5), var(--glitch-offset));
    clip-path: inset(80% 0 5% 0);
  }
  20% {
    transform: translate(0, 0);
    clip-path: var(--glitch-clip-2);
  }
  100% {
    transform: translate(0, 0);
    clip-path: var(--glitch-clip-2);
  }
}
</style>