# Anthropic API Setup

## Prerequisites
- Anthropic account at [console.anthropic.com](https://console.anthropic.com)

## Getting Your API Key
1. Log in to [console.anthropic.com](https://console.anthropic.com)
2. Go to **API Keys** in the sidebar
3. Click **Create Key**
4. Copy the key (starts with `sk-ant-`)

## Configuration
```bash
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

In `cuttlefish.toml`:
```toml
[providers.anthropic]
provider_type = "anthropic"
model = "claude-sonnet-4-6"
```

## Available Models
- `claude-opus-4-6` — Most capable, best for complex reasoning
- `claude-sonnet-4-6` — Balanced performance and speed
- `claude-haiku-4-5` — Fastest, best for simple tasks

## Troubleshooting
- **401 Unauthorized**: Check your API key is correct and not expired
- **429 Rate Limited**: You've exceeded your rate limit; wait and retry
