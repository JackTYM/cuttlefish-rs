# REST API Specification

## Base URL

All API endpoints are relative to: `http://<host>:<port>`

## Authentication

Most endpoints require authentication via:
- **API Key Header**: `Authorization: Bearer <api_key>`
- **API Key Query**: `?key=<api_key>`

## Endpoints

### Health Check

#### GET /health
Health check endpoint (no auth required).

**Response**:
```json
{
  "status": "ok",
  "version": "0.2.0"
}
```

---

### Templates

#### GET /api/templates
List all available project templates.

**Response**:
```json
[
  {
    "name": "rust-cli",
    "description": "Rust CLI application template",
    "language": "rust"
  }
]
```

#### GET /api/templates/{name}
Get details for a specific template.

**Response**:
```json
{
  "name": "rust-cli",
  "description": "Rust CLI application template",
  "language": "rust",
  "files": ["Cargo.toml", "src/main.rs"]
}
```

#### POST /api/templates/fetch
Fetch a template from a remote URL.

**Request**:
```json
{
  "url": "https://github.com/user/template",
  "name": "my-template"
}
```

---

### Projects

#### GET /api/projects
List all projects.

**Response**:
```json
[
  {
    "id": "uuid-string",
    "name": "my-project",
    "status": "active",
    "created_at": "2024-01-15T10:00:00Z"
  }
]
```

#### POST /api/projects
Create a new project.

**Request**:
```json
{
  "name": "my-project",
  "template": "rust-cli",
  "description": "My awesome project"
}
```

**Response**:
```json
{
  "id": "uuid-string",
  "name": "my-project",
  "status": "active",
  "created_at": "2024-01-15T10:00:00Z"
}
```

#### GET /api/projects/{id}
Get project details.

#### DELETE /api/projects/{id}
Cancel/delete a project.

#### POST /api/projects/{id}/archive
Archive a completed project.

---

### System

#### GET /api/system/config
Get current system configuration.

**Response**:
```json
{
  "server": {
    "host": "127.0.0.1",
    "port": 8080
  },
  "providers": ["anthropic", "openai"],
  "agents": {
    "orchestrator": { "category": "deep" },
    "coder": { "category": "deep" }
  }
}
```

#### PUT /api/system/config
Update system configuration.

#### GET /api/system/status
Get system status including connected clients, active projects, etc.

#### POST /api/system/providers/{id}/test
Test a provider connection.

**Response**:
```json
{
  "connected": true
}
```

#### POST /api/system/api-key/regenerate
Regenerate the API key.

---

### Sandbox

#### POST /api/sandbox
Create a new sandbox container.

**Request**:
```json
{
  "project_id": "uuid-string",
  "language": "rust",
  "preset": "standard"
}
```

**Presets**:
- `light`: 512MB RAM, 1.0 CPU
- `standard`: 2048MB RAM, 2.0 CPU
- `heavy`: 4096MB RAM, 4.0 CPU

**Response**:
```json
{
  "id": "sandbox-uuid",
  "status": "created",
  "project_id": "uuid-string"
}
```

#### GET /api/sandbox
List all sandboxes.

#### GET /api/sandbox/{id}
Get sandbox status.

#### DELETE /api/sandbox/{id}
Remove a sandbox.

#### POST /api/sandbox/{id}/exec
Execute a command in the sandbox.

**Request**:
```json
{
  "command": "cargo build",
  "args": ["--release"],
  "working_dir": "/workspace"
}
```

**Response**:
```json
{
  "stdout": "Compiling...",
  "stderr": "",
  "exit_code": 0
}
```

#### POST /api/sandbox/{id}/snapshot
Create a snapshot of the sandbox state.

#### GET /api/sandbox/health
Check Docker availability.

---

### Safety

#### GET /api/safety/pending
List pending approvals.

#### POST /api/safety/approve/{action_id}
Approve a pending action.

#### POST /api/safety/reject/{action_id}
Reject a pending action.

**Request**:
```json
{
  "reason": "Too risky"
}
```

#### GET /api/safety/config
Get safety gate configuration.

#### PUT /api/safety/config
Update safety gate configuration.

**Request**:
```json
{
  "auto_approve_threshold": 0.9,
  "prompt_threshold": 0.5
}
```

---

### Usage

#### GET /api/usage
Get usage statistics.

**Query Parameters**:
- `project_id`: Filter by project
- `period`: `daily`, `weekly`, `monthly`

#### GET /api/usage/costs
Get cost breakdown.

---

### Memory

#### GET /api/memory/{project_id}
Get project memory summary.

#### GET /api/memory/{project_id}/decisions
Get decision log for a project.

---

### Authentication

#### POST /api/auth/register
Register a new user (if enabled).

#### POST /api/auth/login
Login and get JWT token.

#### POST /api/auth/refresh
Refresh JWT token.

---

## Error Responses

All errors follow this format:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE"
}
```

**HTTP Status Codes**:
- `400` Bad Request - Invalid input
- `401` Unauthorized - Missing or invalid auth
- `403` Forbidden - Insufficient permissions
- `404` Not Found - Resource doesn't exist
- `500` Internal Server Error - Server-side error
- `503` Service Unavailable - Service temporarily unavailable (e.g., Docker not running)
