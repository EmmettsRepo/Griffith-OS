# Griffith OS (GOS)

A cross-platform **OS extension** — a native layer that installs onto macOS, Linux, or Windows,
gives you everything your current OS already does **plus** a strong privacy/anonymity stack, secure
GOS-to-GOS data transfer, a bundled developer toolchain, and its own expandable app environment.

GOS is **not** a bare-metal operating system and **not** a web app. It's a native Tauri application
(Rust core + web UI) that augments the host OS. The host keeps running underneath; GOS adds
capabilities on top and integrates with the host through bridges you can switch off at any time.

> **Honest scope:** GOS aims for strong, Tails-grade privacy by default. It cannot make you literally
> invisible. See [docs/THREAT_MODEL.md](docs/THREAT_MODEL.md). This is legitimate privacy software,
> like Tor and Tails — not a tool for evading law enforcement.

## Architecture

| Crate | Role |
|-------|------|
| `gos-core` | Shared types, settings store, node identity keypair |
| `gos-privacy` | Tor (arti) · DNS-over-HTTPS (hickory) · leak tests · privacy engine |
| `gos-platform` | `PlatformOps` trait + per-OS impls (system proxy, firewall kill-switch, MAC randomize) |
| `gos-bridge` | Host cross-over (files / launch apps / clipboard / devices) — each toggleable |
| `src-tauri` | Tauri app: commands, tray, event stream |
| `ui/` | React + TS + Tailwind control panel |

## Build status: v1 — Privacy / Anonymity Core (in progress)

## Develop

```bash
# one-time: rust + node are required (node 20+, rust stable)
cd ui && npm install        # UI deps + Tauri CLI
npm run tauri dev           # run the app (from ui/)
```

## Roadmap

1. **v1 Privacy core** ← current
2. Host bridge (toggleable cross-over)
3. Encrypted GOS↔GOS transfer (iroh)
4. Dev toolchain provisioner (mise + Homebrew)
5. Full-screen desktop environment
6. Privacy browser
7. Plugin / app SDK + store

See the full plan in `~/.claude/plans/`.
