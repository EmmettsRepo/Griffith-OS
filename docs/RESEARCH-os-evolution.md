# GOS → Deep macOS Integration & "Real OS": Research Report

*Generated 2026-06-14 · Multi-agent web research (60+ sources, primary Apple/Tor/NixOS/Asahi docs) · Confidence: High*

## Executive summary

Three honest conclusions shape everything below:

1. **You cannot make macOS "become" GOS.** SIP blocks even *root* from modifying system components; the system volume is read-only and Merkle-sealed by Apple; Apple-silicon secure boot verifies that seal *before the kernel runs*. WindowServer/the compositor cannot be replaced. ([Apple SSV](https://support.apple.com/guide/security/signed-system-volume-security-secd698747c9/web), [Apple SIP](https://support.apple.com/guide/security/system-integrity-protection-secb7ea06b49/web))
2. **"Full macOS integration" IS achievable** — as a native **menu-bar agent + a sanctioned NetworkExtension** that routes *all* traffic through Tor/WireGuard with a kill switch. This is exactly how Mullvad/Tailscale/Orbot work. But it **must be a native Swift app + system-extension target — Tauri cannot host it.** The Rust core survives as a linked library/daemon.
3. **A literal "fully functional OS" = a separate NixOS-based GOS Linux**, run as a fast VM *over* macOS (Apple Virtualization.framework) and/or booted on x86 PCs. Apple Silicon **physically cannot boot from USB**, so bare-metal GOS on M-series is not the path.

Two hard gates: **(a)** distributing the macOS system-extension needs the $99/yr Apple Developer Program + notarization, and **a 15-year-old cannot legally hold that account** — a parent/guardian or an LLC must. **(b)** GOS must be framed as a *lawful* privacy tool (Tails-style), never "untraceable / evade law enforcement."

---

## 1. Deep macOS system integration

**Route all traffic → `NEPacketTunnelProvider`.** Of the five NetworkExtension provider types, only the Packet Tunnel Provider gives a TUN device carrying *all* IP traffic (TCP+UDP+ICMP) into your own tunnel — the canonical VPN provider. Transparent Proxy is TCP/UDP-flow only and breaks on UDP/localhost; Content Filter is read-only (allow/drop, good for a kill-switch/leak blocker, not routing); App/DNS proxies are scoped. ([Apple TN3134](https://developer.apple.com/documentation/technotes/tn3134-network-extension-provider-deployment), [forums/738636](https://developer.apple.com/forums/thread/738636), [WWDC25 234](https://developer.apple.com/videos/play/wwdc2025/234/))

**The entitlement is NOT gated.** Per Apple DTS (Quinn), `com.apple.developer.networking.networkextension` (covers packet-tunnel/content-filter/etc.) is **auto-granted to any paid developer — no manual Apple approval for ~10 years.** ([forums/816877](https://developer.apple.com/forums/thread/816877)) The real friction is *plumbing*: Developer ID signing, the `-systemextension` entitlement suffix, provisioning-profile/entitlement matching, and notarization.

**How the pros do it (copy Mullvad).** Mullvad = open-source **Rust + NEPacketTunnelProvider**, GUI and tunnel in separate processes, Rust WireGuard engine (GotaTun/boringtun fork) FFI-linked into the extension. They **deliberately avoid `includeAllNetworks`** (documented app-update network deadlock + silent ICMP/TCP failures, reported to Apple Feb 2025) and run a **userspace netstack** instead. Kill switch = always-on drop-on-failure. ([Mullvad: why not includeAllNetworks](https://mullvad.net/en/blog/why-we-still-dont-use-includeallnetworks), [GotaTun](https://mullvad.net/en/blog/announcing-gotatun-the-future-of-wireguard-at-mullvad-vpn), [DeepWiki](https://deepwiki.com/mullvad/mullvadvpn-app/4-api-and-backend-services)) Tailscale ships a packet-tunnel system extension (`IPNExtension`); Orbot is Tor-over-NetworkExtension and documents a **~50 MB extension memory cap** — a real constraint for an embedded `arti` engine (keep heavy Tor in an XPC daemon). ([Tailscale variants](https://tailscale.com/kb/1065/macos-variants), [Orbot FAQ](https://orbot.app/en/faqs/))

**Feel like the OS, not an app.** `LSUIElement` + SwiftUI **`MenuBarExtra`** = no Dock icon, menu-bar-only. **`SMAppService`** (macOS 13+, replaces SMJobBless) registers launch-at-login + a privileged helper whose executable lives inside the app bundle. Global hotkey via `RegisterEventHotKey`. ([SMAppService writeup](https://theevilbit.github.io/posts/smappservice/), [MenuBarExtra](https://sarunw.com/posts/swiftui-menu-bar-app/))

**Architecture reality.** The extension MUST be a native, signed, sandboxed Swift `.app` + extension target — **Tauri has no support for this** ([tauri#9586](https://github.com/tauri-apps/tauri/issues/9586)). But the Rust core is *not* rewritten: ship it as a `cdylib`/`staticlib` with a C-ABI and either **FFI-link it into the extension** (Mullvad/GotaTun pattern) or run it as an **SMAppService XPC daemon**. Tauri can remain as an optional dashboard window.

**Verdict:** Achievable solo, in order: (1) menu-bar agent (easy), (2) native Swift shell + `NEPacketTunnelProvider` FFI-linking the Rust `arti`/DoH core, (3) kill switch via PF helper or OS drop-on-failure. Biggest costs: learning native macOS + **code-signing/notarization hell** + the Apple-account age gate.

## 2. The literal "fully functional OS"

**Best base = NixOS.** It's declarative/reproducible (the OS *is* a versioned config file — matches "expandable"), has a built-in **`hardened` profile**, first-class **aarch64**, atomic rollback, and 120k+ packages. Tails (amnesic) fights "ships a dev toolchain"; Whonix/Qubes have **no Apple-Silicon support**; custom Debian/Alpine = more manual upkeep. **Reuse the Whonix *gateway pattern*** (a tiny Tor-router VM others route through) as a NixOS module rather than forking Whonix. ([NixOS config](https://wiki.nixos.org/wiki/NixOS_system_configuration), [NixOS hardened](https://dataswamp.org/~solene/2022-01-13-nixos-hardened.html), [Whonix gateway](https://www.whonix.org/wiki/Whonix-Gateway))

**Installed-immutable > amnesic** for a daily-driver privacy *dev* OS (persistence + tamper-resistance + rollback); offer an *optional* amnesic/Tor-everything "panic" profile. Build images with `nixos-generators`/`mkosi`/`live-build` — a first bootable ISO is a weekend; maintenance is the real cost (NixOS absorbs it via config-as-code). ([mkosi](https://github.com/systemd/mkosi), [bootc/Universal Blue](https://lwn.net/Articles/1042708/))

**Apple Silicon bare metal is the wrong first target.** **M-series physically cannot boot from USB** ([Asahi FAQ](https://asahilinux.org/docs/project/faq/)). Asahi: M1/M2 usable, M3 no-GPU, M4/M5 bring-up. Bootable-USB GOS works on **x86 PCs / Intel Macs** only.

**Verdict:** Fork-and-extend **NixOS**; reuse the Whonix gateway *pattern*; ship primarily as a VM (next section) + an x86 bootable-USB edition as the "runs on bare metal" proof. Writing a novel kernel/OS from scratch is not feasible and not needed.

## 3. Bridging — a real GOS OS running *over* macOS

**Apple Virtualization.framework is the winner** for "real OS over macOS": native hypervisor on Apple Silicon, **Rosetta-for-Linux** (`VZLinuxRosettaDirectoryShare` runs unmodified x86-64 Linux binaries near-native), GPU, **VirtioFS shared folders**, and **shared clipboard**. **UTM (Apple backend)** delivers all of this without writing Swift. QEMU only for cross-arch emulation. A NixOS **Tor-gateway VM** + GOS workstation VM on a host-only network recreates Whonix isolation. ([Apple Virtualization](https://developer.apple.com/documentation/virtualization), [Rosetta-for-Linux](https://developer.apple.com/documentation/Virtualization/running-intel-binaries-in-linux-vms-with-rosetta), [UTM Rosetta](https://docs.getutm.app/advanced/rosetta/))

This path **fulfills the original "runs over your current OS + items cross over" vision — but as a *real* OS**: VirtioFS = files cross over, shared clipboard = clipboard crosses over, host networking = devices cross over. **No Apple Developer account required.**

## 4. Supporting pieces

- **Toolchain:** **Nix** for the real OS image (only genuinely reproducible "all languages baked in"); **mise + a small Brewfile** for the macOS-app dev environment today. ([Nix vs Homebrew](https://betterstack.com/community/guides/linux/homebrew-vs-nix/))
- **Browser — do NOT build your own.** Bundle/launch **Mullvad Browser** (MPL-2.0, Firefox-ESR hardening, *no* Tor-network dependency — fits GOS's own-proxy model) or **Tor Browser** (adds Tor network). A custom Chromium/webview makes users a *unique* fingerprint — the opposite of the goal; anti-fingerprinting is an anonymity-set problem only Tor/Mullvad's years of Firefox-ESR work solve. Ship **unmodified** (don't rebrand — trademarks). ([Tor: Mullvad Browser](https://blog.torproject.org/releasing-mullvad-browser/), [MPL-2.0](https://github.com/mullvad/browser-extension/blob/main/LICENSE.md))
- **GOS↔GOS transfer:** **`iroh`** (Rust/QUIC, key-based dialing, NAT hole-punching, BLAKE3 blobs, →1.0; self-host a relay; pin versions) for a persistent device mesh; **croc/magic-wormhole** for instant code-word sends. ([iroh](https://github.com/n0-computer/iroh))

## 5. Honest constraints (must internalize)

- **Cannot replace macOS internals** (SIP blocks root, SSV read-only + Merkle-sealed, secure boot verifies before kernel). GOS *augments* macOS; it doesn't *become* it. ([Apple SIP](https://support.apple.com/guide/security/system-integrity-protection-secb7ea06b49/web), [Apple SSV](https://support.apple.com/guide/security/signed-system-volume-security-secd698747c9/web))
- **$99/yr Apple Developer Program** is mandatory for signing+notarizing the system extension; **enrollment requires legal age of majority** → a **15-year-old cannot enroll**; a parent/guardian (individual) or an **LLC** must hold the account. Solve before any distributable build. ([Apple enrollment](https://developer.apple.com/help/account/membership/program-enrollment/))
- **Framing:** lawful privacy tool, threat-model-honest (Tails ships a "safe but not magic" warnings page). Never "untraceable / evade law enforcement" — invites app-review rejection, payment/infra deplatforming, trademark and legal risk. ([Tor: about criminals](https://support.torproject.org/abuse/what-about-criminals/), [Tails warnings](https://tails.net/doc/about/warnings/index.en.html))

---

## Recommended tiered roadmap (detail in `PRP-gos-os-evolution.md`)

- **Tier 1 — Deep macOS integration:** native Swift menu-bar agent + `NEPacketTunnelProvider` system extension FFI-linking the Rust core; SMAppService helper + kill switch; bundle Mullvad Browser. *(needs Apple acct)*
- **Tier 2 — GOS environment:** full-screen "GOS mode," launcher, plugin SDK, host bridge. *(layered, optional)*
- **Tier 3 — GOS Linux (real OS):** NixOS-based GOS + Tor gateway, delivered as an Apple-Virtualization VM "over" macOS (VirtioFS/clipboard cross-over) *and* an x86 bootable image. *(no Apple acct needed — fastest "real OS" demo)*
