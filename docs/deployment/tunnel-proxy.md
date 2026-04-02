# Tunnel Proxy Deployment Guide

This guide explains how to deploy the Cuttlefish tunnel system with Caddy as the reverse proxy.

## Architecture

```
User's Browser                    cuttlefish.ai (Caddy)              Self-Hosted Server
     │                                   │                                  │
     │ GET alice.cuttlefish.ai           │                                  │
     │──────────────────────────────────▶│                                  │
     │                                   │                                  │
     │                           [Lookup: alice → tunnel]                   │
     │                                   │                                  │
     │                                   │  Forward via WebSocket           │
     │                                   │─────────────────────────────────▶│
     │                                   │                                  │
     │                                   │◀─────────────────────────────────│
     │                                   │  Response                        │
     │◀──────────────────────────────────│                                  │
```

## Prerequisites

### 1. DNS Configuration

Create a wildcard DNS record pointing to your server:

```
*.cuttlefish.ai    A    203.0.113.10
cuttlefish.ai      A    203.0.113.10
```

### 2. Install Caddy

```bash
# Ubuntu/Debian
sudo apt install -y debian-keyring debian-archive-keyring apt-transport-https
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/gpg.key' | sudo gpg --dearmor -o /usr/share/keyrings/caddy-stable-archive-keyring.gpg
curl -1sLf 'https://dl.cloudsmith.io/public/caddy/stable/debian.deb.txt' | sudo tee /etc/apt/sources.list.d/caddy-stable.list
sudo apt update
sudo apt install caddy
```

### 3. Configure Environment

Set the JWT secret for the tunnel daemon:

```bash
export TUNNEL_JWT_SECRET="your-secure-random-secret-at-least-32-chars"
```

## Deployment Steps

### 1. Copy Caddyfile

```bash
sudo cp deploy/caddy/Caddyfile /etc/caddy/Caddyfile
```

### 2. Start Tunnel Daemon

```bash
# Build the daemon
cargo build --release --bin tunnel-daemon

# Run (or use systemd service)
TUNNEL_JWT_SECRET="$TUNNEL_JWT_SECRET" ./target/release/tunnel-daemon
```

### 3. Reload Caddy

```bash
sudo systemctl reload caddy
```

## Port Summary

| Port | Service | Purpose |
|------|---------|---------|
| 8080 | Main Cuttlefish | API and WebUI |
| 8081 | Tunnel Daemon HTTP | Routes HTTP requests to tunnels |
| 8082 | Tunnel Daemon WS | Client WebSocket connections |
| 443 | Caddy | HTTPS (auto TLS) |
| 80 | Caddy | HTTP (redirect to HTTPS) |

## TLS Certificates

Caddy automatically obtains TLS certificates via Let's Encrypt. For wildcard certificates, you may need to configure DNS challenge:

```
# In Caddyfile, add TLS directive if using DNS challenge
*.cuttlefish.ai {
    tls {
        dns cloudflare {env.CLOUDFLARE_API_TOKEN}
    }
    ...
}
```

## Troubleshooting

### Tunnel Not Connecting

1. Check if tunnel daemon is running:
   ```bash
   curl http://localhost:8081/health
   ```

2. Check Caddy logs:
   ```bash
   sudo journalctl -u caddy -f
   ```

3. Verify DNS resolution:
   ```bash
   dig alice.cuttlefish.ai
   ```

### WebSocket Errors

Ensure Caddy is forwarding upgrade headers correctly. Check that the tunnel daemon accepts WebSocket connections on port 8082.

## Security Considerations

1. **JWT Secret**: Use a strong, random secret (at least 32 characters)
2. **Firewall**: Only expose ports 80 and 443 publicly
3. **Rate Limiting**: Consider adding rate limiting in Caddy for abuse prevention
