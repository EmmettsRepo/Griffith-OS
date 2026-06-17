{
  description = "Griffith OS (GOS) — a privacy-hardened NixOS that runs as a VM over macOS or boots on PCs";

  inputs = {
    # Pinned to an exact nixos-25.05 commit (not the moving branch) so the laptop's
    # live auto-update only applies YOUR config changes against a fixed base —
    # small, cache-friendly rebuilds. Bump this rev to pull in nixpkgs updates.
    nixpkgs.url = "github:NixOS/nixpkgs/ac62194c3917d5f474c1a844b6fd6da2db95077d";
    nixos-generators = {
      url = "github:nix-community/nixos-generators/8946737ff703382fda7623b9fab071d037e897d5";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, ... }:
    let
      lib = nixpkgs.lib;

      # Core GOS feature set — the actual "operating system", hardware/host-agnostic
      # (no disk, boot, or VM specifics) so it works for VMs AND bare-metal installs.
      gosFeatures = [
        ./modules/gos-base.nix
        ./modules/gos-hardening.nix
        ./modules/gos-tor.nix
        ./modules/gos-anti-tracking.nix
        ./modules/gos-leaktest.nix
        ./modules/gos-autoupdate.nix
        ./modules/gos-desktop.nix
        ./modules/gos-toolchain.nix
        ./modules/gos-browser.nix
        ./modules/gos-transfer.nix
      ];

      vmGuest = ./modules/gos-vm-guest.nix; # UTM / Apple-Virtualization integration
      installer = ./modules/gos-installer.nix; # `gos-install` for bare-metal installs
      anon = { gos.torEverything.enable = true; }; # force all traffic through Tor

      mkSystem = system: modules: lib.nixosSystem { inherit system; modules = modules; };

      # VM workstation (runs over macOS in UTM) — VM guest + VM disk/boot.
      mkWorkstation = system: extra:
        mkSystem system (gosFeatures ++ [ vmGuest ./hosts/disk-vm.nix ] ++ extra);

      # Bare-metal install (external USB/SSD or a PC's disk) — real-hardware host.
      mkPortable = system: extra:
        mkSystem system (gosFeatures ++ [ ./hosts/disk-portable.nix ] ++ extra);

      # Live/installer ISO — boot it on a PC to install GOS (ships `gos-install`).
      mkIso = system: extra:
        nixos-generators.nixosGenerate {
          inherit system;
          format = "iso";
          modules = gosFeatures ++ [ installer ] ++ extra;
        };

      # Disk image for a VM (UTM) — VM guest integration baked in.
      mkQcow = system: extra:
        nixos-generators.nixosGenerate {
          inherit system;
          format = "qcow";
          modules = gosFeatures ++ [ vmGuest ] ++ extra;
        };
    in
    {
      # `nixos-rebuild`-able / evaluable systems (used for validation too).
      nixosConfigurations = {
        # VM over macOS (UTM).
        gos-workstation = mkWorkstation "aarch64-linux" [ ]; # Apple Silicon VM guest
        gos-workstation-x86 = mkWorkstation "x86_64-linux" [ ]; # Intel-Mac / PC VM
        gos-workstation-anon = mkWorkstation "aarch64-linux" [ anon ];
        gos-workstation-anon-x86 = mkWorkstation "x86_64-linux" [ anon ];

        # Bare-metal portable install (target an external SSD / PC disk).
        # Auto-updates itself from this repo so pushes show up on the laptop.
        gos-portable-x86 = mkPortable "x86_64-linux" [
          { gos.autoUpdate = { enable = true; flakeTarget = "gos-portable-x86"; }; }
        ];
        gos-portable-anon-x86 = mkPortable "x86_64-linux" [
          anon
          { gos.autoUpdate = { enable = true; flakeTarget = "gos-portable-anon-x86"; }; }
        ];

        # Optional Whonix-style Tor gateway VM.
        gos-gateway = mkSystem "aarch64-linux" [ ./hosts/gos-gateway.nix ];
      };

      # Bootable / runnable image artifacts (need a Linux builder to realize).
      # `iso*` = installer media for PCs; `qcow*` = VM disks for UTM.
      # `*-anon` boot straight into anonymous mode (Tor-everything).
      packages.aarch64-linux = {
        iso = mkIso "aarch64-linux" [ ];
        qcow = mkQcow "aarch64-linux" [ ];
        iso-anon = mkIso "aarch64-linux" [ anon ];
        qcow-anon = mkQcow "aarch64-linux" [ anon ];
      };
      packages.x86_64-linux = {
        iso = mkIso "x86_64-linux" [ ];
        qcow = mkQcow "x86_64-linux" [ ];
        iso-anon = mkIso "x86_64-linux" [ anon ];
        qcow-anon = mkQcow "x86_64-linux" [ anon ];
      };
    };
}
