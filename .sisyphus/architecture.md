# Cuttlefish Platform Architecture

> **This document captures the full product vision. Individual work plans reference this for context.**

---

## Product Overview

**Cuttlefish** is a portable, multi-agent, multi-model AI coding platform built in Rust. It operates in two modes:

1. **Self-Hosted**: Users run their own Cuttlefish instance on their infrastructure
2. **SaaS (Hosted)**: Users get a managed instance at `{username}.cuttlefish.ai`

Both modes can be connected via a **tunnel system** that allows self-hosted instances to be accessible through the cuttlefish.ai domain.

---

## Architecture Diagrams

### High-Level System Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           cuttlefish.ai (Public)                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────┐     ┌─────────────────────┐                       │
│  │   Marketing Site    │     │   Auth Service      │                       │
│  │   (cuttlefish.ai)   │     │   (Neon Database)   │                       │
│  │   - Hero            │     │   - User accounts   │                       │
│  │   - Features        │     │   - API keys        │                       │
│  │   - Docs            │     │   - Subscriptions   │                       │
│  │   - Marketplace     │     │   - Link codes      │                       │
│  └─────────────────────┘     └─────────────────────┘                       │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    Tunnel Proxy (Caddy/nginx)                        │   │
│  │   Routes *.cuttlefish.ai to appropriate backend                     │   │
│  │   - jacktym.cuttlefish.ai → Self-hosted (via tunnel)                │   │
│  │   - alice.cuttlefish.ai → SaaS VM (direct)                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                        │                                    │
│              ┌─────────────────────────┼─────────────────────────┐         │
│              │                         │                         │         │
│              ▼                         ▼                         ▼         │
│  ┌─────────────────────┐   ┌─────────────────────┐   ┌─────────────────┐  │
│  │  SaaS VMs (KVM)     │   │  Tunnel Daemon      │   │  Template       │  │
│  │  Proxmox-managed    │   │  (WireGuard/WebSocket)│  │  Registry       │  │
│  │  Per-user isolation │   │  Connects self-host │   │  (GitHub-based) │  │
│  └─────────────────────┘   └─────────────────────┘   └─────────────────┘  │
│                                        │                                    │
└────────────────────────────────────────│────────────────────────────────────┘
                                         │
                          ┌──────────────┴──────────────┐
                          │     Self-Hosted Instances    │
                          │     (User's Infrastructure)  │
                          │                              │
                          │  ┌────────────────────────┐  │
                          │  │ Cuttlefish Instance    │  │
                          │  │ - Tunnel client        │  │
                          │  │ - Full functionality   │  │
                          │  │ - Local dashboard      │  │
                          │  └────────────────────────┘  │
                          └─────────────────────────────┘
```

### SaaS Infrastructure (Future - Not V1)

```
Dedicated Server (Hetzner/OVH) - $100-200/month
├── Host OS (Debian 12)
├── Proxmox VE (KVM Hypervisor)
│   ├── orchestrator-vm (Management plane)
│   │   ├── Cuttlefish Orchestrator (Rust service)
│   │   ├── Caddy reverse proxy
│   │   ├── WireGuard server
│   │   └── Neon DB connection
│   │
│   ├── user-alice-vm (Tier 1: $15/mo)
│   │   ├── 2 vCPU, 4GB RAM, 50GB disk
│   │   ├── Cuttlefish instance
│   │   ├── Docker daemon
│   │   └── User's project sandboxes
│   │
│   ├── user-bob-vm (Tier 2: $25/mo)
│   │   ├── 4 vCPU, 8GB RAM, 100GB disk
│   │   └── ...
│   │
│   └── user-charlie-vm (Tier 3: $50/mo)
│       ├── 8 vCPU, 16GB RAM, 200GB disk
│       └── ...
│
└── Shared Storage (optional NFS for templates/cache)
```

---

## Component Breakdown

### 1. Marketing Site (`cuttlefish.ai`)

**Purpose**: Public-facing website for marketing, documentation, and template marketplace.

**Tech Stack**: Nuxt 3 (SSG), Tailwind CSS, deployed to Cloudflare Pages

**Pages**:
- `/` - Hero with terminal animation
- `/features` - Feature grid
- `/install` - Self-host installation guide
- `/docs/*` - Documentation (markdown-driven)
- `/marketplace` - Template browser
- `/pricing` - SaaS tiers (future)
- `/login` - Auth redirect to dashboard

**No authentication required** for marketing pages.

---

### 2. Dashboard App (`app.cuttlefish.ai` or `{user}.cuttlefish.ai`)

**Purpose**: Authenticated dashboard for managing Cuttlefish instances.

**Tech Stack**: Nuxt 3 (SPA), Tailwind CSS, WebSocket for real-time

**Features**:
- Project management (list, create, delete)
- Chat interface with agents
- Build logs viewer
- Diff viewer
- Template browser (integrated)
- Agent activity logs
- Settings (API keys, model config)
- Usage metrics (CPU, RAM, tokens, costs)

**Authentication**: 
- Neon Auth (OAuth + email/password)
- Session-based for web, API key for CLI/integrations

**Subdomain Routing**:
- `app.cuttlefish.ai` - Generic login, redirects to user subdomain
- `{username}.cuttlefish.ai` - User's personal dashboard
- Routes to either SaaS VM or tunnel-connected self-host

---

### 3. Tunnel System (Self-Host ↔ cuttlefish.ai)

**Purpose**: Allow self-hosted Cuttlefish instances to be accessible via `{user}.cuttlefish.ai`.

**How It Works**:

```
┌─────────────────────┐         ┌─────────────────────┐
│  Self-Hosted        │         │  cuttlefish.ai      │
│  Cuttlefish         │         │  Tunnel Daemon      │
│                     │         │                     │
│  ┌───────────────┐  │   WSS   │  ┌───────────────┐  │
│  │ Tunnel Client │◀─┼─────────┼─▶│ Tunnel Server │  │
│  │ (in binary)   │  │         │  │ (Rust daemon) │  │
│  └───────────────┘  │         │  └───────────────┘  │
│         │           │         │         │           │
│         ▼           │         │         ▼           │
│  ┌───────────────┐  │         │  ┌───────────────┐  │
│  │ Local Web UI  │  │         │  │ Reverse Proxy │  │
│  │ :8080         │  │         │  │ (Caddy)       │  │
│  └───────────────┘  │         │  └───────────────┘  │
│                     │         │         │           │
└─────────────────────┘         │         ▼           │
                                │  jacktym.cuttlefish.ai
                                └─────────────────────┘
```

**Connection Flow**:
1. User runs `cuttlefish tunnel connect` on self-hosted instance
2. CLI prompts for link code (generated at cuttlefish.ai/settings/tunnel)
3. Client establishes WebSocket connection to tunnel daemon
4. Daemon registers the connection with user's subdomain
5. Requests to `{user}.cuttlefish.ai` are proxied through the tunnel
6. Responses flow back through the same tunnel

**Security**:
- Link codes are one-time use, expire in 10 minutes
- Tunnel authenticated via JWT after initial handshake
- All traffic encrypted (WSS)
- Rate limiting per tunnel

---

### 4. Auth System (Neon Database)

**Purpose**: Centralized authentication for cuttlefish.ai.

**Tables**:
```sql
-- Users
CREATE TABLE users (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT UNIQUE NOT NULL,
  username TEXT UNIQUE NOT NULL,  -- Used for subdomain
  password_hash TEXT,             -- NULL if OAuth-only
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- OAuth connections
CREATE TABLE oauth_connections (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id),
  provider TEXT NOT NULL,  -- 'github', 'google'
  provider_user_id TEXT NOT NULL,
  UNIQUE(provider, provider_user_id)
);

-- API keys (for CLI, integrations)
CREATE TABLE api_keys (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id),
  key_hash TEXT NOT NULL,
  name TEXT,
  last_used_at TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tunnel link codes
CREATE TABLE tunnel_link_codes (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id),
  code TEXT UNIQUE NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  used_at TIMESTAMPTZ
);

-- Active tunnels
CREATE TABLE active_tunnels (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID REFERENCES users(id),
  connected_at TIMESTAMPTZ DEFAULT NOW(),
  last_heartbeat TIMESTAMPTZ,
  client_version TEXT,
  client_ip INET
);
```

---

### 5. Template System

**Purpose**: Project scaffolding from reusable templates.

**Components**:
- Template manifest parser (`cuttlefish.toml`)
- Variable substitution engine (Tera)
- Local template loading (`templates/` directory)
- Remote fetching (GitHub repos)
- Template registry (curated list + community)

**Marketplace Model**:
- Official templates in `github.com/cuttlefish-templates/`
- Community templates tagged with `cuttlefish-template` topic
- WebUI browser at `/marketplace` and `/templates` (in dashboard)

---

## V1 Scope vs Future

### V1 (Current Focus)

| Component | Status | Notes |
|-----------|--------|-------|
| Agent System (OmO-style) | 🔴 Planned | `v1-agents.md` |
| Template System | 🔴 Planned | `v1-templates.md` |
| Marketing Site | 🔴 Planned | `v1-webui.md` |
| Dashboard (basic) | 🟡 Exists | Needs enhancement |
| Tunnel System | 🔴 Planned | New plan needed |
| Self-host Install | 🟢 Done | `install.sh` exists |

### Post-V1 (Not in Current Plans)

| Component | Priority | Notes |
|-----------|----------|-------|
| SaaS Infrastructure | Later | Proxmox + KVM automation |
| Billing/Subscriptions | Later | Stripe integration |
| Multi-interface (Telegram, Slack) | Later | After core stable |
| WASM Plugin System | Later | Marketplace for plugins |
| Collaboration Features | Later | Multi-user projects |

---

## Pricing Tiers (Future Reference)

| Tier | Price | Resources | Features |
|------|-------|-----------|----------|
| **Free** | $0 | Self-host only | Full features, no SaaS |
| **Starter** | $15/mo | 2 vCPU, 4GB, 50GB | SaaS hosting, tunnel |
| **Pro** | $25/mo | 4 vCPU, 8GB, 100GB | Priority support |
| **Team** | $50/mo | 8 vCPU, 16GB, 200GB | Multi-user, org features |

**BYOK Model**: Users provide their own API keys (AWS Bedrock, Anthropic). Cuttlefish tracks usage for display but doesn't bill for AI calls.

---

## File Locations

| Component | Location |
|-----------|----------|
| Architecture doc | `.sisyphus/architecture.md` (this file) |
| Agent prompts plan | `.sisyphus/plans/v1-agents.md` |
| Template system plan | `.sisyphus/plans/v1-templates.md` |
| WebUI plan | `.sisyphus/plans/v1-webui.md` |
| Tunnel plan | `.sisyphus/plans/v1-tunnel.md` (to be created) |
| Marketing site | `cuttlefish-site/` (to be created) |
| Dashboard app | `cuttlefish-web/` (exists) |
| Tunnel daemon | `crates/cuttlefish-tunnel/` (to be created) |

---

## Key Decisions Made

1. **Auth**: Neon Database for centralized auth
2. **Hosting Model**: Hybrid (self-host + SaaS)
3. **Billing**: BYOK for V1 (users bring API keys)
4. **Tunnel Protocol**: WebSocket-based (simpler than WireGuard for V1)
5. **Template Engine**: Tera (Rust-native, Jinja2-like)
6. **Marketing Site**: Separate Nuxt app at root domain
7. **Dashboard**: Enhanced existing `cuttlefish-web/` at `app.` subdomain
8. **Visual Style**: Hacker/dev-tool aesthetic (terminal-inspired)
