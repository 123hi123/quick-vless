# Quick-Node

Rust CLI for managing Hysteria2 proxy nodes (sing-box).

## Release Checklist (MANDATORY before every push)

**install.sh downloads the binary from GitHub Releases.** If you push code changes without updating the release binary, remote VMs will download the old (or broken) version.

Before pushing:

1. `cargo build --release --target x86_64-unknown-linux-musl`
2. `cp $CARGO_TARGET_DIR/x86_64-unknown-linux-musl/release/quick-node /tmp/quick-node-x86_64-linux`
3. Bump version in `Cargo.toml` if needed
4. `gh release create vX.Y.Z /tmp/quick-node-x86_64-linux --title "vX.Y.Z" --generate-notes`
   - Or update existing release: `gh release upload vX.Y.Z /tmp/quick-node-x86_64-linux --clobber`
5. Then `git push`

**CARGO_TARGET_DIR is set to `~/.cache/cargo-target`** — the binary is NOT in `./target/`.

## Architecture

- `sing-box` — Hysteria2 server (QUIC), multi-user via password auth, self-signed TLS
- `quick-node serve` — HTTP server for Clash YAML subscriptions
- `quick-node check` — systemd timer, enforces user expiry

## Config paths (on deployed VPS)

```
/etc/quick-node/config.json          # AppConfig (IP, hy_port, sub_port)
/etc/quick-node/users.json           # User state
/etc/quick-node/singbox-config.json  # Generated sing-box config
/etc/quick-node/cert.pem             # Self-signed TLS certificate
/etc/quick-node/key.pem              # TLS private key
/etc/quick-node/subs/*.yaml          # Per-user Clash subscription files
```

## Firewall

Hysteria2 uses **UDP** (default port 443). Ensure the GCP/cloud firewall allows UDP ingress on the configured port.
