# OAuth Provider Setup

## Claude OAuth (claude.ai account)

Use your Claude.ai subscription without an API key.

### Setup
Run the OAuth flow:
```bash
cuttlefish auth claude
```
This opens your browser for authentication. Tokens are stored in `~/.cuttlefish/tokens/`.

### Configuration
```toml
[providers.claude-oauth]
provider_type = "claude-oauth"
model = "claude-sonnet-4-6"
```

## ChatGPT OAuth (ChatGPT Plus/Pro)

Use your ChatGPT subscription.

### Setup
Obtain a bearer token from your ChatGPT session, then:
```bash
export CHATGPT_ACCESS_TOKEN="your-bearer-token"
```

### Configuration
```toml
[providers.chatgpt-oauth]
provider_type = "chatgpt-oauth"
model = "gpt-4o"
```
