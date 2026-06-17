# GOS Threat Model & Honest Privacy Claims

GOS is privacy software. To be trustworthy it must be honest about what it can and cannot do. The UI
must never over-promise. This document is the source of truth for those claims.

## What GOS protects against

- **Your ISP / local network seeing where you go.** Tor and/or a VPN tunnel hide destination IPs and
  encrypt traffic in transit. DNS-over-HTTPS hides your lookups from the network.
- **Destination sites seeing your real IP.** Traffic exits from a Tor exit node or VPN endpoint, not
  your home IP ("fake IP" in the user's words = a masked/relayed IP).
- **Local MAC-address tracking.** Optional MAC randomization changes your hardware address on the
  local network.
- **Accidental leaks.** A kill-switch blocks all traffic if the tunnel drops, so nothing escapes in
  the clear. Leak tests verify DNS/IP aren't exposed.

## What GOS does NOT protect against (be honest in the UI)

- **Browser/device fingerprinting.** Canvas, fonts, screen size, and JS side-channels can still
  identify a browser even behind Tor. Mitigated, never eliminated.
- **Logging into accounts.** If you sign into something tied to your identity, you are identified.
- **Global traffic correlation.** An adversary watching both ends of the network can correlate timing.
- **Malware already on the host.** GOS runs on top of the host OS; it can't fix a compromised host.
- **UDP / non-TCP over Tor.** Tor is TCP-only. Whole-system anonymity uses the VPN tunnel; Tor is for
  the browser and selected apps.

## Design rules that follow from this

1. **Default to safe.** Kill-switch and DoH on by default once a tunnel is active.
2. **Show the truth.** Always display the real vs. exit IP and a plain-language verdict
   (`protected` / `partial` / `exposed`). Never claim "untrackable."
3. **Fail closed.** If privacy state is unknown, treat it as exposed and warn.
4. **User owns the bridges.** Every host cross-over (files, apps, clipboard, devices) is opt-in and
   individually revocable; "Isolation Mode" cuts them all at once.
5. **No secret phone-home.** GOS itself must not transmit telemetry.

## Privileges required (and why)

| Action | Needs admin | Why |
|--------|-------------|-----|
| Kill-switch firewall rules | yes | Insert/remove packet-filter rules |
| System proxy configuration | yes (system-wide) | Point the OS at the GOS SOCKS proxy |
| MAC randomization | yes | Change the network interface hardware address |
| Tor / DoH / leak tests | no | Run in user space |

A small privileged helper performs the admin-only actions; the user grants it once.

## GOS Linux (NixOS) — anonymous mode

The GOS Linux OS (`os/`) implements these rules declaratively. `gos.torEverything.enable`
turns on **single-host transparent torification**: an nftables ruleset redirects all
outbound TCP into Tor's `TransPort` and all DNS into Tor's `DNSPort`, while a
default-DROP egress policy lets only loopback, the `tor` user, and DHCP out. This is
**fail-closed by construction** — if Tor stops, the redirect targets are dead and the
drop policy blocks everything, so nothing falls back to the clear. `gos-leaktest`
verifies it (exit IP via Tor, DNS through Tor, IPv6 off, kill-switch).

Honest limits specific to this mode:

- **It is Tails-class, not Whonix-class.** Tor and your apps share one kernel/host, so
  malware running as root could read the real IP before Tor. The two-VM
  `gos-gateway` (Whonix pattern) is the higher-assurance option: the workstation VM
  physically never learns its own public IP. Offer the gateway for higher threat models.
- **Tor is TCP-only.** In anonymous mode, non-TCP/UDP traffic (other than DHCP) is
  **dropped, not sent in the clear** — consistent with "fail closed," but some UDP apps
  simply won't work. That's the correct trade-off, not a bug.
- **Time is not yet Tor-routed.** Clear NTP is disabled (no leak), and the VM takes time
  from the hypervisor clock. Tor-based time (Whonix `sdwdate`) is a planned follow-up.
