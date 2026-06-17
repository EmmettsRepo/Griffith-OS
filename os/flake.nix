{
  description = "Griffith OS (GOS) — a privacy-hardened NixOS that runs as a VM over macOS or boots on PCs";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, nixos-generators, ... }:
    let
      lib = nixpkgs.lib;

      # The GOS feature modules — the actual "operating system", with no disk/boot
      # specifics so the same set works for installed systems AND generated images.
      gosModules = [
        ./modules/gos-base.nix
        ./modules/gos-hardening.nix
        ./modules/gos-tor.nix
        ./modules/gos-anti-tracking.nix
        ./modules/gos-leaktest.nix
        ./modules/gos-desktop.nix
        ./modules/gos-toolchain.nix
        ./modules/gos-browser.nix
        ./modules/gos-transfer.nix
        ./modules/gos-vm-guest.nix
      ];

      # A complete, evaluable workstation system for a given arch (adds disk + boot).
      # `extra` lets a variant flip options (e.g. enable anonymous mode).
      mkWorkstation = system: extra:
        lib.nixosSystem {
          inherit system;
          modules = gosModules ++ [ ./hosts/disk-vm.nix ] ++ extra;
        };

      mkImage = system: format: extra:
        nixos-generators.nixosGenerate {
          inherit system format;
          modules = gosModules ++ extra;
        };
    in
    {
      # `nixos-rebuild`-able / evaluable systems (used for validation too).
      nixosConfigurations = {
        gos-workstation = mkWorkstation "aarch64-linux" [ ]; # Apple Silicon VM guest
        gos-workstation-x86 = mkWorkstation "x86_64-linux" [ ]; # PCs / Intel Macs

        # "Anonymous mode": the whole host is forced through Tor, fail-closed.
        gos-workstation-anon = mkWorkstation "aarch64-linux" [
          { gos.torEverything.enable = true; }
        ];
        gos-workstation-anon-x86 = mkWorkstation "x86_64-linux" [
          { gos.torEverything.enable = true; }
        ];

        gos-gateway = lib.nixosSystem {
          system = "aarch64-linux";
          modules = [ ./hosts/gos-gateway.nix ];
        };
      };

      # Bootable / runnable image artifacts (need a Linux builder to realize).
      # `*-anon` images boot straight into anonymous mode (Tor-everything).
      packages.aarch64-linux = {
        iso = mkImage "aarch64-linux" "iso" [ ];
        qcow = mkImage "aarch64-linux" "qcow" [ ];
        iso-anon = mkImage "aarch64-linux" "iso" [ { gos.torEverything.enable = true; } ];
        qcow-anon = mkImage "aarch64-linux" "qcow" [ { gos.torEverything.enable = true; } ];
      };
      packages.x86_64-linux = {
        iso = mkImage "x86_64-linux" "iso" [ ];
        qcow = mkImage "x86_64-linux" "qcow" [ ];
        iso-anon = mkImage "x86_64-linux" "iso" [ { gos.torEverything.enable = true; } ];
        qcow-anon = mkImage "x86_64-linux" "qcow" [ { gos.torEverything.enable = true; } ];
      };
    };
}
