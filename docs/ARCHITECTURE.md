# Griffith OS — Architecture

Griffith OS (GOS) is a cross-platform "OS extension": a native [Tauri 2](https://tauri.app)
app with a Rust core and a React/TypeScript web UI. It installs on macOS, Linux,
and Windows and adds a strong privacy/anonymity layer plus deep, toggleable host
integration. v1 ships the **privacy core** as a tray/menu-bar app.

> Privacy scope is intentionally honest: GOS targets **Tails-grade privacy, not
> literal invisibility**. See [`docs/THREAT_MODEL.md`](./THREAT_MODEL.md) for the
> full threat model and explicit non-goals.

---

## 1. Overview

The system is a Rust workspace plus a Vite/React frontend. The Rust side owns all
privileged and security-sensitive logic; the UI is a thin, declarative console that
talks to the core exclusively through Tauri commands.

```
┌──────────────────────────────────────────────────────────────┐
│  ui/  (Vite + React 19 + TypeScript, "secure console" CSS)    │
│  Topbar · StatusHero · PrivacyControls · LeakPanel ·          │
│  CrossoverPanel · NodeCard · Toggle                           │
└───────────────┬──────────────────────────────────────────────┘
                │  invoke("command", args)   (lib/api.ts)
                ▼
┌──────────────────────────────────────────────────────────────┐
│  src-tauri/  (Tauri 2 app shell)                              │
│  lib.rs  → builds AppState, registers commands, tray icon     │
│  commands.rs → #[tauri::command] handlers                     │
└───────┬───────────────┬───────────────┬──────────────────────┘
        ▼               ▼               ▼
┌───────────────┐ ┌───────────────┐ ┌───────────────┐
│ gos-privacy   │ │ gos-platform  │ │ gos-bridge    │
│ PrivacyEngine │ │ PlatformOps   │ │ HostBridge    │
│ Tor/DoH/leak  │ │ per-OS impls  │ │ host crossover│
└───────┬───────┘ └───────────────┘ └───────────────┘
        ▼
┌───────────────┐
│ gos-core      │  shared serde types · settings store · identity
└───────────────┘
```

The root workspace manifest is [`Cargo.toml`](../Cargo.toml). Its members are:

- `crates/gos-core`
- `crates/gos-platform`
- `crates/gos-bridge`
- `crates/gos-privacy`
- `src-tauri`

The pinned toolchain lives in [`rust-toolchain.toml`](../rust-toolchain.toml).

---

## 2. Crate-by-crate breakdown

### `crates/gos-core` — shared types, settings, identity

The foundation crate. Everything else depends on it. No I/O-heavy logic; it defines
the vocabulary the rest of the system speaks.

- **Shared serde types.** All structs that cross the Rust↔UI boundary are defined
  here and serialize as **camelCase** so they land in the UI as idiomatic JS
  objects.
- **JSON settings store.** A small persistence layer that reads/writes the user's
  settings as JSON on disk. Acts as the source of truth for which toggles are on.
- **Node identity.** An **ed25519** keypair that gives this install a stable,
  self-sovereign node identity (used by `NodeCard` in the UI).

### `crates/gos-privacy` — the PrivacyEngine

The heart of v1. Owns the Tor connection, encrypted DNS, and leak testing.

| Module        | Responsibility                                                              |
|---------------|-----------------------------------------------------------------------------|
| `engine.rs`   | `PrivacyEngine` — orchestrates start/stop, holds state, exposes the API     |
| `tor.rs`      | Tor via [`arti-client`](https://crates.io/crates/arti-client) — bootstrap & circuits |
| `doh.rs`      | DNS-over-HTTPS — a small JSON-over-TLS client (rustls + webpki-roots)        |
| `net.rs`      | Network helpers: fetching the real/exit IP, request plumbing                |
| `crypto.rs`   | Crypto helpers used by the engine                                           |

- **Tor** is gated behind the cargo feature **`tor`**, which is **on by default**.
  Building without it compiles a Tor-less engine.
- **DoH** uses `rustls` + `webpki-roots` for trust roots, so there is no dependency
  on the host's system certificate store or DNS resolver.
- **Leak tests** compare the **real IP** (fetched directly) against the **exit IP**
  (fetched over Tor); if they differ as expected, traffic is leaving via Tor.
- A runnable example lives at `crates/gos-privacy/examples/check.rs`:

  ```bash
  cargo run -p gos-privacy --example check -- --tor
  ```

  This exercises the engine end-to-end without the Tauri app or UI.

### `crates/gos-platform` — per-OS abstraction

Defines a single **`PlatformOps`** trait and three implementations selected at
compile time:

```
crates/gos-platform/
  macos/    ┐
  linux/    ├── one impl each, behind #[cfg(target_os = "...")]
  windows/  ┘
```

Responsibilities behind the trait:

- **System proxy** configuration.
- **Firewall kill-switch** — routed through a *future privileged helper* (see
  caveats below).
- **MAC randomization.**

Because only one `#[cfg(target_os)]` impl compiles per target, the same call sites
in `src-tauri` produce correct behavior on each OS without branching in business
logic. See §5.

### `crates/gos-bridge` — HostBridge (host crossover)

`HostBridge` is the deep host-integration layer. It exposes crossover capabilities,
each **gated by its own toggle**:

- Files
- Launching apps
- Clipboard
- Devices

When **every toggle is off**, the app is in **Isolation Mode** — no host crossover
at all. This is the privacy-maximizing default posture.

### `src-tauri` — the Tauri 2 app

The shell that wires everything together and presents the tray UI.

- `lib.rs` builds the application state — **identity**, **settings**, **engine**
  (`PrivacyEngine`), **platform** (`PlatformOps`), and **bridge** (`HostBridge`) —
  registers the command handlers, and installs the **tray icon**.
- `commands.rs` defines the `#[tauri::command]` handlers the UI invokes.
- `main.rs` is a thin entry point that calls `gos_lib::run()`.

---

## 3. Rust ↔ UI command flow

The UI never touches privacy, platform, or bridge logic directly. Every action is a
round-trip through a Tauri command.

```
UI component (e.g. PrivacyControls)
   │  calls a wrapper in ui/lib/api.ts
   ▼
api.ts  →  invoke("start_privacy", args)        // Tauri IPC
   │
   ▼
src-tauri/commands.rs  →  #[tauri::command] fn start_privacy(state)
   │   reads AppState built in lib.rs
   ▼
PrivacyEngine / HostBridge / PlatformOps        // the actual work
   │   returns a gos-core serde type (camelCase)
   ▼
JSON crosses back over IPC
   │
   ▼
ui/lib/useGos.ts  updates hook state  →  React re-renders
```

Key points:

- **`ui/lib/api.ts`** is the single wrapper around Tauri `invoke`. UI components
  import typed functions from here rather than calling `invoke` ad hoc.
- **`ui/lib/useGos.ts`** is the state hook that holds the engine/bridge status and
  exposes actions to components.
- Command return types are the **camelCase serde types from `gos-core`**, so the
  TypeScript side receives well-shaped objects with no field-name translation.
- Components map cleanly to capabilities:
  - `PrivacyControls` → engine start/stop + DoH toggles
  - `LeakPanel` → leak-test results (real IP vs exit IP)
  - `CrossoverPanel` → `HostBridge` toggles (Isolation Mode)
  - `NodeCard` → ed25519 node identity from `gos-core`
  - `StatusHero` / `Topbar` → overall status surface

---

## 4. Privacy data flow

### Starting Tor

```
UI: "Start" → invoke("start_privacy")
   ▼
PrivacyEngine (engine.rs) → tor.rs
   ▼
arti-client bootstraps a Tor client (builds circuits)
   ▼
net.rs fetches the EXIT IP over the Tor client
   ▼
engine returns status { tor: connected, exitIp: ... } to the UI
```

The `arti-client` bootstrap is the slow step; the UI reflects a connecting state
until the circuit is ready.

### Leak test

The leak test answers one question: *is my traffic actually leaving via Tor, and is
my DNS encrypted?*

```
Leak test
   ├── direct fetch  (net.rs, no Tor)   → REAL IP
   ├── Tor fetch     (net.rs, over arti) → EXIT IP
   └── DoH check     (doh.rs, JSON-over-TLS via rustls/webpki-roots)
   ▼
Compare: real IP ≠ exit IP  ⇒  traffic is exiting through Tor
         DoH resolves over TLS  ⇒  DNS is not leaking to the local resolver
   ▼
Results surface in LeakPanel
```

A passing test means the **direct IP** and the **Tor exit IP** differ and the DoH
check succeeds. A failing/identical-IP result means traffic is not being tunneled.

You can run the same logic headless with the example:

```bash
cargo run -p gos-privacy --example check -- --tor
```

---

## 5. Three builds from one codebase

GOS produces native macOS, Linux, and Windows builds from a single source tree. Two
mechanisms make this work:

1. **The `PlatformOps` trait + `#[cfg(target_os)]` impls** (`crates/gos-platform`).
   Business logic in `src-tauri` calls trait methods. At compile time, only the
   `macos`, `linux`, or `windows` implementation is included via `#[cfg(target_os
   = "...")]`. There is exactly one implementation per target, so each binary gets
   the correct system-proxy, kill-switch, and MAC-randomization behavior with no
   runtime OS branching in callers.

2. **Tauri 2 packaging.** Tauri bundles the same Rust core + web UI into a native
   app per platform (menu-bar app on macOS, tray app on Linux/Windows). The web UI
   is identical across targets; only the platform crate's compiled impl differs.

The result: one workspace, one command flow, one UI — three native artifacts.

---

## 6. Caveats and current scope

These are load-bearing honesty notes, expanded in
[`docs/THREAT_MODEL.md`](./THREAT_MODEL.md):

- **Tor is TCP-only.** It covers the browser and selected apps, not the whole
  system. **Whole-system tunneling is planned via a VPN**, not Tor.
- **Kill-switch, MAC randomization, and system-proxy changes need admin
  privileges.** They are designed to route through a **privileged helper that is not
  yet built**. Until then, those `PlatformOps` paths are not fully wired.
- **Not invisibility.** GOS raises the cost of correlation and tracking
  significantly, but does not make the user untraceable. UI copy must never claim
  otherwise (see [`CONTRIBUTING.md`](../CONTRIBUTING.md)).
