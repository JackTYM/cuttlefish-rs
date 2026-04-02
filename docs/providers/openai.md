# OpenAI API Setup

## Prerequisites
- OpenAI account at [platform.openai.com](https://platform.openai.com)

## Getting Your API Key
1. Log in to [platform.openai.com](https://platform.openai.com)
2. Go to **API Keys**
3. Click **Create new secret key**
4. Copy the key (starts with `sk-`)

## Configuration
```bash
export OPENAI_API_KEY="sk-your-key-here"
```

In `cuttlefish.toml`:
```toml
[providers.openai]
provider_type = "openai"
model = "gpt-5.4"
```

## Available Models
- `gpt-5.4` — Most capable GPT model
- `gpt-5-nano` — Fast and efficient
- `gpt-4o` — Multimodal, strong reasoning

## Troubleshooting
- **401 Unauthorized**: Invalid API key
- **429 Rate Limited**: Exceeded quota; check your usage limits
