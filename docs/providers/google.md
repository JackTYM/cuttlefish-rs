# Google Gemini API Setup

## Prerequisites
- Google account
- Access to [Google AI Studio](https://aistudio.google.com)

## Getting Your API Key
1. Go to [aistudio.google.com](https://aistudio.google.com)
2. Click **Get API key**
3. Create a new API key or use an existing project

## Configuration
```bash
export GOOGLE_API_KEY="your-key-here"
```

In `cuttlefish.toml`:
```toml
[providers.google]
provider_type = "google"
model = "gemini-2.0-flash"
```

## Available Models
- `gemini-2.0-flash` — Fast, efficient, great for visual tasks
- `gemini-1.5-pro` — More capable, larger context window

## Troubleshooting
- **403 Forbidden**: API key doesn't have Gemini API access enabled
- **429 Rate Limited**: Free tier has lower limits; consider upgrading
