# PRP — Griffith OS: From Privacy App → Deeply-Integrated Layer → Real OS

*Product Requirement Prompt · 2026-06-14 · grounded in [RESEARCH-os-evolution.md](RESEARCH-os-evolution.md)*

## Goal

Evolve GOS from a Tauri app window into (1) a **system-level macOS privacy layer** that feels like
part of the OS, and (2) a **literal, fully-functional GOS operating system** (NixOS-based) that runs
*over* macOS as a near-native VM and boots on PCs. Keep the proven Rust privacy core throughout.

## Why now

v1 works: the Rust core demonstrably routes traffic through Tor (real IP masked) with encrypted DNS,
and the control panel renders. The user wants GOS to stop being "an app like this" and to become a
real OS. Research confirms a credible, honest path exists — split across what macOS *permits* (a
native network extension) and what a *real* OS requires (a separate Linux distro).

## Non-goals / honest limits (do not attempt)

- ❌ Replacing macOS's WindowServer, compositor, system volume, or boot chain (SIP/SSV/secure-boot
  make this impossible — even for root).
- ❌ A bootable GOS USB **on Apple Silicon** (M-series cannot boot from USB).
- ❌ A custom anti-fingerprinting browser (a bespoke webview makes users *more* trackable).
- ❌ Marketing "untraceable / evade law enforcement." GOS is a **lawful, Tails-class privacy tool.**

## The architecture pivot (important)

The always-on system layer **cannot be Tauri** — NetworkExtension/system extensions must be a native,
signed, sandboxed Swift `.app` + extension target. So:

```
            ┌────────────────────────────────────────────────────────┐
            │  gos-core (Rust, EXISTING) → built as a C-ABI cdylib    │
            │  arti(Tor) · DoH · WireGuard(boringtun) · leak · iroh   │
            └───────────────┬───────────────────────┬────────────────┘
                  FFI / XPC │                       │ FFI / XPC
        ┌───────────────────▼─────────┐   ┌─────────▼──────────────────┐
        │ Native Swift menu-bar agent │   │ NEPacketTunnelProvider      │
        │ (LSUIElement + MenuBarExtra)│   │ system extension (Tier 1)   │
        │ + SMAppService priv. helper │   │ routes ALL traffic → Tor/WG │
        └───────────────┬─────────────┘   └────────────────────────────┘
                        │ optional detail window
                ┌───────▼───────────────┐
                │ Tauri dashboard (v1 UI)│  ← today's app becomes this
                └───────────────────────┘
```

The Rust core is reused as-is (compile `gos-privacy` to a `staticlib`/`cdylib`, add a C-ABI). The
Tauri UI is demoted to an optional dashboard; the *system presence* is native Swift.

---

## Tier 1 — Deep macOS system integration  *(answers "not an app like this")*

**Outcome:** a menu-bar app (no Dock window) that tunnels *all* system traffic through Tor/WireGuard
with a kill switch — the difference between "a privacy app" and "a privacy layer."

| Phase | Tasks | Done when |
|------|-------|-----------|
| **1.0 Account & signing** | Parent/guardian or LLC enrolls in Apple Developer Program ($99/yr). Create Developer ID cert + provisioning profile with `packet-tunnel-provider-systemextension`. | A signed+notarized "hello" `.app` launches past Gatekeeper. |
| **1.1 Native shell** | New Swift app target: `LSUIElement`, SwiftUI `MenuBarExtra` status menu, `SMAppService` launch-at-login. Embed the Tauri dashboard as an optional window (or port key panels to SwiftUI). | Menu-bar GOS with connect/disconnect, no Dock icon. |
| **1.2 Rust as a lib** | Add a `gos-ffi` crate: C-ABI over `PrivacyEngine` (start/stop tunnel, status, leak test). Build `cdylib` + header (cbindgen). | Swift calls Rust `gos_start_tor()` and gets status back. |
| **1.3 Packet tunnel** | `NEPacketTunnelProvider` system extension. Start with a **userspace netstack** (boringtun for WireGuard; arti SOCKS for Tor) — follow Mullvad/`wireguard-apple`. **Avoid `includeAllNetworks` initially.** Keep heavy `arti` in an `SMAppService` XPC daemon if it exceeds the ~50 MB extension cap. | Toggling GOS routes *all* browser+app traffic; exit IP ≠ real IP system-wide. |
| **1.4 Kill switch + helper** | Privileged helper (SMAppService) installs PF firewall rules; block egress on tunnel drop. Add DNS-leak/WebRTC checks (reuse `gos-privacy::leak`). | Pull the tunnel → all traffic blocked; leak test passes. |
| **1.5 Browser + polish** | Bundle/launch **Mullvad Browser** (unmodified, MPL-2.0). Global hotkey to summon the panel. Notarized DMG. | One-click "open private browser" routes through GOS. |

**Tier 1 success:** notarized GOS menu-bar app; system-wide Tor/WG tunnel + kill switch; reuses the
Rust core; no Dock window. **Gate: Apple account (age) — see Risks.**

## Tier 2 — GOS environment  *(optional "OS feel" on macOS)*

Full-screen "GOS mode" Space: app launcher, windowing for GOS-native apps, the host cross-over
bridge (files/clipboard/devices, all toggleable — already designed in v1), theming, and a
`manifest.json` **plugin/app SDK** so GOS is expandable. Layered on Tier 1; reuses the Tauri/React UI.

## Tier 3 — GOS Linux, a real OS  *(the literal "fully functional OS")*

**Outcome:** a real, privacy-hardened OS you can run and boot — **no Apple account required**, so this
track can start *immediately* and in parallel with Tier 1's account gate.

| Phase | Tasks | Done when |
|------|-------|-----------|
| **3.0 NixOS GOS image** | `configuration.nix` flake: `hardened` profile, Tor + DoH defaults, pre-installed dev toolchain (Nix), Mullvad/Tor Browser, `iroh`/croc transfer, GOS branding/theme. | `nixos-rebuild` produces a booting GOS desktop. |
| **3.1 Tor gateway** | Second tiny NixOS VM = Whonix-style Tor gateway; GOS workstation routes through it (host-only net). | Workstation traffic exits via the gateway's Tor; fail-closed. |
| **3.2 VM over macOS** | Run GOS via **Apple Virtualization.framework** (UTM Apple backend): VirtioFS shared folders, shared clipboard, Rosetta for x86 Linux binaries. | GOS runs full-screen "over" macOS; files + clipboard cross over. |
| **3.3 Bootable image** | `nixos-generators`/`mkosi` → bootable ISO for **x86 PCs / Intel Macs** (+ optional amnesic "panic" profile). | GOS boots on real PC hardware from USB. |
| **3.4 GOS↔GOS** | Wire `iroh` (self-hosted relay) for encrypted device-to-device transfer between GOS installs. | Send a file GOS→GOS, end-to-end encrypted, across NAT. |

**Tier 3 success:** a NixOS-based GOS that runs as a seamless VM over macOS *and* boots on PCs, with
Tor-everything, a dev toolchain, a hardened browser, and encrypted GOS↔GOS transfer.

---

## Recommended sequence

1. **Start Tier 3.0–3.2 now** (NixOS GOS VM over macOS) — needs no Apple account, gives a demoable
   "real OS over your current OS" fast, and reuses the privacy concepts.
2. **In parallel, resolve the Apple Developer account** (parent/guardian or LLC), then do Tier 1.
3. **Tier 2** last (or fold pieces into Tier 3's desktop).

## Risks & gates

| Risk / gate | Severity | Mitigation |
|-------------|----------|------------|
| **Apple Developer account — minor can't enroll** | 🔴 Blocks Tier 1 distribution | Parent/guardian individual account, or form an LLC; $99/yr. Start Tier 3 (no account) meanwhile. |
| Code-signing / notarization / `-systemextension` profile hell | 🟠 High time-sink | Budget days; verify profile entitlements with `security cms -D`; ship Developer-ID DMG before App Store. |
| NetworkExtension ~50 MB memory cap vs embedded `arti` | 🟠 | Run heavy Tor in an XPC daemon; extension stays thin. |
| `includeAllNetworks` deadlocks / silent ICMP-TCP drops | 🟠 | Use a userspace netstack like Mullvad; avoid the flag early. |
| Asahi/Apple-Silicon bare metal immaturity | 🟡 | Don't target it; VM + x86 USB are the paths. |
| Native Swift + NetworkExtension learning curve | 🟡 | Mullvad (Rust+NE) and Orbot are open-source blueprints to follow. |
| Legal/ethical framing | 🟠 | Tails-style threat-model honesty; never "untraceable." |

## Open decisions (need user input)

1. **Which track first** — Tier 3 (NixOS "real OS" VM, no account needed) or Tier 1 (native macOS
   extension, needs Apple account)? *(Recommended: Tier 3 prototype now, Tier 1 once account is sorted.)*
2. **Apple Developer account** — can a parent/guardian enroll, or do we form an LLC? *(Blocks Tier 1.)*
3. **Keep the Tauri UI** as the dashboard/GOS-environment renderer, or port the control panel to
   native SwiftUI for the menu-bar agent? *(Recommended: keep Tauri for the dashboard; SwiftUI for the menu bar.)*

## Success metric for the whole effort

A user installs GOS and gets **either** (a) a macOS menu-bar app that routes *all* traffic through
Tor/WG with a kill switch, **or** (b) a real GOS Linux desktop running seamlessly over macOS — both
reusing one Rust privacy core, both honestly scoped, both lawful-privacy-framed.
