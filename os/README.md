# Griffith OS (GOS) — the Linux OS

A privacy-hardened **NixOS**-based operating system. It runs as a near-native VM
*over* macOS (Apple Virtualization.framework / UTM) and boots on x86 PCs. The whole
OS is declarative — it *is* this flake — so it's reproducible and easy to extend.

> **Honest scope:** GOS gives strong, Tails-class privacy by default. It hides your
> traffic and DNS and ships hardened browsers — it does **not** make you invisible.
> It is lawful privacy software. See [../docs/THREAT_MODEL.md](../docs/THREAT_MODEL.md).

## What's in the image

| Module | Gives you |
|--------|-----------|
| `gos-base` | hostname `gos`, user `gos`, UTC timezone, Nix flakes, branding |
| `gos-hardening` | firewall default-deny, sysctl hardening, AppArmor, no core dumps, IPv6 off |
| `gos-tor` | Tor client: SOCKS 9050 + transparent TransPort 9040 / DNSPort 5353; DoT when not torifying |
| `gos-anti-tracking` | **the core**: MAC randomization + no clear-NTP (always on); `gos.torEverything` forces ALL traffic through Tor, fail-closed |
| `gos-leaktest` | `gos-leaktest` command that proves anti-tracking works (exit IP, DNS, IPv6, kill switch) |
| `gos-desktop` | trimmed GNOME, dark + green GOS direction |
| `gos-toolchain` | git, gcc, node+npm, python, go, rustup, neovim — reproducible |
| `gos-browser` | **Mullvad Browser** + **Tor Browser** (hardened, anti-fingerprint) |
| `gos-transfer` | `croc` + `magic-wormhole` for encrypted device-to-device sends |
| `gos-vm-guest` | VirtioFS shared folder (`/mnt/host`) + SPICE clipboard for macOS cross-over |

`hosts/gos-gateway.nix` is an optional Whonix-style **Tor gateway** VM that forces
*all* of a workstation VM's traffic through Tor (run it as a second VM).

## Anti-tracking — the core of GOS

Two protections are **always on**, in every GOS build:

- **MAC-address randomization** (NetworkManager) so your NIC isn't a stable identifier
  on any network you join.
- **No clear-text NTP** — time-sync would leak your real IP; it's disabled (the VM uses
  the hypervisor clock).

On top of that, **anonymous mode** forces the *whole host* through Tor, fail-closed —
single VM, no gateway needed (Tails-style). Enable it one of two ways:

```nix
# in a config, or via a *-anon build target:
gos.torEverything.enable = true;
```

When on, an nftables ruleset redirects **all** outbound TCP into Tor's `TransPort` and
**all** DNS into Tor's `DNSPort`, and a default-DROP egress policy permits only
loopback, the `tor` user, and DHCP. If Tor stops, traffic is **blocked, never clear**.
Pre-built anonymous images:

```bash
nix build .#packages.x86_64-linux.iso-anon     # bootable ISO, anonymous by default
nix build .#packages.aarch64-linux.qcow-anon   # Apple-Silicon VM disk, anonymous
```

**Prove it after boot:**

```bash
gos-leaktest                  # exit IP via Tor? DNS via Tor? IPv6 off?
sudo gos-leaktest --killswitch   # also stop Tor briefly → confirm traffic blocks
```

> Honest scope: single-host anonymous mode is **Tails-class**. For a higher threat
> model use the two-VM `gos-gateway` (Whonix pattern), where the workstation never
> learns its own public IP. Tor is TCP-only — in anonymous mode non-TCP traffic is
> dropped, not leaked. See [../docs/THREAT_MODEL.md](../docs/THREAT_MODEL.md).

## Prerequisites

- **Nix** with flakes. Install on macOS (needs your admin password once):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
  ```
- To **build** a Linux image you need a **Linux builder** (you can't realize a
  Linux derivation on macOS alone). Evaluating/validating the config works on macOS.
- **No local Nix? Use CI.** Pushing this repo to GitHub runs
  `.github/workflows/build-os.yml`, which validates the config and builds a
  bootable x86_64 ISO as a downloadable artifact — no local Nix or sudo needed.

## 1. Validate the whole OS config (works on macOS, no build)

```bash
cd os
nix eval --raw .#nixosConfigurations.gos-workstation.config.system.build.toplevel.drvPath
nix eval --raw .#nixosConfigurations.gos-workstation-anon.config.system.build.toplevel.drvPath
```
A `/nix/store/….drv` path means the entire OS definition evaluates cleanly (the second
line validates anonymous mode). CI runs the same two evals on every push.

## 2. Build a runnable image (needs a Linux builder)

**Easiest — build on any Linux box / WSL / CI:**
```bash
nix build .#packages.x86_64-linux.iso     # bootable ISO for PCs / Intel Macs
nix build .#packages.aarch64-linux.qcow   # disk image for an Apple-Silicon VM
```

**From this Mac — add a Linux builder once**, then the same `nix build` works:
- Simplest: enable `nix.linux-builder` via **nix-darwin** (spins a tiny NixOS
  builder VM on demand), or register a remote Linux builder in `/etc/nix/machines`.
- Or run the build in **GitHub Actions** (ubuntu runner) and download the artifact.

## 3. Run GOS over macOS (UTM, recommended)

1. Install **UTM** and create a VM using the **Apple Virtualization backend** (Apple
   Silicon) for near-native speed + Rosetta for x86 Linux binaries.
2. Attach the built `qcow`/ISO. Enable **Clipboard Sharing** and add a **Shared
   Directory** (VirtioFS, tag `share`) → it appears at `/mnt/host` in GOS.
3. Boot. Log in as **`gos` / `gos`** (change the password immediately).

This is the "real OS over your current OS": files cross over via `/mnt/host`,
clipboard crosses over via SPICE — but it's a genuine Linux OS, not an app window.

## 3b. Install GOS to a USB/SSD (boot it on a PC)

A real, persistent GOS install on an external SSD (or a PC's own disk) that boots on
any UEFI x86 machine — e.g. a ThinkPad X1 Carbon. Not a live ISO: full persistence,
rollbacks, the works.

1. **Get the installer ISO** — download `gos-iso-x86_64` from the latest CI run's
   artifacts (GitHub → Actions → newest run), or build `.#packages.x86_64-linux.iso`.
2. **Flash it to a USB stick** (≥4 GB) with balenaEtcher / Raspberry Pi Imager, or:
   ```bash
   sudo dd if=gos.iso of=/dev/sdX bs=4M status=progress conv=fsync   # sdX = the USB stick
   ```
3. **Boot the PC from the USB** (ThinkPad: tap F12 for the boot menu) → live GOS desktop.
4. **Plug in the target SSD**, connect Wi-Fi (top-right), then run the guided installer:
   ```bash
   sudo gos-install
   ```
   It lists your disks, asks which one to use and whether to default to anonymous
   mode, then wipes it, creates the `ESP` + `nixos` partitions, and installs GOS.
5. **Reboot, remove the USB stick**, pick the SSD in the boot menu. Log in `gos`/`gos`,
   run `passwd`, then `gos-leaktest`.

> The installer pulls the GOS flake from GitHub, so it always lays down the current
> definition. Prefer manual? Make `ESP` (FAT32, label `ESP`) + `nixos` (ext4, label
> `nixos`), mount at `/mnt` + `/mnt/boot`, then
> `nixos-install --flake github:EmmettsRepo/Griffith-OS?dir=os#gos-portable-x86`
> (append `-anon` to that name for Tor-everything by default).

## 3c. Live auto-update from GitHub

Portable installs **update themselves from this repo**: push a change here and the
laptop pulls it and rebuilds within ~5 minutes — no manual steps. It's on by default
in `gos-portable-x86` / `gos-portable-anon-x86` (via `system.autoUpgrade`, a
`nixos-upgrade` systemd timer).

- **Rollback-safe:** each cycle runs `nixos-rebuild switch`. A broken push just fails
  the switch and the system stays on the last good generation — it can't brick the OS.
- **Fast + reproducible:** `nixpkgs` is pinned to an exact commit in `flake.nix`, so
  only *your* config changes move; rebuilds are small and hit the binary cache.
- **Works in anonymous mode:** the GitHub fetch is torified by the transparent proxy.
- **Tune or trigger it:**
  ```bash
  systemctl list-timers nixos-upgrade        # see the schedule / next run
  sudo systemctl start nixos-upgrade.service # pull + rebuild right now
  journalctl -u nixos-upgrade -f             # watch an update happen
  ```
  Change the cadence by setting `gos.autoUpdate.dates` (default `*:0/5` = every 5 min).

> Not real-time push (the laptop can't be reached from GitHub behind NAT) — it's a
> fast poll. Already installed an older build? Run once:
> `sudo nixos-rebuild switch --flake github:EmmettsRepo/Griffith-OS?dir=os#gos-portable-x86 --refresh`
> and it self-updates from then on.

## 4. Torify everything (optional gateway)

Build/boot `gos-gateway` as a second VM, put both VMs on a host-only network
(`10.152.152.0/24`, gateway = `.10`), and set the workstation's gateway/DNS to the
gateway. All workstation traffic then exits through Tor, fail-closed.

## Extending GOS

Add a package or service by editing a module in `modules/` (or add a new one to
`gosModules` in `flake.nix`) and rebuild. Because the OS is declarative, every
change is reproducible and reversible (`nixos-rebuild` keeps rollback generations).

## Roadmap

- ✅ **Anti-tracking core** — MAC randomization, no clear-NTP, and fail-closed
  single-host Tor-everything (`gos.torEverything`) verified by `gos-leaktest`.
- Tor-routed time (Whonix `sdwdate`) so anonymous mode never touches NTP.
- Persistent **iroh** GOS↔GOS device mesh (self-hosted relay).
- GOS theme/branding pass (GDM + GTK + wallpaper).
- Amnesic "panic" profile (tmpfs root) building on anonymous mode.
- Native Apple-Virtualization launcher app (skip UTM).
