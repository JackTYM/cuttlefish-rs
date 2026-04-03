# install.sh Rewrite — Systemd + Docker Mode with Enhanced Provider Setup

## TL;DR

> **Quick Summary**: Rewrite install.sh to support systemd (recommended) and Docker deployment modes, with enhanced Bedrock model selection including regional variants, custom model support, and cross-platform compatibility.
> 
> **Deliverables**:
> - Complete rewrite of `install.sh` with deployment mode selection
> - Docker mode with warning about no sandbox isolation
> - Bedrock model selection with US/EU/Global inference profiles
> - Custom model and API endpoint support
> - Cross-platform support (Linux, macOS, WSL)
> 
> **Estimated Effort**: Medium
> **Parallel Execution**: NO - single file rewrite
> **Critical Path**: T1 → T2 → T3 → T4 → T5

---

## Context

### Original Request
User wants install.sh to:
1. Recommend systemd deployment (with Docker for sandboxes)
2. Offer Docker mode option with warning about no sandbox isolation
3. Support custom models and API endpoints
4. Show all Bedrock model strings including US, EU, and Global inference profiles
5. Cross-platform support with proper dependency handling

### Research Findings
From models.dev API, Bedrock Claude models have regional variants:
- **Default**: `anthropic.claude-sonnet-4-6`
- **US Region**: `us.anthropic.claude-sonnet-4-6`
- **EU Region**: `eu.anthropic.claude-sonnet-4-6`
- **Global Inference**: `global.anthropic.claude-sonnet-4-6`

Available Claude 4.x models on Bedrock:
- `anthropic.claude-opus-4-6-v1`
- `anthropic.claude-opus-4-5-20251101-v1:0`
- `anthropic.claude-sonnet-4-6`
- `anthropic.claude-sonnet-4-5-20250929-v1:0`
- `anthropic.claude-haiku-4-5-20251001-v1:0`

Other Bedrock models: Nova, DeepSeek, Llama, Mistral, Qwen

---

## Work Objectives

### Core Objective
Rewrite install.sh to be a comprehensive, cross-platform installer that recommends systemd with Docker sandboxes, but offers Docker-only mode with appropriate warnings.

### Concrete Deliverables
- `install.sh` — Complete rewrite with all features

### Definition of Done
- [ ] `bash -n install.sh` passes (no syntax errors)
- [ ] Deployment mode selection works (systemd vs docker)
- [ ] Docker mode shows sandbox isolation warning
- [ ] Bedrock model selection shows regional variants
- [ ] Custom model/endpoint input works
- [ ] Cross-platform detection works (Linux, macOS, WSL)
- [ ] All existing functionality preserved

### Must Have
- Deployment mode selection (systemd recommended, docker optional)
- Clear warning about Docker mode lacking sandbox isolation
- Bedrock model selection with US/EU/Global inference profiles
- Custom model ID input option
- Custom API endpoint support for OpenAI-compatible providers
- Cross-platform support (Linux apt/dnf/pacman, macOS brew, WSL)

### Must NOT Have (Guardrails)
- Don't remove any existing provider support
- Don't require root for all operations (only when necessary)
- Don't hardcode paths that vary by platform
- Don't skip the Docker warning in Docker mode

---

## Verification Strategy

### Test Decision
- **Infrastructure exists**: NO (bash script)
- **Automated tests**: None — manual verification
- **Framework**: N/A

### QA Policy
Manual verification via bash syntax check and dry-run testing.

---

## Execution Strategy

### Sequential Execution (Single File)

This is a single-file rewrite, executed sequentially.

---

## TODOs

- [ ] 1. Platform Detection and Deployment Mode Selection

  **What to do**:
  - Add platform detection (Linux, macOS, WSL, Windows)
  - Detect package manager (apt, dnf, pacman, brew)
  - Check for systemd availability
  - Add deployment mode selection menu:
    - Option 1: Systemd Service (Recommended) — with Docker sandboxes
    - Option 2: Docker Container — with WARNING about no sandbox isolation
  - Show prominent warning box when Docker mode selected

  **Must NOT do**:
  - Don't assume Linux-only
  - Don't force root when not needed

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: T2, T3, T4, T5
  - **Blocked By**: None

  **References**:
  - `install.sh:464-561` — Current dependency checking
  - OpenCode's `install` script for cross-platform patterns

  **Acceptance Criteria**:
  - [ ] Platform correctly detected on Linux, macOS, WSL
  - [ ] Deployment mode menu displays correctly
  - [ ] Docker mode shows prominent warning

  **QA Scenarios**:
  ```
  Scenario: Platform detection on Linux
    Tool: Bash
    Steps:
      1. Run: bash -c 'source install.sh; detect_platform'
      2. Verify PLATFORM variable is set to "linux"
    Expected: PLATFORM="linux", HAS_SYSTEMD=true

  Scenario: Docker mode warning displays
    Tool: Bash
    Steps:
      1. Run install.sh, select option 2 (Docker mode)
      2. Verify warning box appears with sandbox isolation warning
    Expected: Yellow warning box with "No project sandbox isolation" message
  ```

  **Commit**: YES
  - Message: `feat(install): add platform detection and deployment mode selection`
  - Files: `install.sh`

---

- [ ] 2. Enhanced Bedrock Model Selection

  **What to do**:
  - Add region/inference profile selection:
    - Default (us-east-1): `anthropic.claude-*`
    - US Region Profile: `us.anthropic.claude-*`
    - EU Region Profile: `eu.anthropic.claude-*`
    - Global Inference: `global.anthropic.claude-*`
    - Custom model ID
  - Add Claude model tier selection:
    - Flagship: Opus 4.6, Opus 4.5
    - Balanced: Sonnet 4.6, Sonnet 4.5
    - Fast: Haiku 4.5
  - Add other Bedrock model options:
    - Amazon Nova Pro/Lite
    - DeepSeek R1
    - Llama 3.3 70B
    - Mistral Large 3
  - Support custom model ID entry

  **Must NOT do**:
  - Don't remove existing Bedrock support
  - Don't hardcode model versions that will expire

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: T3, T4, T5
  - **Blocked By**: T1

  **References**:
  - `install.sh:412-445` — Current AWS configuration
  - models.dev API output for Bedrock models

  **Acceptance Criteria**:
  - [ ] Region selection menu works
  - [ ] Model tier selection works
  - [ ] Custom model ID entry works
  - [ ] Final model string is correctly formatted

  **QA Scenarios**:
  ```
  Scenario: Select EU region with Sonnet 4.6
    Tool: Bash
    Steps:
      1. Run Bedrock configuration
      2. Select "EU Region Profile"
      3. Select "Claude Sonnet 4.6"
    Expected: BEDROCK_MODEL="eu.anthropic.claude-sonnet-4-6"

  Scenario: Custom model ID entry
    Tool: Bash
    Steps:
      1. Run Bedrock configuration
      2. Select "Custom model ID"
      3. Enter "meta.llama3-3-70b-instruct-v1:0"
    Expected: BEDROCK_MODEL="meta.llama3-3-70b-instruct-v1:0"
  ```

  **Commit**: YES
  - Message: `feat(install): add enhanced Bedrock model selection with regions`
  - Files: `install.sh`

---

- [ ] 3. Custom API Endpoint Support

  **What to do**:
  - Add custom API endpoint option for OpenAI provider
  - Support Azure OpenAI endpoints
  - Support proxy endpoints (e.g., for rate limiting)
  - Add base_url configuration to generated config

  **Must NOT do**:
  - Don't require custom endpoints
  - Don't break standard API usage

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: T4, T5
  - **Blocked By**: T2

  **References**:
  - `install.sh:766-781` — Current OpenAI configuration

  **Acceptance Criteria**:
  - [ ] Custom endpoint prompt appears when requested
  - [ ] base_url written to config when provided
  - [ ] Standard API works without custom endpoint

  **QA Scenarios**:
  ```
  Scenario: Custom OpenAI endpoint
    Tool: Bash
    Steps:
      1. Configure OpenAI provider
      2. Select "Use custom API endpoint"
      3. Enter "https://my-proxy.example.com/v1"
    Expected: Config contains base_url = "https://my-proxy.example.com/v1"
  ```

  **Commit**: YES (with T2)
  - Message: `feat(install): add custom API endpoint support`
  - Files: `install.sh`

---

- [ ] 4. Systemd and Docker Deployment

  **What to do**:
  - Update systemd service file generation
  - Add Docker Compose file generation for Docker mode
  - Docker mode config should disable sandbox (enabled = false)
  - Add appropriate volume mounts for Docker mode

  **Must NOT do**:
  - Don't mount docker.sock in Docker mode (defeats the purpose)
  - Don't enable sandboxes in Docker mode

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: T5
  - **Blocked By**: T3

  **References**:
  - `install.sh:991-1028` — Current systemd service generation

  **Acceptance Criteria**:
  - [ ] Systemd service file generated correctly
  - [ ] Docker Compose file generated for Docker mode
  - [ ] Sandbox disabled in Docker mode config

  **QA Scenarios**:
  ```
  Scenario: Systemd mode generates service file
    Tool: Bash
    Steps:
      1. Run install with systemd mode
      2. Check /etc/systemd/system/cuttlefish.service exists
    Expected: Service file exists with correct ExecStart

  Scenario: Docker mode generates compose file
    Tool: Bash
    Steps:
      1. Run install with docker mode
      2. Check docker-compose.yml exists
      3. Verify no docker.sock mount
    Expected: Compose file without docker.sock volume
  ```

  **Commit**: YES
  - Message: `feat(install): add Docker deployment mode with compose file`
  - Files: `install.sh`

---

- [ ] 5. Cross-Platform Dependency Installation

  **What to do**:
  - Update dependency installation for all platforms
  - Add Homebrew installation for macOS
  - Handle Docker Desktop on macOS
  - Support WSL-specific configurations
  - Add launchd option mention for macOS

  **Must NOT do**:
  - Don't assume apt is available
  - Don't break on missing package manager

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Blocks**: None
  - **Blocked By**: T4

  **References**:
  - `install.sh:515-561` — Current install_dependencies

  **Acceptance Criteria**:
  - [ ] Linux package managers supported (apt, dnf, pacman)
  - [ ] macOS Homebrew supported
  - [ ] WSL detected and handled

  **QA Scenarios**:
  ```
  Scenario: macOS uses Homebrew
    Tool: Bash (on macOS)
    Steps:
      1. Run install.sh on macOS
      2. Verify Homebrew used for dependencies
    Expected: brew install commands used

  Scenario: Linux apt detected
    Tool: Bash (on Ubuntu/Debian)
    Steps:
      1. Run install.sh
      2. Verify apt-get used
    Expected: apt-get install commands used
  ```

  **Commit**: YES
  - Message: `feat(install): add cross-platform dependency installation`
  - Files: `install.sh`

---

## Final Verification Wave

- [ ] F1. **Syntax and Structure Check**
  Run `bash -n install.sh` and `shellcheck install.sh` to verify script validity.

- [ ] F2. **Dry Run Test**
  Run install.sh in a test environment and verify all menus display correctly.

- [ ] F3. **Documentation Update**
  Update README.md Quick Start section if install.sh command changes.

---

## Commit Strategy

Single commit after all tasks complete:
- Message: `feat(install): rewrite with systemd/docker modes, bedrock regions, cross-platform`
- Files: `install.sh`, potentially `README.md`

---

## Success Criteria

### Verification Commands
```bash
bash -n install.sh  # No syntax errors
shellcheck install.sh  # No critical warnings
```

### Final Checklist
- [ ] Deployment mode selection works
- [ ] Docker mode warning displays prominently
- [ ] Bedrock regional variants selectable
- [ ] Custom model ID supported
- [ ] Cross-platform detection works
- [ ] All existing providers still work
