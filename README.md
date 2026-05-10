# quick-node

Rust CLI tool for managing Shadowsocks 2022 proxy nodes on Linux VPS.

One command to init, one command to create a user — get share links instantly.

## Features

- **Shadowsocks 2022** (shadowsocks-rust) — `2022-blake3-aes-256-gcm`, modern and fast
- **Multi-user** — single port, per-user identity PSK
- **Two link formats** per user: SS URL, Clash subscription
- **Expiry control** — auto-disable users via systemd timer
- **Built-in HTTP server** — serves Clash YAML subscriptions
- **Single binary** — no Python, no Node.js, no runtime dependencies

## Quick Start

```bash
# Install on a fresh VPS (as root)
curl -sSL https://raw.githubusercontent.com/123hi123/quick-node/master/install.sh | bash

# Initialize node
quick-node init --port 8388

# Create a user (outputs share links)
quick-node user add joe --expires 30d --traffic-limit 100GB
```

Output:

```
=== joe ===

SS:
  ss://MjAyMi1ibGFrZTMtYWVz...@1.2.3.4:8388#joe

Clash Subscribe:
  http://1.2.3.4:8443/sub/token
```

## Commands

| Command | Description |
|---------|-------------|
| `quick-node init` | Download ssserver, generate keys, setup systemd |
| `quick-node user add <name>` | Create user, output share links |
| `quick-node user list` | List all users with status |
| `quick-node user remove <name>` | Remove a user |
| `quick-node refresh` | Re-detect public IP, update all links |
| `quick-node status` | Show server status |
| `quick-node serve` | Start HTTP subscription server |
| `quick-node check` | Check expiry (called by timer) |

### Init Options

```
quick-node init [OPTIONS]
  -p, --port <PORT>              SS port [default: 8388]
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
├── ssserver (Shadowsocks 2022, single-port multi-user)
│   └── port 8388 → 2022-blake3-aes-256-gcm
│
├── HTTP subscription server
│   └── port 8443 → GET /sub/{token} → Clash YAML
│
└── systemd timer (every 10min)
    └── quick-node check → enforce expiry
```

### Files on VPS

```
/usr/local/bin/quick-node          # CLI binary
/usr/local/bin/ssserver            # shadowsocks-rust server
/etc/quick-node/
├── config.json                    # Node config (IP, key, ports)
├── users.json                     # User state (expiry, traffic)
├── ss-config.json                 # Generated ssserver config
└── subs/*.yaml                    # Per-user Clash subscription files
```

## Client Compatibility

SS 2022 (`2022-blake3-aes-256-gcm`) requires clients that support the protocol:

- **Clash Meta / mihomo** — full support
- **Shadowrocket** (iOS) — full support
- **sing-box** — full support
- **Surge** (macOS/iOS) — full support
- **sslocal** (shadowsocks-rust) — full support

Note: Original Clash (no longer maintained) does NOT support SS 2022.

## Build from Source

```bash
cargo build --release

# Static binary (no glibc dependency)
cargo build --release --target x86_64-unknown-linux-musl
```

## License

MIT
