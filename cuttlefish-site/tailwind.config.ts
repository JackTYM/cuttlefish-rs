import type { Config } from 'tailwindcss'
import typography from '@tailwindcss/typography'

export default {
  content: [
    './components/**/*.{js,vue,ts}',
    './layouts/**/*.vue',
    './pages/**/*.vue',
    './plugins/**/*.{js,ts}',
    './nuxt.config.{js,ts}',
  ],
  darkMode: 'class',
  theme: {
    extend: {
      colors: {
        'dark-bg': '#0f172a',
        'dark-card': '#1e293b',
        'dark-border': '#334155',
      },
      fontFamily: {
        mono: ['JetBrains Mono', 'Fira Code', 'SF Mono', 'monospace'],
      },
      typography: {
        invert: { css: { '--tw-prose-pre-bg': '#1e293b' } },
      },
    },
  },
  plugins: [typography],
} satisfies Config
