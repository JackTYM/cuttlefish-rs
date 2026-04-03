# Learnings — install-sh-v2

## 2026-04-02 Session Start

### install.sh Structure (1128 lines)
- Lines 1-75: Banner, colors, global vars, PROVIDERS_DATA
- Lines 77-170: Utility functions (print, prompt, toggle_provider)
- Lines 173-270: Provider menu (print_provider_menu, select_providers)
- Lines 271-460: Credential collection (collect_provider_credentials, configure_ollama, configure_aws_provider, configure_azure_provider)
- Lines 462-578: Dependency checks (check_dependencies, install_dependencies, install_rust)
- Lines 580-637: Configuration (configure_server, configure_database, configure_sandbox, configure_discord)
- Lines 644-712: Installation (create_user, create_directories, build_cuttlefish)
- Lines 714-927: Config writing (write_config — generates cuttlefish.toml)
- Lines 929-989: Env file writing (write_env_file)
- Lines 991-1028: Systemd service (write_systemd_service)
- Lines 1030-1084: Finalization (print_summary)
- Lines 1086-1128: main() function

### Key Outdated Model IDs (lines 860-875)
- `anthropic.claude-3-5-sonnet-20241022-v2:0` → needs update to `anthropic.claude-sonnet-4-6`
- `anthropic.claude-3-haiku-20240307-v1:0` → needs update to `anthropic.claude-haiku-4-5-20251001-v1:0`

### Bedrock Provider (crates/cuttlefish-providers/src/bedrock.rs)
- Uses `model_id: String` field — no hardcoded model IDs in Rust code
- Model IDs come from config/install.sh only
- No claude-3 references in Rust code — only in install.sh

### Platform Detection Approach
- `$OSTYPE == "darwin"*` → macOS
- `grep -qi microsoft /proc/version` → WSL
- `/etc/os-release` → Linux (with `$ID` for distro)

### Deployment Mode Variables
- `DEPLOYMENT_MODE` = "systemd" | "docker"
- Docker mode requires prominent warning box + confirmation

### Task File Conflicts
- Tasks 1 and 2 BOTH modify install.sh → must be SEQUENTIAL (Task 1 first, then Task 2)
- Task 3 modifies: install.sh lines 860-875, cuttlefish.example.toml, docs/providers/bedrock.md
  → Task 3 can run AFTER Task 1 (since Task 1 restructures the file)
  → Actually Task 3 should run AFTER Task 1 to avoid conflicts

### Execution Order
- Task 1 first (platform detection + deployment mode — restructures main() and adds new functions)
- Task 2 after Task 1 (cross-platform deps — rewrites install_dependencies())
- Task 3 after Task 1 (model ID updates — modifies write_config() Bedrock section)
- Tasks 4 and 5 after Tasks 1+3 (Bedrock menu + custom endpoint — extend configure_aws_provider())
- Task 6 last (integration + docs)

## Task 3 Completion (2026-04-02)

### Changes Made
1. **install.sh** (lines 1263, 1268):
   - Line 1263: `anthropic.claude-3-5-sonnet-20241022-v2:0` → `anthropic.claude-sonnet-4-6`
   - Line 1268: `anthropic.claude-3-haiku-20240307-v1:0` → `anthropic.claude-haiku-4-5-20251001-v1:0`

2. **cuttlefish.example.toml**:
   - Already has correct Claude 4.x format (no changes needed)
   - Line 32: `anthropic.claude-sonnet-4-6-20260101-v1:0` ✓
   - Line 37: `anthropic.claude-haiku-4-5-20260101-v1:0` ✓

3. **docs/providers/bedrock.md**:
   - File does not exist (no action needed)

### Verification
- ✓ `bash -n install.sh` passes syntax check
- ✓ No `claude-3-5` or `claude-3-haiku` references remain in primary configs
- ✓ All Bedrock model IDs updated to Claude 4.x family

## Task: Advanced Provider Configuration (2026-04-02)

### Changes Made
1. **Global Variables** (after line 69):
   - Added 16 new vars: `*_CUSTOM_MODEL` and `*_BASE_URL` for each provider
   - `OLLAMA_BASE_URL` defaults to `http://localhost:11434`

2. **configure_provider_advanced()** function (after collect_provider_credentials):
   - ~70 lines of code
   - Accepts `provider_id` and `provider_name` args
   - Prompts for custom model ID (supported: anthropic, openai, google, moonshot, zhipu, minimax, xai, ollama)
   - Prompts for custom API endpoint (supported: anthropic, openai, moonshot, zhipu, minimax, xai, ollama)
   - Both prompts default to "n" (skip)

3. **collect_provider_credentials()** calls:
   - Added `configure_provider_advanced "$pid" "$pname"` after `api)` case success message
   - Added `configure_provider_advanced "$pid" "$pname"` after `local)` case configure_ollama

4. **write_config()** updates:
   - All provider cases now use `${*_CUSTOM_MODEL:-default}` for model
   - Anthropic, OpenAI: uses sed to append base_url after api_key_env line if set
   - Moonshot, Zhipu, Minimax, xAI: uses sed to append base_url if set
   - Ollama: directly uses `$ollama_url` in heredoc (already had base_url)
   - Google: no base_url support (API doesn't support custom endpoints)
   - Bedrock, Azure, Claude OAuth: unchanged (use their own config flows)

### Verification
- ✓ `bash -n install.sh` passes
- install.sh now ~1800 lines
