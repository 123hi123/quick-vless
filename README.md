# quick-node

Rust CLI tool for managing Hysteria2 proxy nodes (via sing-box) on Linux VPS.

One command to init, one command to create a user — get share links instantly.

## Features

- **Hysteria2** (sing-box) — QUIC-based, self-signed TLS, UDP transport
- **Multi-user** — password-based authentication per user
- **SOCKS5 proxy** — built-in SOCKS5 inbound on the same server
- **Three link formats** per user: Hysteria2 URL, SOCKS5 URL, Clash subscription
- **Expiry & traffic control** — auto-disable users via systemd timer
- **Built-in HTTP server** — serves Clash YAML subscriptions
- **Single binary** — no Python, no Node.js, no runtime dependencies

## Quick Start

```bash
# Install on a fresh VPS (as root)
curl -sSL https://raw.githubusercontent.com/123hi123/quick-node/master/install.sh | bash

# Initialize node
quick-node init --port 443

# Create a user (outputs share links)
quick-node user add joe --expires 30d --traffic-limit 100GB
```

Output:

```
=== joe ===

Hysteria2:
  hysteria2://password@1.2.3.4:443/?insecure=1#joe

SOCKS5:
  socks5://proxy:pass@1.2.3.4:1080

Clash Subscribe:
  http://1.2.3.4:8443/sub/token
```

## Commands

| Command | Description |
|---------|-------------|
| `quick-node init` | Download sing-box, generate TLS cert, setup systemd |
| `quick-node user add <name>` | Create user, output share links |
| `quick-node user list` | List all users with status |
| `quick-node user remove <name>` | Remove a user |
| `quick-node refresh` | Re-detect public IP, update all links |
| `quick-node status` | Show server status |
| `quick-node serve` | Start HTTP subscription server |
| `quick-node check` | Check expiry & traffic (called by timer) |

### Init Options

```
quick-node init [OPTIONS]
  -p, --port <PORT>              Hysteria2 listen port, UDP [default: 443]
      --socks-port <SOCKS_PORT>  SOCKS5 listen port [default: 1080]
      --sub-port <SUB_PORT>      Subscription HTTP port [default: 8443]
      --ip <IP>                  Server IP (auto-detected if omitted)
```

### User Add Options

```
quick-node user add <NAME> [OPTIONS]
  -e, --expires <EXPIRES>              Duration: 30d, 6h, 1w, 0=never [default: 30d]
  -t, --traffic-limit <TRAFFIC_LIMIT>  Limit: 100GB, 500MB, 0=unlimited [default: 0]
```

## Architecture

```
quick-node (single Rust binary)
│
├── sing-box (Hysteria2, QUIC/UDP, multi-user password auth)
│   ├── Hysteria2 inbound → port 443 (UDP), self-signed TLS
│   └── SOCKS5 inbound → port 1080
│
├── HTTP subscription server
│   └── port 8443 → GET /sub/{token} → Clash YAML
│
└── systemd timer (every 10min)
    └── quick-node check → enforce expiry & traffic limits
```

### Files on VPS

```
/usr/local/bin/quick-node              # CLI binary
/usr/local/bin/sing-box                # sing-box server
/etc/quick-node/
├── config.json                        # Node config (IP, ports, SOCKS5 pass)
├── users.json                         # User state (expiry, traffic)
├── singbox-config.json                # Generated sing-box config
├── cert.pem                           # Self-signed TLS certificate
├── key.pem                            # TLS private key
└── subs/*.yaml                        # Per-user Clash subscription files
```

## Client Compatibility

Hysteria2 requires clients that support the protocol:

- **Clash Meta / mihomo** — full support
- **Shadowrocket** (iOS) — full support
- **sing-box** — full support
- **Surge** (macOS/iOS) — full support
- **NekoBox** (Android) — full support

## Build from Source

```bash
cargo build --release

# Static binary (no glibc dependency)
cargo build --release --target x86_64-unknown-linux-musl
```

## License

MIT
