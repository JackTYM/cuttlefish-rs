# V1 Template System — Marketplace, Variable Substitution & Starter Templates

## TL;DR

> **Quick Summary**: Build out Cuttlefish's template system with 5+ starter templates, YAML frontmatter metadata, variable substitution (`{{project_name}}`), remote GitHub fetching, and API/WebUI for browsing.
> 
> **Deliverables**:
> - 5+ starter template files in `templates/` directory
> - Template manifest parser (YAML frontmatter)
> - Variable substitution engine (Tera-based)
> - GitHub template fetching (remote marketplace)
> - REST API endpoints for template CRUD + browsing
> - WebUI template gallery component
> 
> **Estimated Effort**: Medium (2-3 days)
> **Parallel Execution**: YES - 4 waves
> **Critical Path**: Task 1 (Manifest) → Task 2-4 (Core + Templates) → Task 5-7 (API + Remote) → Task 8 (WebUI)

---

## Context

### Original Request
Build a comprehensive template system with marketplace functionality. User stated "Templates come FIRST - easy, no runtime risk, immediately useful, community-driven."

### Current State (Research Findings)
**Already exists:**
- `templates` table with CRUD in `cuttlefish-db`
- `Template` model (id, name, description, content_md, language, created_at)
- `ProjectTemplate` struct + `load_templates_from_dir()` in `cuttlefish-core/src/advanced.rs`
- Docker `ImageRegistry` mapping templates → base images
- API accepts `template` parameter on project creation

**Missing:**
- No actual template files (empty `templates/` directory)
- No manifest/metadata format (YAML frontmatter)
- No variable substitution (`{{project_name}}`)
- No remote GitHub template fetching
- No API endpoints for template browsing
- No WebUI template gallery

### Metis Review
**Identified Gaps** (addressed in plan):
- Need clear manifest format that extends existing `ProjectTemplate` struct
- Variable substitution should use Tera (already in Rust ecosystem) not Handlebars
- GitHub fetching needs caching to avoid rate limits
- WebUI gallery needs to work without JS framework changes

---

## Work Objectives

### Core Objective
Transform Cuttlefish's scaffolded template infrastructure into a fully functional template marketplace with starter templates, variable substitution, and remote fetching.

### Concrete Deliverables
- `templates/nuxt-cloudflare.md` (TypeScript/Vue template)
- `templates/fastapi-postgres.md` (Python template)  
- `templates/rust-axum.md` (Rust template)
- `templates/discord-bot.md` (Discord.js template)
- `templates/static-site.md` (HTML/CSS/JS template)
- `crates/cuttlefish-core/src/template_manifest.rs` (YAML frontmatter parser)
- `crates/cuttlefish-core/src/template_engine.rs` (Tera variable substitution)
- `crates/cuttlefish-core/src/template_registry.rs` (local + remote registry)
- REST API endpoints: `GET /api/templates`, `GET /api/templates/:name`, `POST /api/templates/fetch`
- WebUI: Template gallery page

### Definition of Done
- [ ] `cargo test --workspace` passes with new template tests
- [ ] `cargo clippy --workspace -- -D warnings` clean
- [ ] 5+ template files exist with valid YAML frontmatter
- [ ] Variable substitution works (`{{project_name}}` → actual name)
- [ ] Can fetch templates from GitHub URL
- [ ] API endpoints return template list

### Must Have
- YAML frontmatter in each template (name, description, language, docker_image, variables)
- Tera-based variable substitution (safe, sandboxed)
- Local templates from `templates/` directory
- API for listing and retrieving templates
- Error handling for malformed templates

### Must NOT Have (Guardrails)
- No unsafe code
- No `unwrap()` — use `?` or `expect("reason")`
- No arbitrary code execution in templates (Tera only, no Lua/JS)
- No changes to database schema (use existing `templates` table)
- No breaking changes to existing `ProjectTemplate` struct
- No hardcoded GitHub tokens (use optional env var)

---

## Verification Strategy

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed.

### Test Decision
- **Infrastructure exists**: YES (176 tests pass)
- **Automated tests**: YES (Tests-after)
- **Framework**: Rust's `#[test]` and `#[tokio::test]`

### QA Policy
Every task includes agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Rust code**: Use `cargo test`, `cargo clippy`, `cargo build`
- **Template files**: Validate YAML frontmatter, check variable syntax
- **API**: Use `curl` to test endpoints
- **Integration**: Create project from template, verify substitutions

---

## Execution Strategy

### Parallel Execution Waves

```
Wave 1 (Foundation — manifest parsing + core types):
├── Task 1: Template manifest parser (YAML frontmatter) [quick]
└── Task 2: Variable substitution engine (Tera) [quick]

Wave 2 (Content — templates + local loading):
├── Task 3: Create 5+ starter template files [quick]
├── Task 4: Template registry (local loading) [unspecified-high]
└── Task 5: Wire templates into project creation [unspecified-high]

Wave 3 (Remote — GitHub fetching + API):
├── Task 6: GitHub template fetching [unspecified-high]
├── Task 7: REST API endpoints [unspecified-high]
└── Task 8: Template validation CLI command [quick]

Wave 4 (UI — WebUI gallery):
└── Task 9: WebUI template gallery component [visual-engineering]

Wave FINAL (Verification):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Integration QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → Task 4 → Task 5 → Task 7 → F1-F4 → user okay
Parallel Speedup: ~60% faster than sequential
Max Concurrent: 3 (Waves 2 & 3)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|------------|--------|------|
| 1 | — | 3, 4, 5 | 1 |
| 2 | — | 4, 5 | 1 |
| 3 | 1 | 5, 7 | 2 |
| 4 | 1, 2 | 5, 6, 7 | 2 |
| 5 | 1, 2, 3, 4 | 7 | 2 |
| 6 | 4 | 7 | 3 |
| 7 | 3, 4, 5, 6 | 9 | 3 |
| 8 | — | — | 3 |
| 9 | 7 | F1-F4 | 4 |

### Agent Dispatch Summary

- **Wave 1**: 2 tasks — T1 → `quick`, T2 → `quick`
- **Wave 2**: 3 tasks — T3 → `quick`, T4 → `unspecified-high`, T5 → `unspecified-high`
- **Wave 3**: 3 tasks — T6 → `unspecified-high`, T7 → `unspecified-high`, T8 → `quick`
- **Wave 4**: 1 task — T9 → `visual-engineering`
- **FINAL**: 4 tasks — F1 → `oracle`, F2 → `unspecified-high`, F3 → `unspecified-high`, F4 → `deep`

---

## TODOs

- [ ] 1. Template Manifest Parser (YAML Frontmatter)

  **What to do**:
  - Create `crates/cuttlefish-core/src/template_manifest.rs`
  - Define `TemplateManifest` struct with: name, description, language, docker_image, variables (Vec<TemplateVariable>), author, version, tags
  - Define `TemplateVariable` struct with: name, description, default, required
  - Implement `parse_manifest(content: &str) -> Result<(TemplateManifest, String), TemplateError>` that extracts YAML frontmatter (between `---` delimiters) and returns remaining content
  - Add `TemplateError` variants to `crates/cuttlefish-core/src/error.rs`
  - Export from `crates/cuttlefish-core/src/lib.rs`
  - Write unit tests for parsing valid/invalid frontmatter

  **Must NOT do**:
  - Don't modify existing `ProjectTemplate` struct (extend separately)
  - Don't use external YAML parser beyond `serde_yaml`

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Single file, well-defined scope, follows existing patterns
  - **Skills**: []
    - No special skills needed

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 2)
  - **Blocks**: Tasks 3, 4, 5
  - **Blocked By**: None

  **References**:
  - `crates/cuttlefish-core/src/advanced.rs:47-58` - Existing `ProjectTemplate` struct to extend
  - `crates/cuttlefish-core/src/error.rs` - Error type patterns
  - `crates/cuttlefish-agents/src/prompt_registry.rs` (from v1-agents) - YAML frontmatter parsing pattern

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-core template_manifest` → PASS
  - [ ] `cargo clippy -p cuttlefish-core -- -D warnings` → clean

  **QA Scenarios**:
  ```
  Scenario: Parse valid template manifest
    Tool: Bash (cargo test)
    Preconditions: Template manifest module exists
    Steps:
      1. Run `cargo test -p cuttlefish-core template_manifest::tests`
      2. Verify test for valid YAML frontmatter passes
    Expected Result: All tests pass, manifest fields correctly extracted
    Evidence: .sisyphus/evidence/task-1-manifest-parse.txt

  Scenario: Handle invalid YAML gracefully
    Tool: Bash (cargo test)
    Preconditions: Error handling tests exist
    Steps:
      1. Run `cargo test -p cuttlefish-core template_manifest::tests::test_invalid_yaml`
      2. Verify returns TemplateError, not panic
    Expected Result: Returns Err variant with descriptive message
    Evidence: .sisyphus/evidence/task-1-manifest-error.txt
  ```

  **Commit**: YES (groups with Task 2)
  - Message: `feat(core): add template manifest parser and Tera engine`
  - Files: `crates/cuttlefish-core/src/template_manifest.rs`, `crates/cuttlefish-core/src/error.rs`, `crates/cuttlefish-core/src/lib.rs`

- [ ] 2. Variable Substitution Engine (Tera)

  **What to do**:
  - Add `tera = "1"` to `crates/cuttlefish-core/Cargo.toml`
  - Create `crates/cuttlefish-core/src/template_engine.rs`
  - Implement `TemplateEngine` struct wrapping `Tera`
  - Implement `render(template_content: &str, variables: &HashMap<String, String>) -> Result<String, TemplateError>`
  - Built-in variables: `project_name`, `project_description`, `timestamp`, `uuid`
  - Validate variable names (alphanumeric + underscore only)
  - Export from `crates/cuttlefish-core/src/lib.rs`
  - Write unit tests for substitution

  **Must NOT do**:
  - Don't allow arbitrary Tera filters (security risk) — allowlist only
  - Don't use `include` or `extends` (file system access)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Single file, well-defined scope, Tera is straightforward
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1 (with Task 1)
  - **Blocks**: Tasks 4, 5
  - **Blocked By**: None

  **References**:
  - Tera docs: https://keats.github.io/tera/docs/
  - `crates/cuttlefish-core/Cargo.toml` - Dependency pattern

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-core template_engine` → PASS
  - [ ] Tera dependency added to Cargo.toml
  - [ ] `{{ project_name }}` substitutes correctly

  **QA Scenarios**:
  ```
  Scenario: Substitute project_name variable
    Tool: Bash (cargo test)
    Preconditions: TemplateEngine module exists
    Steps:
      1. Run `cargo test -p cuttlefish-core template_engine::tests::test_basic_substitution`
      2. Template "Hello {{ project_name }}" with {"project_name": "MyApp"} → "Hello MyApp"
    Expected Result: Substitution works correctly
    Evidence: .sisyphus/evidence/task-2-substitution.txt

  Scenario: Reject dangerous Tera features
    Tool: Bash (cargo test)
    Preconditions: Security tests exist
    Steps:
      1. Run `cargo test -p cuttlefish-core template_engine::tests::test_no_include`
      2. Template with `{% include "secret.txt" %}` should error
    Expected Result: Returns Err, does not read files
    Evidence: .sisyphus/evidence/task-2-security.txt
  ```

  **Commit**: YES (groups with Task 1)
  - Message: `feat(core): add template manifest parser and Tera engine`
  - Files: `crates/cuttlefish-core/src/template_engine.rs`, `crates/cuttlefish-core/Cargo.toml`

- [ ] 3. Create 5+ Starter Template Files

  **What to do**:
  - Create `templates/` directory in project root
  - Create `templates/nuxt-cloudflare.md` — Nuxt 3 + Cloudflare Pages template:
    - YAML frontmatter with variables: project_name, description
    - Project structure: nuxt.config.ts, app.vue, pages/, server/
    - Cloudflare wrangler.toml, deployment instructions
  - Create `templates/fastapi-postgres.md` — FastAPI + PostgreSQL template:
    - Python project with pyproject.toml, main.py, models.py
    - Alembic migrations, Docker Compose for Postgres
  - Create `templates/rust-axum.md` — Rust Axum web server template:
    - Cargo workspace, axum routes, sqlx for DB
    - Docker multi-stage build
  - Create `templates/discord-bot.md` — Discord.js bot template:
    - Node.js with TypeScript, slash commands
    - Docker deployment
  - Create `templates/static-site.md` — Simple HTML/CSS/JS template:
    - index.html, styles.css, main.js
    - No build step, just files
  - Each template must have valid YAML frontmatter

  **Must NOT do**:
  - Don't include actual secrets or API keys (use placeholders)
  - Don't make templates too complex (starter-level)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: Content creation, not complex code
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 4, 5)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 5, 7
  - **Blocked By**: Task 1 (needs manifest format)

  **References**:
  - Existing README architecture section for stack patterns
  - Similar templates: create-t3-app, Nuxt starter, FastAPI cookiecutter

  **Acceptance Criteria**:
  - [ ] 5 template files exist in `templates/`
  - [ ] Each has valid YAML frontmatter (parsed by Task 1's parser)
  - [ ] Each has meaningful content (not placeholders)

  **QA Scenarios**:
  ```
  Scenario: Template files have valid frontmatter
    Tool: Bash
    Preconditions: Templates directory exists with files
    Steps:
      1. Run `ls templates/*.md | wc -l`
      2. For each file, verify YAML frontmatter exists between --- delimiters
      3. Run parser test against each file
    Expected Result: 5+ files, all parse successfully
    Evidence: .sisyphus/evidence/task-3-templates-list.txt

  Scenario: Nuxt template contains required files
    Tool: Bash (grep)
    Preconditions: nuxt-cloudflare.md exists
    Steps:
      1. grep for "nuxt.config.ts" in template
      2. grep for "wrangler.toml" in template
    Expected Result: Both patterns found
    Evidence: .sisyphus/evidence/task-3-nuxt-content.txt
  ```

  **Commit**: YES
  - Message: `feat(templates): add 5 starter project templates`
  - Files: `templates/*.md`

- [ ] 4. Template Registry (Local Loading)

  **What to do**:
  - Create `crates/cuttlefish-core/src/template_registry.rs`
  - Implement `TemplateRegistry` struct:
    ```rust
    pub struct TemplateRegistry {
        templates: HashMap<String, LoadedTemplate>,
        cache_dir: PathBuf,
    }
    
    pub struct LoadedTemplate {
        pub manifest: TemplateManifest,
        pub content: String,
        pub source: TemplateSource,  // Local or Remote
    }
    ```
  - Implement `load_from_dir(dir: &Path) -> Result<Self, TemplateError>`:
    - Iterate `.md` files
    - Parse frontmatter using Task 1's parser
    - Store in HashMap by name
  - Implement `get(&self, name: &str) -> Option<&LoadedTemplate>`
  - Implement `list(&self) -> Vec<&LoadedTemplate>`
  - Implement `render(&self, name: &str, vars: &HashMap<String, String>) -> Result<String, TemplateError>`:
    - Get template
    - Apply variable substitution using Task 2's engine
  - Export from lib.rs
  - Write tests

  **Must NOT do**:
  - Don't load templates on every access (cache in memory)
  - Don't modify existing `load_templates_from_dir()` (new registry wraps it)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Core data structure with multiple methods
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 3, 5)
  - **Parallel Group**: Wave 2
  - **Blocks**: Tasks 5, 6, 7
  - **Blocked By**: Tasks 1, 2

  **References**:
  - `crates/cuttlefish-core/src/advanced.rs:65-126` — Existing `load_templates_from_dir()`
  - `crates/cuttlefish-sandbox/src/images.rs` — Registry pattern

  **Acceptance Criteria**:
  - [ ] `cargo test -p cuttlefish-core template_registry` → PASS
  - [ ] Registry loads templates from directory
  - [ ] `get()` and `list()` work correctly
  - [ ] `render()` applies substitutions

  **QA Scenarios**:
  ```
  Scenario: Load templates from directory
    Tool: Bash (cargo test)
    Preconditions: Registry module and template files exist
    Steps:
      1. Run `cargo test -p cuttlefish-core template_registry::tests::test_load_from_dir`
      2. Verify 5 templates loaded
    Expected Result: All templates accessible via get()
    Evidence: .sisyphus/evidence/task-4-registry-load.txt

  Scenario: Render template with variables
    Tool: Bash (cargo test)
    Preconditions: Registry loaded with test template
    Steps:
      1. Run `cargo test -p cuttlefish-core template_registry::tests::test_render`
      2. Verify {{ project_name }} replaced
    Expected Result: Variables substituted correctly
    Evidence: .sisyphus/evidence/task-4-registry-render.txt
  ```

  **Commit**: YES
  - Message: `feat(core): add template registry with local loading`
  - Files: `crates/cuttlefish-core/src/template_registry.rs`, `crates/cuttlefish-core/src/lib.rs`

- [ ] 5. Wire Templates into Project Creation

  **What to do**:
  - Modify `crates/cuttlefish-api/src/api_routes.rs`:
    - Inject `TemplateRegistry` into `AppState`
    - In `create_project()`, if `template` provided:
      - Look up template in registry
      - Render with `{ project_name, description }`
      - Store rendered content (or use for scaffold)
  - Modify `crates/cuttlefish-agents/src/orchestrator.rs`:
    - When starting new project, load template context
    - Include rendered template in initial prompt to agents
  - Add integration test for template-based project creation

  **Must NOT do**:
  - Don't change database schema
  - Don't auto-scaffold files yet (just provide context to agents)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Integration across multiple crates
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 2 (after Tasks 1-4)
  - **Blocks**: Task 7
  - **Blocked By**: Tasks 1, 2, 3, 4

  **References**:
  - `crates/cuttlefish-api/src/api_routes.rs:37-51` — Current `create_project()`
  - `crates/cuttlefish-api/src/routes.rs` — AppState pattern

  **Acceptance Criteria**:
  - [ ] Project creation with template parameter works
  - [ ] Template content available to agents
  - [ ] API returns project with template_name set

  **QA Scenarios**:
  ```
  Scenario: Create project with template
    Tool: Bash (curl)
    Preconditions: Server running with templates loaded
    Steps:
      1. POST /api/projects with {"name": "test", "description": "Test", "template": "static-site"}
      2. Verify response contains template_name
    Expected Result: Project created with template
    Evidence: .sisyphus/evidence/task-5-create-with-template.txt

  Scenario: Invalid template name returns error
    Tool: Bash (curl)
    Preconditions: Server running
    Steps:
      1. POST /api/projects with {"name": "test", "template": "nonexistent"}
      2. Verify 400 error with message
    Expected Result: Clear error about unknown template
    Evidence: .sisyphus/evidence/task-5-invalid-template.txt
  ```

  **Commit**: YES
  - Message: `feat(api): wire template registry into project creation`
  - Files: `crates/cuttlefish-api/src/api_routes.rs`, `crates/cuttlefish-api/src/routes.rs`

- [ ] 6. GitHub Template Fetching

  **What to do**:
  - Create `crates/cuttlefish-core/src/template_fetcher.rs`
  - Implement `TemplateFetcher` struct:
    ```rust
    pub struct TemplateFetcher {
        client: reqwest::Client,
        cache_dir: PathBuf,
        github_token: Option<String>,  // From GITHUB_TOKEN env
    }
    ```
  - Implement `fetch(url: &str) -> Result<LoadedTemplate, TemplateError>`:
    - Parse GitHub URL (github.com/owner/repo or raw URL)
    - Use GitHub API to download file content
    - Parse frontmatter
    - Cache locally in `~/.cache/cuttlefish/templates/`
  - Implement `fetch_registry(url: &str) -> Result<Vec<TemplateRef>, TemplateError>`:
    - Download index.json from remote registry
    - Return list of available templates
  - Handle rate limiting (429) with retry
  - Export from lib.rs

  **Must NOT do**:
  - Don't require GitHub token (use anonymous with rate limits)
  - Don't fetch entire repos (just single files)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: HTTP client, caching, error handling
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 7, 8)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 7
  - **Blocked By**: Task 4

  **References**:
  - `reqwest` crate docs: https://docs.rs/reqwest
  - GitHub raw content: `https://raw.githubusercontent.com/{owner}/{repo}/{branch}/{path}`

  **Acceptance Criteria**:
  - [ ] Can fetch template from GitHub URL
  - [ ] Template cached locally after fetch
  - [ ] Rate limiting handled gracefully
  - [ ] Works without token (anonymous)

  **QA Scenarios**:
  ```
  Scenario: Fetch template from GitHub
    Tool: Bash (cargo test)
    Preconditions: Fetcher module exists, network available
    Steps:
      1. Run test that fetches from real GitHub URL (public repo)
      2. Verify template parsed correctly
    Expected Result: Template loaded with manifest
    Evidence: .sisyphus/evidence/task-6-github-fetch.txt

  Scenario: Cache prevents duplicate fetches
    Tool: Bash (cargo test)
    Preconditions: Template already cached
    Steps:
      1. Fetch same URL twice
      2. Verify only one network request made (use mock or timing)
    Expected Result: Second fetch uses cache
    Evidence: .sisyphus/evidence/task-6-cache-hit.txt
  ```

  **Commit**: YES
  - Message: `feat(core): add GitHub template fetching with caching`
  - Files: `crates/cuttlefish-core/src/template_fetcher.rs`, `crates/cuttlefish-core/Cargo.toml`

- [ ] 7. REST API Endpoints for Templates

  **What to do**:
  - Add to `crates/cuttlefish-api/src/api_routes.rs`:
    - `GET /api/templates` — List all templates (local + cached remote)
    - `GET /api/templates/:name` — Get single template details
    - `POST /api/templates/fetch` — Fetch template from URL
    - `DELETE /api/templates/:name` — Remove cached template
  - Response types:
    ```rust
    pub struct TemplateListResponse {
        pub templates: Vec<TemplateSummary>,
    }
    
    pub struct TemplateSummary {
        pub name: String,
        pub description: String,
        pub language: String,
        pub source: String,  // "local" or "remote"
        pub tags: Vec<String>,
    }
    ```
  - Add routes to `crates/cuttlefish-api/src/routes.rs`
  - Write API tests

  **Must NOT do**:
  - Don't expose template content in list endpoint (fetch separately)
  - Don't allow arbitrary URL fetching (validate GitHub URLs only)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
    - Reason: Multiple REST endpoints with validation
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 6, 8)
  - **Parallel Group**: Wave 3
  - **Blocks**: Task 9
  - **Blocked By**: Tasks 3, 4, 5, 6

  **References**:
  - `crates/cuttlefish-api/src/api_routes.rs` — Existing API patterns
  - `crates/cuttlefish-api/src/routes.rs` — Route registration

  **Acceptance Criteria**:
  - [ ] GET /api/templates returns list
  - [ ] GET /api/templates/:name returns details
  - [ ] POST /api/templates/fetch downloads from URL
  - [ ] All endpoints have tests

  **QA Scenarios**:
  ```
  Scenario: List templates via API
    Tool: Bash (curl)
    Preconditions: Server running with templates loaded
    Steps:
      1. curl http://localhost:8080/api/templates
      2. Verify JSON array with 5+ items
    Expected Result: Templates listed with metadata
    Evidence: .sisyphus/evidence/task-7-list-api.txt

  Scenario: Fetch remote template via API
    Tool: Bash (curl)
    Preconditions: Server running
    Steps:
      1. POST /api/templates/fetch with {"url": "https://github.com/..."}
      2. Verify 200 response with template details
      3. GET /api/templates to verify it's now listed
    Expected Result: Template fetched and available
    Evidence: .sisyphus/evidence/task-7-fetch-api.txt
  ```

  **Commit**: YES
  - Message: `feat(api): add template REST endpoints`
  - Files: `crates/cuttlefish-api/src/api_routes.rs`, `crates/cuttlefish-api/src/routes.rs`

- [ ] 8. Template Validation CLI Command

  **What to do**:
  - Add subcommand to main binary: `cuttlefish template validate <path>`
  - Implement in `src/main.rs` or new `src/commands/template.rs`:
    - Load file from path
    - Parse frontmatter
    - Validate all required fields present
    - Check variable syntax in content
    - Report errors with line numbers if possible
  - Also add: `cuttlefish template list` to show local templates
  - Also add: `cuttlefish template render <name> --var key=value` for testing

  **Must NOT do**:
  - Don't add heavy CLI framework (use clap subcommands)

  **Recommended Agent Profile**:
  - **Category**: `quick`
    - Reason: CLI wiring, straightforward
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES (with Tasks 6, 7)
  - **Parallel Group**: Wave 3
  - **Blocks**: None
  - **Blocked By**: None (can use existing modules)

  **References**:
  - `src/main.rs` — Existing CLI structure
  - clap subcommands: https://docs.rs/clap/latest/clap/

  **Acceptance Criteria**:
  - [ ] `cuttlefish template validate templates/nuxt-cloudflare.md` passes
  - [ ] Invalid template shows clear error
  - [ ] `cuttlefish template list` shows all templates

  **QA Scenarios**:
  ```
  Scenario: Validate valid template
    Tool: Bash
    Preconditions: Binary built, template files exist
    Steps:
      1. Run `./target/release/cuttlefish template validate templates/nuxt-cloudflare.md`
      2. Verify exit code 0
    Expected Result: Validation passes
    Evidence: .sisyphus/evidence/task-8-validate-pass.txt

  Scenario: Validate invalid template shows error
    Tool: Bash
    Preconditions: Binary built
    Steps:
      1. Create malformed template file
      2. Run `./target/release/cuttlefish template validate bad.md`
      3. Verify exit code 1 with error message
    Expected Result: Clear error about what's wrong
    Evidence: .sisyphus/evidence/task-8-validate-fail.txt
  ```

  **Commit**: YES
  - Message: `feat(cli): add template validation commands`
  - Files: `src/main.rs` or `src/commands/template.rs`

- [ ] 9. WebUI Template Gallery Component

  **What to do**:
  - Create `cuttlefish-web/pages/templates.vue`:
    - Fetch templates from API on mount
    - Display grid of template cards
    - Each card shows: name, description, language badge, tags
    - Click card to see full details in modal/sidebar
    - "Use Template" button that pre-fills project creation
  - Create `cuttlefish-web/components/TemplateCard.vue`:
    - Reusable card component
    - Language color coding (TypeScript=blue, Python=yellow, Rust=orange)
  - Add "Templates" link to navigation
  - Style with terminal aesthetic (dark cards, cyan accents)

  **Must NOT do**:
  - Don't add new npm dependencies
  - Don't implement template editing (read-only gallery)

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
    - Reason: UI component with visual design
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: NO
  - **Parallel Group**: Wave 4 (after API ready)
  - **Blocks**: F1-F4
  - **Blocked By**: Task 7

  **References**:
  - `cuttlefish-web/pages/index.vue` — Existing page patterns
  - `cuttlefish-web/pages/project/[id].vue` — Component patterns

  **Acceptance Criteria**:
  - [ ] /templates page shows template grid
  - [ ] Cards display correct metadata
  - [ ] "Use Template" navigates to project creation
  - [ ] Mobile responsive

  **QA Scenarios**:
  ```
  Scenario: Template gallery renders
    Tool: Playwright
    Preconditions: Dev server running with API
    Steps:
      1. Navigate to /templates
      2. Wait for API response
      3. Verify 5+ template cards visible
    Expected Result: Gallery displays templates
    Evidence: .sisyphus/evidence/task-9-gallery.png

  Scenario: Use Template flow
    Tool: Playwright
    Preconditions: On /templates page
    Steps:
      1. Click first template card
      2. Click "Use Template" button
      3. Verify navigation to project creation with template pre-selected
    Expected Result: Template selection preserved
    Evidence: .sisyphus/evidence/task-9-use-template.png
  ```

  **Commit**: YES
  - Message: `feat(web): add template gallery page`
  - Files: `cuttlefish-web/pages/templates.vue`, `cuttlefish-web/components/TemplateCard.vue`

---

## Final Verification Wave

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read the plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy --workspace -- -D warnings` + `cargo test --workspace`. Review all changed files for: `as any`/`@ts-ignore` (if TS), empty catches, console.log in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT`

- [ ] F3. **Integration QA** — `unspecified-high`
  Start from clean state. Create a project using each template. Verify variable substitution works. Test API endpoints with curl. Test GitHub template fetching. Save evidence to `.sisyphus/evidence/final-qa/`.
  Output: `Templates [N/N pass] | API [N/N] | Substitution [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read "What to do", read actual diff. Verify 1:1 match. Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT`

---

## Commit Strategy

| Wave | Commit |
|------|--------|
| 1 | `feat(core): add template manifest parser and Tera engine` |
| 2 | `feat(core): add starter templates and local registry` |
| 3 | `feat(api): add template REST endpoints and GitHub fetching` |
| 4 | `feat(webui): add template gallery page` |

---

## Success Criteria

### Verification Commands
```bash
cargo test --workspace  # All tests pass
cargo clippy --workspace -- -D warnings  # Clean
ls templates/*.md | wc -l  # Expected: >= 5
curl http://localhost:8080/api/templates  # Returns JSON array
```

### Final Checklist
- [ ] 5+ template files with valid YAML frontmatter
- [ ] Variable substitution works in project creation
- [ ] GitHub template fetching works
- [ ] API returns template list
- [ ] No unsafe code added
- [ ] All clippy warnings resolved
