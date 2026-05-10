# Quick-Node

Rust CLI for managing Shadowsocks 2022 proxy nodes (ssserver + sslocal).

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

- `ssserver` — Shadowsocks server, 2022-blake3-aes-256-gcm, single-port multi-user
- `sslocal` — connects to local ssserver via internal `__socks` user, exposes SOCKS5 on port 1080
- `quick-node serve` — HTTP server for Clash YAML subscriptions
- `quick-node check` — systemd timer, enforces user expiry

## Config paths (on deployed VPS)

```
/etc/quick-node/config.json        # AppConfig (IP, keys, ports)
/etc/quick-node/users.json         # User state
/etc/quick-node/ss-config.json     # Generated ssserver config
/etc/quick-node/sslocal-config.json # Generated sslocal config
/etc/quick-node/subs/*.yaml        # Per-user Clash subscription files
```
