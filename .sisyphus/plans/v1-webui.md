# V1 WebUI — Marketing Site + Application Dashboard

## TL;DR

> **Quick Summary**: Build a complete web presence with marketing site (cuttlefish.dev) and application dashboard (app.cuttlefish.dev) using Nuxt 3, featuring hacker/dev-tool aesthetics with terminal-inspired design.
> 
> **Deliverables**:
> - Marketing site: Hero, Features, Installation, Documentation, Marketplace pages
> - App dashboard: Project management, Chat, Template browser, Agent logs, Settings
> - Shared component library with terminal aesthetics
> - Subdomain routing configuration
> 
> **Estimated Effort**: Large (4-5 days)
> **Parallel Execution**: YES - 5 waves
> **Critical Path**: Task 1 (Structure) → Tasks 2-5 (Marketing) → Tasks 6-10 (Dashboard) → Task 11 (Deploy)

---

## Context

### Original Request
Build both marketing/landing site and application dashboard as a unified web experience, using true subdomains (`cuttlefish.dev` for marketing, `app.cuttlefish.dev` for app).

### Current State (Research Findings)
**Already exists (`cuttlefish-web/`):**
- Nuxt 3 + Tailwind CSS + Typography plugin
- Project list page (`pages/index.vue`)
- Project detail with Chat/Build Log/Diff tabs (`pages/project/[id].vue`)
- WebSocket composable for real-time updates
- Dark theme foundation (gray-950 background)

**Missing:**
- Marketing pages (Hero, Features, Installation, Documentation)
- Template browser UI
- Agent activity logs viewer
- Settings/configuration UI
- Marketplace page
- Proper navigation structure
- Subdomain routing

### Design Direction
- **Style**: Hacker/Dev Tool — terminal aesthetics, monospace fonts, dark mode heavy
- **Structure**: True subdomains — marketing at root, app at `app.` subdomain

---

## Work Objectives

### Core Objective
Create a professional, developer-focused web presence that markets Cuttlefish effectively while providing a powerful application dashboard for actual usage.

### Concrete Deliverables

**Marketing Site (`cuttlefish-site/`):**
- `pages/index.vue` — Hero with animated terminal demo
- `pages/features.vue` — Feature grid with code examples
- `pages/install.vue` — Installation guide with copy-paste commands
- `pages/docs/[...slug].vue` — Documentation (markdown-driven)
- `pages/marketplace.vue` — Template marketplace browser
- `components/` — Shared terminal-style components

**App Dashboard (`cuttlefish-web/` - enhanced):**
- `pages/index.vue` — Project list (exists, enhance)
- `pages/project/[id].vue` — Project detail (exists, enhance)
- `pages/templates.vue` — Template browser
- `pages/logs.vue` — Agent activity logs
- `pages/settings.vue` — Configuration UI
- `layouts/default.vue` — App shell with sidebar nav

**Shared:**
- `packages/ui/` — Shared component library (optional)
- Terminal-inspired components: CodeBlock, CommandLine, StatusBadge

### Definition of Done
- [ ] Marketing site builds and deploys to `cuttlefish.dev`
- [ ] App dashboard builds and deploys to `app.cuttlefish.dev`
- [ ] All pages render without errors
- [ ] Mobile responsive (min 375px width)
- [ ] Lighthouse accessibility score ≥ 90
- [ ] No console errors in production build

### Must Have
- Animated terminal hero on landing page
- Copy-to-clipboard on all code blocks
- Dark mode only (no light mode toggle)
- Real API integration for dashboard
- WebSocket for real-time updates in app

### Must NOT Have (Guardrails)
- No light mode — dark only
- No JavaScript frameworks other than Vue/Nuxt
- No UI component libraries (Tailwind only) — keep bundle small
- No placeholder "lorem ipsum" content
- No external analytics without consent
- No authentication in marketing site (app uses API key)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: Partial (Nuxt dev server)
- **Automated tests**: YES (Playwright for E2E)
- **Framework**: Playwright for E2E, Vitest for unit

### QA Policy
Every task includes agent-executed QA scenarios using Playwright.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Pages**: Playwright navigation + screenshot
- **Components**: Visual regression
- **API Integration**: Mock server + real calls

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — structure + shared components):
├── Task 1: Create cuttlefish-site/ Nuxt app [quick]
├── Task 2: Shared terminal components (CodeBlock, StatusBadge) [visual-engineering]
└── Task 3: App layout with sidebar navigation [visual-engineering]

Wave 2 (Marketing — landing pages):
├── Task 4: Hero page with terminal animation [visual-engineering]
├── Task 5: Features page [visual-engineering]
├── Task 6: Installation guide page [visual-engineering]
└── Task 7: Documentation structure (markdown) [quick]

Wave 3 (Dashboard — core pages):
├── Task 8: Template browser page [visual-engineering]
├── Task 9: Agent activity logs page [visual-engineering]
├── Task 10: Settings/config page [visual-engineering]
└── Task 11: Enhance existing project pages [visual-engineering]

Wave 4 (Marketplace + Polish):
├── Task 12: Marketplace page (marketing site) [visual-engineering]
├── Task 13: Mobile responsive pass [visual-engineering]
└── Task 14: Accessibility audit + fixes [unspecified-high]

Wave 5 (Deploy):
└── Task 15: Deployment configuration (Cloudflare/Vercel) [quick]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Visual QA with Playwright screenshots (visual-engineering)
├── Task F3: API integration QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 2 → Task 4 → Task 8 → Task 15 → F1-F4 → user okay
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 4 (Waves 2 & 3)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 2, 4-7, 12 | 1 |
| 2 | — | 3, 4-14 | 1 |
| 3 | 2 | 8-11 | 1 |
| 4 | 1, 2 | 12, 13 | 2 |
| 5 | 1, 2 | 13 | 2 |
| 6 | 1, 2 | 13 | 2 |
| 7 | 1 | — | 2 |
| 8 | 2, 3 | 13 | 3 |
| 9 | 2, 3 | 13 | 3 |
| 10 | 2, 3 | 13 | 3 |
| 11 | 2, 3 | 13 | 3 |
| 12 | 1, 2, 4 | 13 | 4 |
| 13 | 4-12 | 14 | 4 |
| 14 | 13 | 15 | 4 |
| 15 | 14 | F1-F4 | 5 |

### Agent Dispatch Summary

- **Wave 1**: 3 tasks — T1 → `quick`, T2 → `visual-engineering`, T3 → `visual-engineering`
- **Wave 2**: 4 tasks — T4-T6 → `visual-engineering`, T7 → `quick`
- **Wave 3**: 4 tasks — T8-T11 → `visual-engineering`
- **Wave 4**: 3 tasks — T12-T13 → `visual-engineering`, T14 → `unspecified-high`
- **Wave 5**: 1 task — T15 → `quick`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `visual-engineering`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Create Marketing Site Nuxt App

  **What to do**:
  - Create `cuttlefish-site/` directory alongside `cuttlefish-web/`
  - Initialize Nuxt 3 with: `npx nuxi@latest init cuttlefish-site`
  - Add Tailwind CSS: `npm install @nuxtjs/tailwindcss @tailwindcss/typography`
  - Configure `nuxt.config.ts` with SSG mode for static hosting
  - Create base `tailwind.config.ts` with dark theme defaults (gray-950 bg, cyan/purple accents)
  - Create `layouts/default.vue` with marketing nav (Features, Install, Docs, Marketplace, "Launch App →")
  - Create placeholder `pages/index.vue`

  **Must NOT do**:
  - Don't use SSR mode (static site generation only)
  - Don't add any UI component libraries

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Scaffolding task, well-defined steps
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 2, 3)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 4-7, 12
  - **Blocked By**: None

  **References**:
  - `cuttlefish-web/nuxt.config.ts` — Existing config pattern
  - `cuttlefish-web/package.json` — Dependencies to mirror
  - `cuttlefish-web/tailwind.config.ts` — Tailwind setup pattern

  **Acceptance Criteria**:
  - [ ] `cd cuttlefish-site && npm run dev` starts without errors
  - [ ] `npm run generate` produces static site in `.output/public/`
  - [ ] Tailwind classes work (test with `bg-gray-950`)

  **QA Scenarios**:
  ```
  Scenario: Dev server starts successfully
    Tool: Bash
    Preconditions: cuttlefish-site/ exists with dependencies installed
    Steps:
      1. Run `cd cuttlefish-site && timeout 30 npm run dev &`
      2. Wait 10 seconds for server startup
      3. Run `curl -s http://localhost:3000 | head -20`
    Expected Result: Returns HTML with Nuxt app shell
    Evidence: .sisyphus/evidence/task-1-dev-server.txt

  Scenario: Static generation works
    Tool: Bash
    Preconditions: Nuxt app configured for SSG
    Steps:
      1. Run `cd cuttlefish-site && npm run generate`
      2. Run `ls -la .output/public/`
    Expected Result: index.html and assets present
    Evidence: .sisyphus/evidence/task-1-static-gen.txt
  ```

  **Commit**: YES
  - Message: `feat(site): initialize marketing site with Nuxt 3`
  - Files: `cuttlefish-site/*`

- [ ] 2. Create Shared Terminal Components

  **What to do**:
  - Create `cuttlefish-web/components/terminal/` directory
  - Create `CodeBlock.vue` — Syntax highlighted code with copy button, line numbers optional
  - Create `CommandLine.vue` — Single command with `$` prefix and copy button
  - Create `StatusBadge.vue` — Colored badges (success/warning/error/info)
  - Create `TerminalWindow.vue` — Window chrome with title bar dots, optional typing animation
  - Create `GlitchText.vue` — Text with glitch hover effect (optional flair)
  - Use monospace font (JetBrains Mono or similar via Google Fonts)
  - All components use Tailwind, no external deps

  **Must NOT do**:
  - Don't use external syntax highlighting library initially (CSS-only approach)
  - Don't make components too complex — keep them composable

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: UI components requiring visual design skills
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1, 3)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 3-14 (all UI tasks use these)
  - **Blocked By**: None

  **References**:
  - Terminal aesthetic examples: Warp, Hyper, iTerm2
  - `cuttlefish-web/pages/project/[id].vue:53-63` — Existing log styling pattern

  **Acceptance Criteria**:
  - [ ] All 5 components render without errors
  - [ ] Copy button copies to clipboard
  - [ ] Components work in both cuttlefish-web and cuttlefish-site

  **QA Scenarios**:
  ```
  Scenario: CodeBlock renders with copy functionality
    Tool: Playwright
    Preconditions: Dev server running, test page with CodeBlock
    Steps:
      1. Navigate to test page with CodeBlock
      2. Click copy button
      3. Verify clipboard contains code content
    Expected Result: Code copied to clipboard, button shows "Copied!"
    Evidence: .sisyphus/evidence/task-2-codeblock.png

  Scenario: TerminalWindow typing animation
    Tool: Playwright
    Preconditions: TerminalWindow with animated prop
    Steps:
      1. Navigate to page with animated TerminalWindow
      2. Wait 2 seconds
      3. Take screenshot mid-animation
    Expected Result: Text appears character by character
    Evidence: .sisyphus/evidence/task-2-terminal-animation.png
  ```

  **Commit**: YES
  - Message: `feat(web): add terminal-style component library`
  - Files: `cuttlefish-web/components/terminal/*`

- [ ] 3. App Layout with Sidebar Navigation

  **What to do**:
  - Create `cuttlefish-web/layouts/default.vue` with sidebar layout
  - Sidebar items: Projects, Templates, Logs, Settings
  - Sidebar collapsible on mobile (hamburger menu)
  - Header with connection status indicator (existing pattern)
  - Main content area with proper scrolling
  - Use terminal aesthetic: dark sidebar, subtle borders, cyan accents
  - Add `layouts/fullscreen.vue` for pages that don't need sidebar (e.g., focused chat)

  **Must NOT do**:
  - Don't change existing page logic, only wrap with layout
  - Don't remove existing header patterns

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: Layout design requiring visual skills
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 1, 2)
  - **Parallel Group**: Wave 1
  - **Blocks**: Tasks 8-11
  - **Blocked By**: Task 2 (needs shared components)

  **References**:
  - `cuttlefish-web/pages/index.vue:3-9` — Existing header pattern
  - VS Code sidebar for inspiration (icons + labels)

  **Acceptance Criteria**:
  - [ ] Sidebar shows on all dashboard pages
  - [ ] Sidebar collapses to icons on narrow screens
  - [ ] Navigation links work correctly
  - [ ] Mobile hamburger menu works

  **QA Scenarios**:
  ```
  Scenario: Sidebar navigation works
    Tool: Playwright
    Preconditions: App running with new layout
    Steps:
      1. Navigate to /
      2. Click "Templates" in sidebar
      3. Verify URL is /templates
    Expected Result: Navigation works, active item highlighted
    Evidence: .sisyphus/evidence/task-3-sidebar-nav.png

  Scenario: Mobile responsive sidebar
    Tool: Playwright
    Preconditions: App running
    Steps:
      1. Set viewport to 375x667 (iPhone SE)
      2. Verify sidebar is hidden
      3. Click hamburger menu
      4. Verify sidebar appears as overlay
    Expected Result: Mobile menu works correctly
    Evidence: .sisyphus/evidence/task-3-mobile-menu.png
  ```

  **Commit**: YES
  - Message: `feat(web): add sidebar layout for dashboard`
  - Files: `cuttlefish-web/layouts/*`

- [ ] 4. Hero Page with Terminal Animation

  **What to do**:
  - Create `cuttlefish-site/pages/index.vue` hero page
  - Animated terminal showing Cuttlefish in action (typed commands → responses)
  - Headline: "🐙 Cuttlefish" with tagline about multi-agent coding
  - Subheadline explaining the value prop (portable, multi-model, self-developing)
  - CTA buttons: "Get Started" → /install, "View Source" → GitHub
  - Feature highlights below fold (3-4 cards)
  - Use the cuttlefish analogy: "Like cuttlefish adapting to their environment..."
  - Background: subtle grid pattern or particles (not distracting)

  **Must NOT do**:
  - Don't use heavy animations that hurt performance
  - Don't use stock photos or generic illustrations
  - Don't make the terminal demo too long (10-15 seconds max loop)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: High-impact visual design page
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 5, 6, 7)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 12, 13
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `README.md` — Philosophy and feature descriptions
  - Terminal animation inspiration: Vercel homepage, Railway homepage

  **Acceptance Criteria**:
  - [ ] Hero loads in < 2 seconds
  - [ ] Terminal animation plays smoothly
  - [ ] CTAs link correctly
  - [ ] Page scores 90+ on Lighthouse performance

  **QA Scenarios**:
  ```
  Scenario: Hero page renders correctly
    Tool: Playwright
    Preconditions: Marketing site dev server running
    Steps:
      1. Navigate to /
      2. Wait for terminal animation to start
      3. Take full-page screenshot
    Expected Result: Hero visible with animation playing
    Evidence: .sisyphus/evidence/task-4-hero.png

  Scenario: CTA buttons work
    Tool: Playwright
    Preconditions: Hero page loaded
    Steps:
      1. Click "Get Started" button
      2. Verify navigation to /install
      3. Go back, click "View Source"
      4. Verify opens GitHub in new tab
    Expected Result: Both CTAs function correctly
    Evidence: .sisyphus/evidence/task-4-cta-nav.txt
  ```

  **Commit**: YES
  - Message: `feat(site): add hero page with terminal animation`
  - Files: `cuttlefish-site/pages/index.vue`

- [ ] 5. Features Page

  **What to do**:
  - Create `cuttlefish-site/pages/features.vue`
  - Feature grid with 6-8 key features:
    - Multi-Agent System (Orchestrator → Coder → Critic)
    - Multi-Model Routing (different models for different tasks)
    - Docker Sandboxes (isolated execution)
    - Multi-Interface (Discord, WebUI, TUI)
    - Self-Developing (GitHub Actions auto-update)
    - Template Marketplace
  - Each feature: icon, title, description, optional code snippet
  - Use terminal components for code examples
  - Comparison table: Cuttlefish vs other AI coding tools
  - CTA at bottom: "Ready to dive in? Get Started"

  **Must NOT do**:
  - Don't use generic icons (create custom or use meaningful ones)
  - Don't write marketing fluff (technical accuracy)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 6, 7)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `README.md` — Feature descriptions
  - Linear, Vercel, Railway feature pages for inspiration

  **Acceptance Criteria**:
  - [ ] 6+ features displayed in grid
  - [ ] Code snippets use terminal components
  - [ ] Page is mobile responsive
  - [ ] No broken images or icons

  **QA Scenarios**:
  ```
  Scenario: Features page renders all sections
    Tool: Playwright
    Preconditions: Marketing site running
    Steps:
      1. Navigate to /features
      2. Count feature cards (expect 6+)
      3. Scroll to comparison table
      4. Take full-page screenshot
    Expected Result: All features visible, table present
    Evidence: .sisyphus/evidence/task-5-features.png
  ```

  **Commit**: YES
  - Message: `feat(site): add features page`
  - Files: `cuttlefish-site/pages/features.vue`

- [ ] 6. Installation Guide Page

  **What to do**:
  - Create `cuttlefish-site/pages/install.vue`
  - Tab-based sections: Quick Start, Docker, Manual, Production
  - Each section uses CommandLine components for copy-paste
  - Prerequisites section with version requirements
  - Configuration walkthrough with code blocks
  - Troubleshooting FAQ accordion
  - Link to full docs for advanced topics

  **Must NOT do**:
  - Don't duplicate full documentation (link instead)
  - Don't show secrets in examples (use placeholders)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5, 7)
  - **Parallel Group**: Wave 2
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `install.sh` — Current installation script
  - `README.md` Installation section

  **Acceptance Criteria**:
  - [ ] Tabs switch content correctly
  - [ ] All commands are copy-able
  - [ ] Prerequisites clearly listed
  - [ ] FAQ accordion works

  **QA Scenarios**:
  ```
  Scenario: Installation tabs work
    Tool: Playwright
    Preconditions: Marketing site running
    Steps:
      1. Navigate to /install
      2. Click "Docker" tab
      3. Verify Docker-specific content shown
      4. Click "Production" tab
      5. Verify systemd content shown
    Expected Result: Tabs switch content
    Evidence: .sisyphus/evidence/task-6-install-tabs.png
  ```

  **Commit**: YES
  - Message: `feat(site): add installation guide page`
  - Files: `cuttlefish-site/pages/install.vue`

- [ ] 7. Documentation Structure

  **What to do**:
  - Create `cuttlefish-site/content/docs/` for markdown content
  - Install `@nuxt/content` module for markdown rendering
  - Create `cuttlefish-site/pages/docs/[...slug].vue` catch-all route
  - Create sidebar navigation for docs
  - Initial docs pages:
    - `getting-started.md` — Quick overview
    - `configuration.md` — Config reference
    - `agents.md` — Agent system explanation
    - `templates.md` — Template system
    - `api.md` — REST API reference
  - Add search (optional, can use Cmd+K modal)

  **Must NOT do**:
  - Don't write exhaustive docs (just structure + key pages)
  - Don't duplicate README content verbatim

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Mostly config and structure
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5, 6)
  - **Parallel Group**: Wave 2
  - **Blocks**: None (docs are standalone)
  - **Blocked By**: Task 1

  **References**:
  - Nuxt Content: https://content.nuxt.com/
  - VitePress, Docusaurus for doc site patterns

  **Acceptance Criteria**:
  - [ ] /docs/getting-started renders markdown
  - [ ] Sidebar shows all doc pages
  - [ ] Links between docs work
  - [ ] Code blocks syntax highlighted

  **QA Scenarios**:
  ```
  Scenario: Documentation renders from markdown
    Tool: Playwright
    Preconditions: Marketing site running
    Steps:
      1. Navigate to /docs/getting-started
      2. Verify heading rendered
      3. Click sidebar link to /docs/configuration
      4. Verify page changes
    Expected Result: Markdown renders, navigation works
    Evidence: .sisyphus/evidence/task-7-docs.png
  ```

  **Commit**: YES
  - Message: `feat(site): add documentation structure`
  - Files: `cuttlefish-site/content/docs/*`, `cuttlefish-site/pages/docs/[...slug].vue`

- [ ] 8. Template Browser Page (Dashboard)

  **What to do**:
  - Create `cuttlefish-web/pages/templates.vue`
  - Fetch templates from `GET /api/templates`
  - Display grid of template cards (reuse from v1-templates Task 9)
  - Filter by language (TypeScript, Python, Rust, etc.)
  - Search by name/description
  - "Use Template" button → navigate to project creation with template pre-selected
  - "View Details" modal with full template content

  **Must NOT do**:
  - Don't allow template editing (read-only)
  - Don't fetch remote templates here (that's /api/templates/fetch)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 9, 10, 11)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 2, 3

  **References**:
  - `cuttlefish-web/pages/index.vue` — Existing grid pattern
  - v1-templates plan Task 9

  **Acceptance Criteria**:
  - [ ] Templates load from API
  - [ ] Filter by language works
  - [ ] Search works
  - [ ] "Use Template" navigates correctly

  **QA Scenarios**:
  ```
  Scenario: Template browser with filter
    Tool: Playwright
    Preconditions: Dashboard running, API has templates
    Steps:
      1. Navigate to /templates
      2. Click "Python" filter
      3. Verify only Python templates shown
    Expected Result: Filtering works
    Evidence: .sisyphus/evidence/task-8-template-filter.png
  ```

  **Commit**: YES
  - Message: `feat(web): add template browser page`
  - Files: `cuttlefish-web/pages/templates.vue`

- [ ] 9. Agent Activity Logs Page

  **What to do**:
  - Create `cuttlefish-web/pages/logs.vue`
  - Fetch recent agent activity from API (new endpoint needed or use existing)
  - Display timeline of agent actions:
    - Timestamp, agent name, action type, brief description
    - Color-coded by agent (Orchestrator=purple, Coder=yellow, Critic=red)
  - Filter by: project, agent, time range
  - Click item to expand full details (tool calls, responses)
  - Auto-refresh with WebSocket or polling

  **Must NOT do**:
  - Don't show raw LLM responses (summarize)
  - Don't load all history at once (paginate)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 8, 10, 11)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 2, 3

  **References**:
  - `cuttlefish-web/pages/project/[id].vue:25-40` — Agent message styling

  **Acceptance Criteria**:
  - [ ] Logs display in timeline format
  - [ ] Filter by project works
  - [ ] Expand/collapse details works
  - [ ] Auto-refresh works

  **QA Scenarios**:
  ```
  Scenario: Logs page with filters
    Tool: Playwright
    Preconditions: Dashboard running, some agent activity exists
    Steps:
      1. Navigate to /logs
      2. Select specific project filter
      3. Verify only that project's logs shown
      4. Click log item to expand
    Expected Result: Filtering and expansion work
    Evidence: .sisyphus/evidence/task-9-logs.png
  ```

  **Commit**: YES
  - Message: `feat(web): add agent activity logs page`
  - Files: `cuttlefish-web/pages/logs.vue`

- [ ] 10. Settings/Config Page

  **What to do**:
  - Create `cuttlefish-web/pages/settings.vue`
  - Sections:
    - API Keys: Show masked keys, copy button, regenerate
    - Model Configuration: Select models for each category
    - Sandbox Limits: CPU, memory, disk settings
    - Tunnel: Connection status, link code generation (if self-hosted)
    - About: Version, links to docs/GitHub
  - Form inputs with validation
  - Save button with feedback (success/error toast)
  - Some settings may require server restart (warn user)

  **Must NOT do**:
  - Don't expose full API keys (mask all but last 4 chars)
  - Don't auto-save (explicit save button)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 8, 9, 11)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 2, 3

  **References**:
  - `cuttlefish.example.toml` — Configuration options

  **Acceptance Criteria**:
  - [ ] All settings sections render
  - [ ] API key masking works
  - [ ] Save shows success feedback
  - [ ] Validation prevents invalid input

  **QA Scenarios**:
  ```
  Scenario: Settings save flow
    Tool: Playwright
    Preconditions: Dashboard running
    Steps:
      1. Navigate to /settings
      2. Change a model selection
      3. Click Save
      4. Verify success toast appears
    Expected Result: Settings saved successfully
    Evidence: .sisyphus/evidence/task-10-settings-save.png
  ```

  **Commit**: YES
  - Message: `feat(web): add settings page`
  - Files: `cuttlefish-web/pages/settings.vue`

- [ ] 11. Enhance Existing Project Pages

  **What to do**:
  - Update `cuttlefish-web/pages/index.vue`:
    - Add template selector to project creation form
    - Show template badge on project cards
    - Add "Archived" filter
  - Update `cuttlefish-web/pages/project/[id].vue`:
    - Add "Files" tab showing sandbox file tree
    - Add "Settings" tab for project-specific config
    - Improve diff viewer with syntax highlighting
    - Add "Archive" and "Delete" buttons

  **Must NOT do**:
  - Don't break existing functionality
  - Don't change WebSocket protocol

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 8, 9, 10)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 2, 3

  **References**:
  - Existing files: `cuttlefish-web/pages/index.vue`, `cuttlefish-web/pages/project/[id].vue`

  **Acceptance Criteria**:
  - [ ] Template selector in project creation
  - [ ] New tabs work in project detail
  - [ ] Archive/Delete buttons functional
  - [ ] No regression in existing features

  **QA Scenarios**:
  ```
  Scenario: Create project with template
    Tool: Playwright
    Preconditions: Dashboard running, templates available
    Steps:
      1. Navigate to /
      2. Fill project name
      3. Select template from dropdown
      4. Click Create
      5. Verify project created with template
    Expected Result: Template applied to new project
    Evidence: .sisyphus/evidence/task-11-project-template.png
  ```

  **Commit**: YES
  - Message: `feat(web): enhance project pages with templates and new tabs`
  - Files: `cuttlefish-web/pages/index.vue`, `cuttlefish-web/pages/project/[id].vue`

- [ ] 12. Marketplace Page (Marketing Site)

  **What to do**:
  - Create `cuttlefish-site/pages/marketplace.vue`
  - Show community templates from GitHub
  - Categories: Official, Community, New
  - Each card: name, description, author, stars, downloads
  - "Use Template" → deep link to app with template pre-selected
  - "Submit Your Template" → link to contribution guide
  - Search and filter functionality

  **Must NOT do**:
  - Don't fetch from API (SSG, use GitHub directly or pre-build)
  - Don't require authentication to browse

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 13, 14)
  - **Parallel Group**: Wave 4
  - **Blocks**: Task 13
  - **Blocked By**: Tasks 1, 2, 4

  **References**:
  - npm, crates.io, VS Code marketplace for inspiration

  **Acceptance Criteria**:
  - [ ] Templates display in categories
  - [ ] Search works
  - [ ] "Use Template" links to app
  - [ ] "Submit" link works

  **QA Scenarios**:
  ```
  Scenario: Marketplace search
    Tool: Playwright
    Preconditions: Marketing site running
    Steps:
      1. Navigate to /marketplace
      2. Type "nuxt" in search
      3. Verify filtered results
    Expected Result: Only Nuxt-related templates shown
    Evidence: .sisyphus/evidence/task-12-marketplace-search.png
  ```

  **Commit**: YES
  - Message: `feat(site): add marketplace page`
  - Files: `cuttlefish-site/pages/marketplace.vue`

- [ ] 13. Mobile Responsive Pass

  **What to do**:
  - Test all pages at viewports: 375px, 768px, 1024px, 1440px
  - Fix layout issues:
    - Navigation collapses to hamburger on mobile
    - Tables become scrollable or stacked cards
    - Forms stack vertically
    - Images scale appropriately
  - Test touch interactions (no hover-only features)
  - Test on real device if possible

  **Must NOT do**:
  - Don't hide critical content on mobile
  - Don't use tiny tap targets (<44px)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 12, 14)
  - **Parallel Group**: Wave 4
  - **Blocks**: Task 14
  - **Blocked By**: Tasks 4-12

  **References**:
  - Tailwind responsive prefixes: sm:, md:, lg:, xl:

  **Acceptance Criteria**:
  - [ ] All pages usable at 375px width
  - [ ] No horizontal scroll on mobile
  - [ ] Touch targets >= 44px
  - [ ] Forms work on mobile

  **QA Scenarios**:
  ```
  Scenario: Mobile navigation
    Tool: Playwright
    Preconditions: Both sites running
    Steps:
      1. Set viewport to 375x667
      2. Navigate to each page
      3. Verify no horizontal overflow
      4. Take screenshots
    Expected Result: All pages mobile-friendly
    Evidence: .sisyphus/evidence/task-13-mobile-screenshots/
  ```

  **Commit**: YES
  - Message: `style(web): mobile responsive improvements`
  - Files: Various Vue files with responsive fixes

- [ ] 14. Accessibility Audit + Fixes

  **What to do**:
  - Run Lighthouse accessibility audit on all pages
  - Fix issues:
    - Add alt text to images
    - Ensure color contrast ratios
    - Add ARIA labels to interactive elements
    - Ensure keyboard navigation works
    - Add skip-to-content link
  - Test with screen reader (VoiceOver/NVDA)
  - Add `prefers-reduced-motion` support for animations

  **Must NOT do**:
  - Don't sacrifice design for accessibility (both possible)
  - Don't use ARIA where native HTML works

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Accessibility requires careful attention
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (after Task 13)
  - **Parallel Group**: Wave 4
  - **Blocks**: Task 15
  - **Blocked By**: Task 13

  **References**:
  - WCAG 2.1 guidelines
  - axe-core for automated testing

  **Acceptance Criteria**:
  - [ ] Lighthouse accessibility score >= 90 on all pages
  - [ ] Keyboard navigation works throughout
  - [ ] No critical WCAG violations
  - [ ] Reduced motion respected

  **QA Scenarios**:
  ```
  Scenario: Lighthouse accessibility audit
    Tool: Bash (lighthouse CLI)
    Preconditions: Sites deployed or running locally
    Steps:
      1. Run lighthouse on each page with --only-categories=accessibility
      2. Verify score >= 90
    Expected Result: All pages pass
    Evidence: .sisyphus/evidence/task-14-lighthouse-reports/
  ```

  **Commit**: YES
  - Message: `fix(web): accessibility improvements`
  - Files: Various Vue files with a11y fixes

- [ ] 15. Deployment Configuration

  **What to do**:
  - Create `cuttlefish-site/vercel.json` or `netlify.toml` for marketing site
  - Create `deploy/docker-compose.yml` for dashboard + API
  - Create GitHub Actions workflow `.github/workflows/deploy-site.yml`:
    - Build marketing site on push to main
    - Deploy to Cloudflare Pages / Vercel / Netlify
  - Create GitHub Actions workflow `.github/workflows/deploy-app.yml`:
    - Build dashboard and API Docker images
    - Push to GitHub Container Registry
  - Document deployment in `docs/deployment/`

  **Must NOT do**:
  - Don't commit secrets (use GitHub Secrets)
  - Don't auto-deploy without CI passing

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Config files, mostly YAML
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 5 (after all UI done)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 14

  **References**:
  - Existing `.github/workflows/` if any
  - Vercel/Netlify/Cloudflare docs

  **Acceptance Criteria**:
  - [ ] Marketing site deploys from GitHub Actions
  - [ ] Docker compose starts full stack
  - [ ] No secrets in committed files
  - [ ] Documentation complete

  **QA Scenarios**:
  ```
  Scenario: Docker compose starts stack
    Tool: Bash
    Preconditions: Docker installed
    Steps:
      1. Run `docker compose -f deploy/docker-compose.yml up -d`
      2. Wait 30 seconds
      3. curl http://localhost:8080/health
      4. curl http://localhost:3000
    Expected Result: Both API and dashboard respond
    Evidence: .sisyphus/evidence/task-15-docker-compose.txt
  ```

  **Commit**: YES
  - Message: `chore: add deployment configuration`
  - Files: `deploy/*`, `.github/workflows/*`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Visual QA with Playwright** — `visual-engineering`
  Run Playwright visual regression tests on all pages. Take screenshots at 1920x1080, 1366x768, 375x667 viewports. Compare against design intent. Flag visual issues.
  Output: `Pages [N/N pass] | Viewports [N/N] | Visual Issues [N] | VERDICT`

- [ ] F3. **API Integration QA** — `unspecified-high`
  Test all API integrations in dashboard. Create project, view templates, check logs. Verify WebSocket connection. Test with mock server and real server.
  Output: `API Calls [N/N pass] | WebSocket [PASS/FAIL] | Integration [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1 match. Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(site): initialize marketing site with Nuxt 3` |
| 1 | `feat(web): add terminal-style component library` |
| 1 | `feat(web): add sidebar layout for dashboard` |
| 2 | `feat(site): add hero page with terminal animation` |
| 2 | `feat(site): add features page` |
| 2 | `feat(site): add installation guide page` |
| 2 | `feat(site): add documentation structure` |
| 3 | `feat(web): add template browser page` |
| 3 | `feat(web): add agent logs page` |
| 3 | `feat(web): add settings page` |
| 4 | `feat(site): add marketplace page` |
| 4 | `style(web): mobile responsive improvements` |
| 5 | `chore: add deployment configuration` |

---

## Success Criteria

### Verification Commands
```bash
cd cuttlefish-site && npm run generate  # Static site builds
cd cuttlefish-web && npm run build      # App builds
npx playwright test                      # E2E tests pass
npx lighthouse https://localhost:3000 --only-categories=accessibility  # Score >= 90
```

### Final Checklist
- [ ] Marketing site has: Hero, Features, Install, Docs, Marketplace
- [ ] App dashboard has: Projects, Templates, Logs, Settings
- [ ] Terminal aesthetic consistent across all pages
- [ ] Mobile responsive at 375px width
- [ ] No console errors in production
- [ ] All links work (no 404s)
