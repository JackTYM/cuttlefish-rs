# Install.sh V2 — Comprehensive Cross-Platform Installer

## TL;DR

> **Quick Summary**: Rewrite `install.sh` to support deployment mode selection (systemd vs Docker), cross-platform installation (Linux/macOS/WSL), AWS Bedrock regional model variants with full model ID display, and custom model/endpoint configuration for all providers.
> 
> **Deliverables**:
> - Rewritten `install.sh` (~1500+ lines) with all new features
> - Deployment mode selector with Docker sandbox isolation warning
> - Cross-platform support (Linux distros, macOS with Homebrew/Colima, WSL)
> - Bedrock model selector showing all regional variants (default, us., eu., global.)
> - Custom model string and API endpoint configuration
> - Updated model IDs throughout codebase (Claude 4.x family)
> 
> **Estimated Effort**: Medium (3-5 tasks)
> **Parallel Execution**: YES — 2 waves
> **Critical Path**: Platform Detection → Deployment Mode → Provider Config → Model Selection

---

## Context

### Original Request
User requested comprehensive rewrite of `install.sh` to:
1. Support systemd (recommended) and Docker deployment modes
2. Warn about no sandbox isolation in Docker mode
3. Cross-platform support for Linux, macOS, and WSL
4. Show all Bedrock regional model variants (default, US, EU, global)
5. Support custom model strings and API endpoints
6. Update all outdated model references to Claude 4.x

### Current State Analysis
The existing `install.sh` (1128 lines) has:
- ✅ Provider selection menu (11 providers)
- ✅ API key collection
- ✅ Systemd service creation
- ✅ Basic dependency checking
- ❌ No deployment mode selection
- ❌ Linux-only (no macOS/WSL support)
- ❌ Outdated Bedrock model IDs (still using claude-3-5-sonnet)
- ❌ No regional model variant selection
- ❌ No custom model/endpoint support

### Research Findings
**AWS Bedrock Regional Model IDs** (from AWS docs):
```
# Claude Sonnet 4.6
anthropic.claude-sonnet-4-6                    # Default (single region)
us.anthropic.claude-sonnet-4-6                 # US cross-region inference
eu.anthropic.claude-sonnet-4-6                 # EU cross-region inference  
global.anthropic.claude-sonnet-4-6             # Global cross-region inference

# Claude Opus 4.6
anthropic.claude-opus-4-6-v1                   # Default
us.anthropic.claude-opus-4-6-v1                # US
eu.anthropic.claude-opus-4-6-v1                # EU
global.anthropic.claude-opus-4-6-v1            # Global

# Claude Haiku 4.5
anthropic.claude-haiku-4-5-20251001-v1:0       # Default
us.anthropic.claude-haiku-4-5-20251001-v1:0    # US
global.anthropic.claude-haiku-4-5-20251001-v1:0 # Global

# Claude Opus 4.5
anthropic.claude-opus-4-5-20251101-v1:0        # Default
us.anthropic.claude-opus-4-5-20251101-v1:0     # US
global.anthropic.claude-opus-4-5-20251101-v1:0 # Global

# Claude Sonnet 4.5
anthropic.claude-sonnet-4-5-20250929-v1:0      # Default
global.anthropic.claude-sonnet-4-5-20250929-v1:0 # Global
```

**Anthropic Direct API Model IDs**:
```
claude-sonnet-4-6-20250514
claude-opus-4-6-20250514  
claude-haiku-4-5-20250414
```

---

## Work Objectives

### Core Objective
Create a production-ready installer that works across platforms, offers deployment flexibility, and provides comprehensive model configuration options.

### Concrete Deliverables
- `install.sh` — Complete rewrite with all new features
- Updated `crates/cuttlefish-providers/src/bedrock.rs` — Correct model IDs
- Updated `cuttlefish.example.toml` — Correct model references
- Updated `docs/providers/bedrock.md` — Document all regional variants

### Definition of Done
- [ ] `install.sh` runs successfully on Ubuntu 22.04
- [ ] `install.sh` runs successfully on macOS (with Homebrew)
- [ ] `install.sh` detects and handles WSL correctly
- [ ] Deployment mode selection works (systemd vs Docker)
- [ ] Docker mode shows sandbox isolation warning
- [ ] Bedrock model selector shows all 4 regional variants
- [ ] Custom model string input works
- [ ] All model IDs in codebase are Claude 4.x

### Must Have
- Deployment mode selection at start of installation
- Clear warning about Docker mode lacking sandbox isolation
- Platform detection (Linux distro, macOS, WSL)
- Appropriate package manager usage per platform
- Bedrock regional variant selection menu
- Custom model string input option
- Custom API endpoint input for each provider

### Must NOT Have (Guardrails)
- No breaking changes to existing config format
- No removal of currently supported providers
- No hardcoded credentials
- No silent failures — all errors must be reported clearly

---

## Verification Strategy

### Test Decision
- **Automated tests**: Shell script testing via shellcheck + manual verification
- **QA**: Test on Ubuntu, macOS VM, WSL

### QA Policy
Each task includes verification scenarios to run manually.

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — can run in parallel):
├── Task 1: Platform Detection + Deployment Mode Selection [quick]
├── Task 2: Cross-Platform Dependency Installation [unspecified-high]
└── Task 3: Update Model IDs Across Codebase [quick]

Wave 2 (Provider Configuration — after Wave 1):
├── Task 4: Bedrock Regional Model Selection Menu [unspecified-high]
└── Task 5: Custom Model/Endpoint Configuration [quick]

Wave FINAL (Integration):
└── Task 6: Integration Testing + Documentation [unspecified-high]
```

### Dependency Matrix

| Task | Depends On | Blocks |
|------|-----------|--------|
| 1 | — | 4, 5, 6 |
| 2 | — | 6 |
| 3 | — | 4, 6 |
| 4 | 1, 3 | 6 |
| 5 | 1 | 6 |
| 6 | 1-5 | — |

---

## TODOs

### Wave 1 — Foundation

- [x] 1. Platform Detection + Deployment Mode Selection

  **What to do**:
  - Add platform detection at script start:
    ```bash
    detect_platform() {
        if [[ "$OSTYPE" == "darwin"* ]]; then
            PLATFORM="macos"
        elif grep -qi microsoft /proc/version 2>/dev/null; then
            PLATFORM="wsl"
        elif [[ -f /etc/os-release ]]; then
            PLATFORM="linux"
            source /etc/os-release
            DISTRO="$ID"
        else
            error "Unsupported platform"
            exit 1
        fi
    }
    ```
  - Add deployment mode selection menu BEFORE dependency checks:
    ```
    === Deployment Mode ===
    
    1) Systemd Service (Recommended)
       - Cuttlefish runs as a system service
       - Uses Docker for project sandbox isolation
       - Automatic restart on failure
       - Requires: Linux with systemd, Docker
    
    2) Docker Container Mode
       ⚠️  WARNING: NO SANDBOX ISOLATION
       - Cuttlefish runs inside a Docker container
       - Projects execute in the SAME container (no isolation)
       - Suitable for: single-user, testing, or when systemd unavailable
       - Requires: Docker only
    
    Select deployment mode [1-2]:
    ```
  - Store selection in `DEPLOYMENT_MODE` variable ("systemd" or "docker")
  - For Docker mode, display prominent warning:
    ```
    ╔═══════════════════════════════════════════════════════════════╗
    ║  ⚠️  SECURITY WARNING: Docker Mode Selected                   ║
    ╠═══════════════════════════════════════════════════════════════╣
    ║  In Docker mode, all projects run in the SAME container.      ║
    ║  There is NO isolation between projects.                      ║
    ║                                                               ║
    ║  A malicious or buggy project could:                          ║
    ║  • Access files from other projects                           ║
    ║  • Interfere with other running builds                        ║
    ║  • Consume all container resources                            ║
    ║                                                               ║
    ║  Only use Docker mode for:                                    ║
    ║  • Single-user development                                    ║
    ║  • Testing purposes                                           ║
    ║  • Environments where systemd is unavailable                  ║
    ╚═══════════════════════════════════════════════════════════════╝
    
    Do you understand and accept these limitations? [y/N]:
    ```
  - Adjust subsequent installation steps based on mode

  **Must NOT do**:
  - Do NOT skip the warning for Docker mode
  - Do NOT make Docker mode the default

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 2, 3)
  - **Blocks**: Tasks 4, 5, 6
  - **Blocked By**: None

  **References**:
  - Current `install.sh` lines 1-75 — existing structure to extend
  - Current `install.sh` lines 1090-1125 — main() flow to modify

  **Acceptance Criteria**:
  - [ ] Platform detected correctly on Linux, macOS, WSL
  - [ ] Deployment mode menu displays with both options
  - [ ] Docker mode warning is prominent and requires confirmation
  - [ ] `DEPLOYMENT_MODE` variable set correctly

  **QA Scenarios**:
  ```
  Scenario: Deployment mode selection on Linux
    Tool: Bash
    Steps:
      1. Run install.sh on Ubuntu
      2. Observe deployment mode menu appears
      3. Select option 1 (systemd)
      4. Assert DEPLOYMENT_MODE="systemd"
      5. Assert no Docker warning shown
    Expected Result: Systemd mode selected without warning
    Evidence: .sisyphus/evidence/install-v2-task1-systemd.txt

  Scenario: Docker mode shows warning
    Tool: Bash
    Steps:
      1. Run install.sh
      2. Select option 2 (Docker)
      3. Assert warning box is displayed
      4. Type 'n' at confirmation
      5. Assert returns to mode selection
      6. Select Docker again, type 'y'
      7. Assert DEPLOYMENT_MODE="docker"
    Expected Result: Warning shown, confirmation required
    Evidence: .sisyphus/evidence/install-v2-task1-docker-warning.txt
  ```

  **Commit**: NO (part of larger install.sh rewrite)

---

- [x] 2. Cross-Platform Dependency Installation

  **What to do**:
  - Rewrite `install_dependencies()` to handle all platforms:
  
  **Linux (Debian/Ubuntu)**:
  ```bash
  apt-get update
  apt-get install -y curl git build-essential pkg-config libssl-dev
  # Docker via official repo
  curl -fsSL https://get.docker.com | sh
  systemctl enable docker && systemctl start docker
  ```
  
  **Linux (Fedora/RHEL)**:
  ```bash
  dnf install -y curl git gcc make openssl-devel
  dnf install -y docker
  systemctl enable docker && systemctl start docker
  ```
  
  **Linux (Arch)**:
  ```bash
  pacman -Sy --noconfirm curl git base-devel openssl
  pacman -S --noconfirm docker
  systemctl enable docker && systemctl start docker
  ```
  
  **macOS**:
  ```bash
  # Check for Homebrew
  if ! command -v brew &>/dev/null; then
      /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
  fi
  brew install curl git openssl
  
  # Docker Desktop or Colima
  if ! command -v docker &>/dev/null; then
      echo "Docker not found. Options:"
      echo "  1) Docker Desktop (GUI, requires license for commercial use)"
      echo "  2) Colima (CLI, free and open source)"
      read -p "Select [1-2]: " docker_choice
      if [[ "$docker_choice" == "1" ]]; then
          brew install --cask docker
          echo "Please launch Docker Desktop from Applications"
      else
          brew install colima docker
          colima start
      fi
  fi
  ```
  
  **WSL**:
  ```bash
  # WSL uses Linux package managers but Docker may need special handling
  apt-get update && apt-get install -y curl git build-essential
  
  # Check if Docker Desktop is providing docker
  if ! command -v docker &>/dev/null; then
      echo "Docker not found in WSL."
      echo "Options:"
      echo "  1) Use Docker Desktop for Windows (recommended)"
      echo "     - Install Docker Desktop on Windows"  
      echo "     - Enable WSL integration in Docker Desktop settings"
      echo "  2) Install Docker directly in WSL"
      read -p "Select [1-2]: " wsl_docker
      if [[ "$wsl_docker" == "2" ]]; then
          curl -fsSL https://get.docker.com | sh
          # Note: systemd may not be available in all WSL configs
          if command -v systemctl &>/dev/null; then
              systemctl enable docker && systemctl start docker
          else
              service docker start
          fi
      fi
  fi
  ```

  - Add Rust installation for all platforms (rustup works everywhere)
  - Handle case where systemd is not available (older WSL, some containers)

  **Must NOT do**:
  - Do NOT assume systemd is always available
  - Do NOT install Docker Desktop automatically on macOS without user choice

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 3)
  - **Blocks**: Task 6
  - **Blocked By**: None

  **References**:
  - Current `install.sh` lines 515-578 — existing `install_dependencies()` and `install_rust()`
  - Docker official install: https://get.docker.com
  - Colima for macOS: https://github.com/abiosoft/colima

  **Acceptance Criteria**:
  - [ ] Dependencies install correctly on Ubuntu 22.04
  - [ ] Dependencies install correctly on macOS with Homebrew
  - [ ] WSL detected and handled appropriately
  - [ ] Docker options presented on macOS (Desktop vs Colima)
  - [ ] Rust installed via rustup on all platforms

  **QA Scenarios**:
  ```
  Scenario: Ubuntu dependency installation
    Tool: Bash (Ubuntu VM/container)
    Steps:
      1. Fresh Ubuntu 22.04 with no dependencies
      2. Run install.sh
      3. Select systemd mode
      4. Assert apt packages installed
      5. Assert Docker running: docker info
      6. Assert Rust installed: rustc --version
    Expected Result: All dependencies installed and working
    Evidence: .sisyphus/evidence/install-v2-task2-ubuntu.txt

  Scenario: macOS dependency installation
    Tool: Bash (macOS)
    Steps:
      1. Run install.sh
      2. Select Docker mode (systemd not available)
      3. When prompted for Docker, select Colima
      4. Assert Homebrew used for packages
      5. Assert Colima started: colima status
    Expected Result: macOS-appropriate tools installed
    Evidence: .sisyphus/evidence/install-v2-task2-macos.txt
  ```

  **Commit**: NO (part of larger install.sh rewrite)

---

- [x] 3. Update Model IDs Across Codebase

  **What to do**:
  - Update `crates/cuttlefish-providers/src/bedrock.rs`:
    - Change all model ID constants to Claude 4.x family
    - Add regional variant support
  - Update `install.sh` provider definitions (line 35-47):
    ```bash
    PROVIDERS_DATA="
    1:anthropic:Anthropic:api:sk-ant-:claude-opus-4-6, claude-sonnet-4-6, claude-haiku-4-5
    2:openai:OpenAI:api:sk-:gpt-5.4, gpt-5-nano, gpt-4o
    ...
    "
    ```
  - Update `install.sh` Bedrock config section (lines 860-875):
    - Replace `anthropic.claude-3-5-sonnet-20241022-v2:0` with `anthropic.claude-sonnet-4-6`
    - Replace `anthropic.claude-3-haiku-20240307-v1:0` with `anthropic.claude-haiku-4-5-20251001-v1:0`
  - Update `cuttlefish.example.toml`:
    - Ensure all model references are Claude 4.x
  - Update `docs/providers/bedrock.md` with correct model IDs

  **Must NOT do**:
  - Do NOT remove support for older models (user may still specify them)
  - Do NOT change model IDs without updating all references

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 1, 2)
  - **Blocks**: Task 4, 6
  - **Blocked By**: None

  **References**:
  - AWS Bedrock docs: Model IDs for Claude family
  - Current `install.sh` lines 35-47 — PROVIDERS_DATA
  - Current `install.sh` lines 860-875 — Bedrock config generation
  - `crates/cuttlefish-providers/src/bedrock.rs` — Rust provider implementation
  - `cuttlefish.example.toml` — Example config file

  **Acceptance Criteria**:
  - [ ] No references to claude-3-5, claude-3-haiku, etc. remain (except as fallbacks)
  - [ ] Default Bedrock models are Claude 4.x family
  - [ ] Example config shows Claude 4.x models
  - [ ] Documentation updated with correct model IDs

  **QA Scenarios**:
  ```
  Scenario: Verify no outdated model IDs
    Tool: Bash (grep)
    Steps:
      1. grep -r "claude-3-5" --include="*.rs" --include="*.toml" --include="*.sh"
      2. grep -r "claude-3-haiku" --include="*.rs" --include="*.toml" --include="*.sh"
      3. Assert no matches (except intentional fallback/legacy references)
    Expected Result: All primary model references are Claude 4.x
    Evidence: .sisyphus/evidence/install-v2-task3-model-ids.txt
  ```

  **Commit**: YES
  - Message: `fix(providers): update all model IDs to Claude 4.x family`
  - Files: `install.sh`, `cuttlefish.example.toml`, `crates/cuttlefish-providers/src/bedrock.rs`, `docs/providers/bedrock.md`

---

### Wave 2 — Provider Configuration

- [x] 4. Bedrock Regional Model Selection Menu

  **What to do**:
  - Replace the simple Bedrock configuration with an interactive model selector:
  
  ```bash
  configure_bedrock_models() {
      echo ""
      echo -e "${BOLD}=== AWS Bedrock Model Configuration ===${NC}"
      echo ""
      echo "Select your preferred Claude model for Bedrock:"
      echo ""
      echo -e "${YELLOW}Claude Sonnet 4.6 (Recommended for coding):${NC}"
      echo "  1) anthropic.claude-sonnet-4-6              (Default - single region)"
      echo "  2) us.anthropic.claude-sonnet-4-6           (US cross-region)"
      echo "  3) eu.anthropic.claude-sonnet-4-6           (EU cross-region)"
      echo "  4) global.anthropic.claude-sonnet-4-6       (Global cross-region)"
      echo ""
      echo -e "${YELLOW}Claude Opus 4.6 (Most capable):${NC}"
      echo "  5) anthropic.claude-opus-4-6-v1             (Default)"
      echo "  6) us.anthropic.claude-opus-4-6-v1          (US cross-region)"
      echo "  7) eu.anthropic.claude-opus-4-6-v1          (EU cross-region)"
      echo "  8) global.anthropic.claude-opus-4-6-v1      (Global cross-region)"
      echo ""
      echo -e "${YELLOW}Claude Haiku 4.5 (Fast & economical):${NC}"
      echo "  9) anthropic.claude-haiku-4-5-20251001-v1:0 (Default)"
      echo " 10) us.anthropic.claude-haiku-4-5-20251001-v1:0 (US cross-region)"
      echo " 11) global.anthropic.claude-haiku-4-5-20251001-v1:0 (Global)"
      echo ""
      echo -e "${YELLOW}Claude Opus 4.5:${NC}"
      echo " 12) anthropic.claude-opus-4-5-20251101-v1:0  (Default)"
      echo " 13) us.anthropic.claude-opus-4-5-20251101-v1:0 (US cross-region)"
      echo " 14) global.anthropic.claude-opus-4-5-20251101-v1:0 (Global)"
      echo ""
      echo -e "${YELLOW}Claude Sonnet 4.5:${NC}"
      echo " 15) anthropic.claude-sonnet-4-5-20250929-v1:0 (Default)"
      echo " 16) global.anthropic.claude-sonnet-4-5-20250929-v1:0 (Global)"
      echo ""
      echo -e "${CYAN}Custom:${NC}"
      echo " 17) Enter custom model ID"
      echo ""
      
      prompt BEDROCK_MODEL_CHOICE "Select primary model [1-17]" "1"
      
      case "$BEDROCK_MODEL_CHOICE" in
          1)  BEDROCK_PRIMARY_MODEL="anthropic.claude-sonnet-4-6" ;;
          2)  BEDROCK_PRIMARY_MODEL="us.anthropic.claude-sonnet-4-6" ;;
          3)  BEDROCK_PRIMARY_MODEL="eu.anthropic.claude-sonnet-4-6" ;;
          4)  BEDROCK_PRIMARY_MODEL="global.anthropic.claude-sonnet-4-6" ;;
          5)  BEDROCK_PRIMARY_MODEL="anthropic.claude-opus-4-6-v1" ;;
          6)  BEDROCK_PRIMARY_MODEL="us.anthropic.claude-opus-4-6-v1" ;;
          7)  BEDROCK_PRIMARY_MODEL="eu.anthropic.claude-opus-4-6-v1" ;;
          8)  BEDROCK_PRIMARY_MODEL="global.anthropic.claude-opus-4-6-v1" ;;
          9)  BEDROCK_PRIMARY_MODEL="anthropic.claude-haiku-4-5-20251001-v1:0" ;;
          10) BEDROCK_PRIMARY_MODEL="us.anthropic.claude-haiku-4-5-20251001-v1:0" ;;
          11) BEDROCK_PRIMARY_MODEL="global.anthropic.claude-haiku-4-5-20251001-v1:0" ;;
          12) BEDROCK_PRIMARY_MODEL="anthropic.claude-opus-4-5-20251101-v1:0" ;;
          13) BEDROCK_PRIMARY_MODEL="us.anthropic.claude-opus-4-5-20251101-v1:0" ;;
          14) BEDROCK_PRIMARY_MODEL="global.anthropic.claude-opus-4-5-20251101-v1:0" ;;
          15) BEDROCK_PRIMARY_MODEL="anthropic.claude-sonnet-4-5-20250929-v1:0" ;;
          16) BEDROCK_PRIMARY_MODEL="global.anthropic.claude-sonnet-4-5-20250929-v1:0" ;;
          17) prompt BEDROCK_PRIMARY_MODEL "Enter custom model ID" "" ;;
          *)  BEDROCK_PRIMARY_MODEL="anthropic.claude-sonnet-4-6" ;;
      esac
      
      echo ""
      echo "Select fast/economical model (for quick tasks):"
      # Similar menu for fast model selection...
      # Default to Haiku variant matching the region of primary
  }
  ```

  - Automatically suggest matching regional variant for fast model
  - Explain cross-region inference benefits (load balancing, availability)

  **Must NOT do**:
  - Do NOT hardcode region — let user select or use default
  - Do NOT hide the custom option

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 5)
  - **Blocks**: Task 6
  - **Blocked By**: Tasks 1, 3

  **References**:
  - AWS Bedrock inference profiles documentation
  - Current `install.sh` `configure_aws_provider()` function (lines 412-445)

  **Acceptance Criteria**:
  - [ ] All 16+ model options displayed clearly
  - [ ] Regional variants grouped by model family
  - [ ] Custom model ID option works
  - [ ] Selected model written to config correctly
  - [ ] Fast model suggestion matches primary region

  **QA Scenarios**:
  ```
  Scenario: Select US cross-region Sonnet
    Tool: Bash
    Steps:
      1. Run install.sh, select Bedrock provider
      2. At model menu, select option 2 (us.anthropic.claude-sonnet-4-6)
      3. Assert BEDROCK_PRIMARY_MODEL="us.anthropic.claude-sonnet-4-6"
      4. Assert config file contains correct model ID
    Expected Result: US variant selected and configured
    Evidence: .sisyphus/evidence/install-v2-task4-us-region.txt

  Scenario: Custom model ID entry
    Tool: Bash
    Steps:
      1. Run install.sh, select Bedrock provider
      2. At model menu, select option 17 (custom)
      3. Enter "anthropic.claude-sonnet-4-6-20250514-v2:0"
      4. Assert custom model written to config
    Expected Result: Custom model ID accepted
    Evidence: .sisyphus/evidence/install-v2-task4-custom.txt
  ```

  **Commit**: NO (part of larger install.sh rewrite)

---

- [x] 5. Custom Model/Endpoint Configuration

  **What to do**:
  - Add custom API endpoint option for ALL providers that support it:
  
  ```bash
  configure_provider_advanced() {
      local provider_id="$1"
      local provider_name="$2"
      
      if prompt_yn "Configure advanced options for $provider_name?" "n"; then
          echo ""
          echo -e "${BOLD}Advanced Configuration for $provider_name${NC}"
          echo ""
          
          # Custom model
          echo "Default model: ${DEFAULT_MODEL}"
          if prompt_yn "Use a different model?" "n"; then
              prompt CUSTOM_MODEL "Enter model ID/name" "$DEFAULT_MODEL"
          else
              CUSTOM_MODEL="$DEFAULT_MODEL"
          fi
          
          # Custom endpoint (for providers that support it)
          case "$provider_id" in
              anthropic|openai|moonshot|zhipu|minimax|xai|ollama)
                  echo ""
                  echo "Default API endpoint: ${DEFAULT_ENDPOINT}"
                  if prompt_yn "Use a custom API endpoint?" "n"; then
                      prompt CUSTOM_ENDPOINT "Enter API base URL" "$DEFAULT_ENDPOINT"
                  else
                      CUSTOM_ENDPOINT="$DEFAULT_ENDPOINT"
                  fi
                  ;;
          esac
          
          # Provider-specific options
          case "$provider_id" in
              anthropic)
                  echo ""
                  echo "Anthropic API version options:"
                  echo "  1) 2023-06-01 (stable)"
                  echo "  2) 2024-01-01 (latest features)"
                  prompt API_VERSION_CHOICE "Select API version [1-2]" "1"
                  ;;
              bedrock)
                  # Region selection separate from model
                  echo ""
                  echo "AWS Region for Bedrock API calls:"
                  echo "  Note: Cross-region model IDs (us., eu., global.) handle routing automatically"
                  echo "  This setting determines your SOURCE region for API calls"
                  prompt BEDROCK_REGION "AWS region" "us-east-1"
                  ;;
              ollama)
                  echo ""
                  echo "Ollama server URL (default: http://localhost:11434)"
                  prompt OLLAMA_URL "Ollama URL" "http://localhost:11434"
                  ;;
          esac
      fi
  }
  ```

  - Store custom settings in config
  - Document supported customizations per provider

  **Must NOT do**:
  - Do NOT require advanced configuration — make it optional
  - Do NOT break default behavior

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Task 4)
  - **Blocks**: Task 6
  - **Blocked By**: Task 1

  **References**:
  - Current provider configurations in `install.sh`
  - Provider documentation in `docs/providers/`

  **Acceptance Criteria**:
  - [ ] Advanced config is optional (default: skip)
  - [ ] Custom model ID works for all providers
  - [ ] Custom endpoint works for API-based providers
  - [ ] Settings correctly written to config file

  **QA Scenarios**:
  ```
  Scenario: Custom endpoint for Anthropic
    Tool: Bash
    Steps:
      1. Select Anthropic provider
      2. Choose advanced configuration
      3. Enter custom endpoint: https://proxy.example.com/anthropic
      4. Assert config contains base_url = "https://proxy.example.com/anthropic"
    Expected Result: Custom endpoint configured
    Evidence: .sisyphus/evidence/install-v2-task5-custom-endpoint.txt
  ```

  **Commit**: NO (part of larger install.sh rewrite)

---

### Wave FINAL — Integration

- [x] 6. Integration Testing + Documentation

  **What to do**:
  - Run full installer on test environments:
    - Ubuntu 22.04 VM (systemd mode)
    - Ubuntu 22.04 container (Docker mode)
    - macOS (Docker mode with Colima)
    - WSL2 (both modes if systemd available)
  - Fix any issues discovered
  - Update README.md Quick Start section to reflect new features:
    ```markdown
    ## Quick Start
    
    curl -sSL https://raw.githubusercontent.com/JackTYM/cuttlefish-rs/master/install.sh | bash
    
    The installer will:
    1. Detect your platform (Linux/macOS/WSL)
    2. Ask you to choose a deployment mode:
       - **Systemd Service** (recommended) — with Docker sandbox isolation
       - **Docker Container** — simpler setup, no sandbox isolation
    3. Guide you through provider selection and API key setup
    4. Build and install Cuttlefish
    5. Start the service
    ```
  - Add troubleshooting section for common platform-specific issues
  - Run shellcheck on the final script

  **Must NOT do**:
  - Do NOT skip testing on any supported platform
  - Do NOT leave shellcheck warnings

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO — final integration
  - **Blocks**: None
  - **Blocked By**: Tasks 1-5

  **References**:
  - Current README.md Quick Start section
  - ShellCheck: https://www.shellcheck.net/

  **Acceptance Criteria**:
  - [ ] Installer completes successfully on Ubuntu 22.04 (systemd)
  - [ ] Installer completes successfully on Ubuntu 22.04 (Docker mode)
  - [ ] Installer completes successfully on macOS
  - [ ] WSL handling works correctly
  - [ ] shellcheck passes with no errors
  - [ ] README updated with new features

  **QA Scenarios**:
  ```
  Scenario: Full installation on Ubuntu (systemd mode)
    Tool: Bash (clean VM)
    Steps:
      1. Fresh Ubuntu 22.04 VM
      2. curl -sSL .../install.sh | bash
      3. Select systemd mode
      4. Select Bedrock provider, choose global Sonnet 4.6
      5. Complete installation
      6. Assert: systemctl status cuttlefish shows "active"
      7. Assert: curl localhost:8080/health returns 200
    Expected Result: Full working installation
    Evidence: .sisyphus/evidence/install-v2-task6-ubuntu-systemd.txt

  Scenario: Full installation on macOS (Docker mode)
    Tool: Bash (macOS)
    Steps:
      1. Run install.sh
      2. Select Docker mode (accept warning)
      3. Select Colima for Docker
      4. Select Anthropic API provider
      5. Complete installation
      6. Assert: docker ps shows cuttlefish container
      7. Assert: curl localhost:8080/health returns 200
    Expected Result: Full working installation on macOS
    Evidence: .sisyphus/evidence/install-v2-task6-macos.txt
  ```

  **Commit**: YES
  - Message: `feat(install): complete install.sh v2 with cross-platform support and Bedrock regional models`
  - Files: `install.sh`, `README.md`
  - Pre-commit: `shellcheck install.sh`

---

## Final Verification

After all tasks complete:
- [ ] Run shellcheck on install.sh — must pass
- [ ] Test on Ubuntu 22.04 (systemd) — must complete
- [ ] Test on macOS (Docker) — must complete  
- [ ] Verify all Bedrock model variants selectable
- [ ] Verify custom model/endpoint configuration works
- [ ] Verify Docker mode warning displays correctly
- [ ] README reflects all new capabilities

---

## Commit Strategy

| Task | Commit | Pre-commit Check |
|------|--------|------------------|
| 3 | `fix(providers): update all model IDs to Claude 4.x family` | `grep -r "claude-3-5"` returns nothing |
| 6 | `feat(install): complete install.sh v2 with cross-platform support and Bedrock regional models` | `shellcheck install.sh` |

---

## Success Criteria

### Verification Commands
```bash
# ShellCheck validation
shellcheck install.sh

# Model ID verification
grep -r "claude-3-5" --include="*.rs" --include="*.toml" --include="*.sh" | wc -l
# Expected: 0 (or only in legacy/fallback comments)

# Test installation (Ubuntu)
./install.sh --no-root-check  # For testing without sudo
```

### Final Checklist
- [ ] Platform detection works for Linux, macOS, WSL
- [ ] Deployment mode selection with Docker warning
- [ ] Cross-platform dependency installation
- [ ] Bedrock shows all regional variants (default, us., eu., global.)
- [ ] Custom model/endpoint configuration available
- [ ] All model IDs updated to Claude 4.x
- [ ] shellcheck passes
- [ ] README updated
