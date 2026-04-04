# V1 Master Planning Tracker

## Scope Decisions (Confirmed with User)

### V1 Includes
- **Discord Bot**: Slash commands, smart notifications, project channels, agent embeds
- **WebUI Dashboard**: System resources, project chat, agent feed, file browser, workflow viz, test trends
- **Marketing Site**: cuttle.sh landing page
- **TUI Client**: Full terminal UI (Claude Code-like experience)
- **Mobile**: PWA companion app
- **Docker Sandbox**: Language images, volume mounting, resource limits, snapshot/restore
- **Tunnel System**: WebSocket tunnel, username.cuttle.sh routing, binary proxy, dev server preview
- **Agent Memory**: Memory file, "why" command, project state branching
- **Safety**: Confidence gates, diff previews, checkpoint rollback
- **Collaboration**: Multi-user, roles, async handoffs, org configs
- **Templates**: Marketplace, ratings, CI verification
- **Git Integration**: GitHub/GitLab PRs, issues, CI status
- **Cost Tracking**: BYOK visibility, per-project/session costs
- **Auth**: Built-in authentication (for multi-user support)
- **Release**: v0.1.0 tag, binaries, process

### V2/Deferred
- Telegram/Slack/WhatsApp bots (Discord only for V1)
- Proxmox/KVM infrastructure (SaaS deployment)
- Multi-node scaling (SaaS deployment)
- Plugin system (spec says defer)

---

## Plan Status

| # | Plan | Status | Notes |
|---|------|--------|-------|
| 1 | `v1-discord.md` | ✅ DONE | 13 tasks, full bot implementation |
| 2 | `v1-webui.md` | ⏳ TODO | Update existing |
| 3 | `v1-marketing.md` | ⏳ TODO | |
| 4 | `v1-tui.md` | ⏳ TODO | |
| 5 | `v1-mobile.md` | ⏳ TODO | |
| 6 | `v1-sandbox.md` | ✅ DONE | 12 tasks, Docker lifecycle + snapshots |
| 7 | `v1-tunnel.md` | ⏳ TODO | Update existing |
| 8 | `v1-memory.md` | ⏳ TODO | |
| 9 | `v1-safety.md` | ⏳ TODO | |
| 10 | `v1-collaboration.md` | ⏳ TODO | |
| 11 | `v1-templates.md` | ⏳ TODO | Update existing |
| 12 | `v1-git-integration.md` | ⏳ TODO | |
| 13 | `v1-costs.md` | ⏳ TODO | |
| 14 | `v1-agents.md` | ⏳ TODO | Update existing |
| 15 | `v1-auth.md` | ⏳ TODO | |
| 16 | `v1-release.md` | ⏳ TODO | |

---

## Key Technical Decisions

### Authentication
- Built-in auth system (not OAuth, not self-hosted only)
- Required for: multi-user mode, role separation, org configs

### Mobile Approach
- PWA (Progressive Web App) — not native iOS/Android

### Infrastructure
- Docker containers for V1
- Proxmox/KVM deferred to SaaS phase

### Client Priority
- Discord is primary bot for V1
- All clients share same session state

---

## Cross-Cutting Concerns

### Session State
All clients (WebUI, TUI, Discord, Mobile) must:
- Share the same session state
- Be fully serializable and resumable
- Support agent continuing in background

### BYOK Model
- Users configure their own API keys
- Keys never proxied through Cuttlefish servers
- Cost tracking per project/session required

### Config as First-Class
- Docker templates, framework choices, agent hints
- Versioned, exportable, shareable, importable
- Lives in template registry

### Agent Memory
- Memory files live inside project
- Committed to version control
- Travel with project if exported
