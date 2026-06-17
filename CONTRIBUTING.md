# Contributing to Griffith OS

Thanks for working on Griffith OS (GOS). This guide covers the prerequisites, local
setup, how to run the engine and the app, code style, and commit conventions.

Before you start, skim [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) for the crate
layout and command flow, and [`docs/THREAT_MODEL.md`](./docs/THREAT_MODEL.md) for the
honest privacy scope. These shape almost every code-review decision.

---

## Prerequisites

| Tool        | Version                | Notes                                                |
|-------------|------------------------|------------------------------------------------------|
| Rust        | stable (via `rustup`)  | Toolchain is pinned in `rust-toolchain.toml`         |
| Node.js     | 20+                    | For the Vite/React UI and the Tauri CLI              |
| npm         | bundled with Node 20+  | Used to install UI dependencies                      |
| `just`      | optional               | Convenience recipes — see `justfile` (not required)  |

Install Rust with [rustup](https://rustup.rs/); it will respect the pinned
toolchain. Tauri may also require platform build dependencies (Xcode CLT on macOS,
WebKitGTK/`libsoup` on Linux, the WebView2 runtime + MSVC build tools on Windows) —
see the [Tauri 2 prerequisites](https://v2.tauri.app/start/prerequisites/).

---

## Setup

```bash
# 1. Clone, then from the repo root:

# 2. Install UI dependencies (this also provides the local Tauri CLI)
cd ui && npm install
cd ..

# 3. Sanity-check the Rust workspace
cargo check
```

The Tauri CLI is installed **locally** under `ui/node_modules/.bin/tauri`. Run all
app commands from the **repo root** using that binary (or add it to your `PATH`).

---

## Running

### The privacy engine (no UI)

The fastest feedback loop. Exercises `PrivacyEngine` end-to-end — bootstraps Tor,
fetches the exit IP, and runs the leak test — without booting the desktop app:

```bash
cargo run -p gos-privacy --example check -- --tor
```

Drop `--tor` to run the Tor-less path. The example source is
`crates/gos-privacy/examples/check.rs`.

### The full app (Tauri dev)

From the **repo root**, using the UI-local Tauri CLI:

```bash
./ui/node_modules/.bin/tauri dev
```

This builds the Rust core, starts the Vite dev server, and launches the tray/menu-bar
app with hot reload on the UI.

### Production build

```bash
./ui/node_modules/.bin/tauri build
```

> If you have [`just`](https://github.com/casey/just) installed, `just dev`,
> `just build`, `just engine-check`, `just check`, and `just test` wrap all of the
> above. `just` is optional.

---

## Code style

These are enforced in review. Keep them in mind as you write, not just at the end.

### General

- **Small files.** Prefer many small, focused modules over a few large ones. Extract
  utilities when a file grows past its single responsibility (~200–400 lines is the
  comfortable range; treat 800 as a hard ceiling).
- **Immutable patterns.** Return new values instead of mutating in place. Avoid
  shared mutable state outside the explicit `AppState`/engine ownership boundaries.
- **Explicit error handling.** Handle errors at each layer; never silently swallow a
  failure. Surface user-facing failures clearly in the UI, log detail in the core.
- **Validate at boundaries.** Treat all external data (network responses, file
  contents, IPC args) as untrusted and validate before use.

### Rust

- Keep privileged/security-sensitive logic in the Rust crates, never in the UI.
- Shared types that cross the IPC boundary live in `gos-core` and serialize as
  **camelCase** so the UI receives idiomatic objects. Do not hand-roll separate DTOs.
- New per-OS behavior goes behind the `PlatformOps` trait with a `#[cfg(target_os)]`
  impl per platform — do not branch on the OS at call sites.

### UI (React / TypeScript)

- All Rust calls go through `ui/lib/api.ts`. Do not call Tauri `invoke` directly
  from components.
- The bespoke "secure console" CSS is hand-written; there is **no Tailwind**. Match
  the existing styling approach.

### Privacy-specific rules (non-negotiable)

- **Every host bridge capability must stay toggleable.** Any new `HostBridge`
  crossover (files, apps, clipboard, devices, or anything new) must be gated by its
  own toggle and respect **Isolation Mode** (all toggles off ⇒ no host crossover).
- **Never over-promise privacy in UI copy.** GOS is Tails-grade privacy, **not
  invisibility**. Do not write copy implying untraceability, anonymity guarantees,
  or "whole-system Tor." Tor is TCP-only and covers the browser/selected apps;
  whole-system tunneling is a future VPN feature. The kill-switch, MAC
  randomization, and system-proxy paths require a privileged helper that is **not
  yet built** — do not present them as active when they are not. When in doubt,
  match the language in [`docs/THREAT_MODEL.md`](./docs/THREAT_MODEL.md).

---

## Commit convention

This project uses [**Conventional Commits**](https://www.conventionalcommits.org/).

```
<type>: <short, imperative description>

<optional body explaining what and why>
```

Common types:

| Type       | Use for                                            |
|------------|----------------------------------------------------|
| `feat`     | A new feature                                      |
| `fix`      | A bug fix                                          |
| `refactor` | A code change that neither fixes a bug nor adds a feature |
| `docs`     | Documentation only                                 |
| `test`     | Adding or fixing tests                             |
| `chore`    | Tooling, deps, or housekeeping                     |
| `perf`     | A performance improvement                          |
| `ci`       | CI/build configuration                             |

Examples:

```
feat: add DoH provider selection to PrivacyControls
fix: compare exit IP against real IP correctly in leak test
docs: document the privileged-helper requirement for the kill-switch
```

Keep commits focused; analyze the full diff (`git diff <base>...HEAD`) when opening a
PR and write a summary plus a test plan.

---

## Before you open a PR

- [ ] `cargo check` and `cargo test` pass
- [ ] `tsc` is clean (or `just check` for both)
- [ ] The app runs: `./ui/node_modules/.bin/tauri dev`
- [ ] Any new host bridge capability is toggleable and respects Isolation Mode
- [ ] No UI copy over-promises privacy
- [ ] Commits follow Conventional Commits
