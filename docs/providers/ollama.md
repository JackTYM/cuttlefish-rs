# Ollama Local Setup

Run models locally with no API key required.

## Prerequisites
- Linux/macOS system
- 8GB+ RAM (16GB recommended for larger models)

## Installation
```bash
curl -fsSL https://ollama.ai/install.sh | sh
```

## Pull a Model
```bash
ollama pull llama3.1        # 8B model, ~5GB
ollama pull codellama       # Code-focused model
ollama pull mixtral         # 8x7B mixture of experts
```

## Configuration
No API key needed. In `cuttlefish.toml`:
```toml
[providers.ollama]
provider_type = "ollama"
model = "llama3.1"
```

For a remote Ollama server:
```toml
[providers.ollama]
provider_type = "ollama"
model = "llama3.1"
base_url = "http://your-server:11434"
```

## Troubleshooting
- **Connection refused**: Make sure Ollama is running (`ollama serve`)
- **Model not found**: Pull the model first (`ollama pull <model>`)
