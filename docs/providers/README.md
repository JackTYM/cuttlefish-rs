# Model Provider Setup

Cuttlefish supports 11 model providers. Choose the ones that fit your needs.

## Quick Setup

Set the environment variables for your chosen providers, then configure them in `cuttlefish.toml`.

## Providers

| Provider | Guide | API Key Env Var |
|----------|-------|-----------------|
| Anthropic | [anthropic.md](anthropic.md) | `ANTHROPIC_API_KEY` |
| OpenAI | [openai.md](openai.md) | `OPENAI_API_KEY` |
| Google Gemini | [google.md](google.md) | `GOOGLE_API_KEY` |
| Moonshot (Kimi) | [moonshot.md](moonshot.md) | `MOONSHOT_API_KEY` |
| Zhipu (GLM) | [zhipu.md](zhipu.md) | `ZHIPU_API_KEY` |
| MiniMax | [minimax.md](minimax.md) | `MINIMAX_API_KEY` |
| xAI (Grok) | [xai.md](xai.md) | `XAI_API_KEY` |
| AWS Bedrock | [bedrock.md](bedrock.md) | AWS credentials |
| Ollama (local) | [ollama.md](ollama.md) | None |
| Claude OAuth | [oauth.md](oauth.md) | None (PKCE flow) |
| ChatGPT OAuth | [oauth.md](oauth.md) | Bearer token |
