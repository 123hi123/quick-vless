# quick-node

Rust CLI tool for managing VLESS + Reality proxy nodes on Linux VPS.

One command to init, one command to create a user — get three share links instantly.

## Features

- **VLESS + Reality** (Xray-core) — strongest anti-detection, no domain/cert needed
- **Three link formats** per user: VLESS URL, Clash subscription, SOCKS5
- **Traffic & expiry control** — auto-disable users via systemd timer
- **Built-in HTTP server** — serves Clash YAML subscriptions
- **Single binary** — no Python, no Node.js, no runtime dependencies

## Quick Start

```bash
# Install on a fresh VPS (as root)
curl -sSL https://raw.githubusercontent.com/123hi123/quick-node/master/install.sh | bash

# Initialize node
quick-node init --port 443 --sni www.microsoft.com

# Create a user (outputs 3 links)
quick-node user add joe --expires 30d --traffic-limit 100GB
```

Output:

```
=== joe ===

VLESS:
  vless://uuid@1.2.3.4:443?encryption=none&flow=xtls-rprx-vision&security=reality&sni=www.microsoft.com&fp=chrome&pbk=...&sid=...&type=tcp#joe

Clash Subscribe:
  http://1.2.3.4:8443/sub/token

SOCKS5:
  socks5://joe:password@1.2.3.4:1080
```

## Commands

| Command | Description |
|---------|-------------|
| `quick-node init` | Download Xray-core, generate Reality keys, setup systemd |
| `quick-node user add <name>` | Create user, output 3 share links |
| `quick-node user list` | List all users with traffic stats |
| `quick-node user remove <name>` | Remove a user |
| `quick-node refresh` | Re-detect public IP, update all links |
| `quick-node status` | Show server status |
| `quick-node serve` | Start HTTP subscription server |
| `quick-node check` | Check traffic/expiry (called by timer) |

### Init Options

```
quick-node init [OPTIONS]
  -p, --port <PORT>              VLESS port [default: 443]
  -s, --sni <SNI>                Reality SNI target [default: www.microsoft.com]
      --socks-port <SOCKS_PORT>  SOCKS5 port [default: 1080]
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
├── Xray-core (VLESS + Reality + SOCKS5 inbounds)
│   ├── port 443  → VLESS + Reality (TCP, Vision flow)
│   └── port 1080 → SOCKS5 (shared, user/pass auth)
│
├── HTTP subscription server
│   └── port 8443 → GET /sub/{token} → Clash YAML
│
└── systemd timer (every 10min)
    └── quick-node check → enforce traffic limits & expiry
```

### Files on VPS

```
/usr/local/bin/quick-node          # CLI binary
/usr/local/bin/xray                 # Xray-core
/etc/quick-node/
├── config.json                     # Node config (IP, keys, ports)
├── users.json                      # User state (traffic, expiry)
├── xray-config.json                # Generated Xray config
└── subs/*.yaml                     # Per-user Clash subscription files
```

## Build from Source

```bash
cargo build --release

# Static binary (no glibc dependency)
cargo build --release --target x86_64-unknown-linux-musl
```

## License

MIT
