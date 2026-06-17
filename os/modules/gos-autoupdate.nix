# GOS live auto-update: the installed system pulls the latest GOS config from the
# GitHub repo and rebuilds itself on a timer — so a `git push` from your dev
# machine shows up on the laptop a few minutes later, no manual steps.
#
# Safety: each cycle runs `nixos-rebuild switch`. A broken push simply fails the
# switch and the system stays on the last good generation — it can't brick the OS.
# nixpkgs is pinned (in flake.nix), so only YOUR changes move; rebuilds are small
# and hit the binary cache. Works in anonymous mode too (the fetch is torified).
{ config, pkgs, lib, ... }:
let
  cfg = config.gos.autoUpdate;
in
{
  options.gos.autoUpdate = {
    enable = lib.mkEnableOption "pulling + rebuilding from the GOS GitHub repo on a timer";

    flakeTarget = lib.mkOption {
      type = lib.types.str;
      default = "gos-portable-x86";
      description = "Which nixosConfiguration in the GOS flake this machine tracks.";
    };

    dates = lib.mkOption {
      type = lib.types.str;
      default = "*:0/5"; # every 5 minutes — near-real-time
      description = "systemd OnCalendar schedule for the update check.";
    };
  };

  config = lib.mkIf cfg.enable {
    system.autoUpgrade = {
      enable = true;
      # The module auto-appends `--refresh --flake <this>` when `flake` is set, so
      # new commits are picked up promptly (--refresh bypasses the 1h flake cache).
      # nixpkgs is pinned by rev in flake.nix, so the inputs are already locked.
      flake = "github:EmmettsRepo/Griffith-OS?dir=os#${cfg.flakeTarget}";
      dates = cfg.dates;
      randomizedDelaySec = "45s";
      allowReboot = false; # config/service changes apply live; never auto-reboots
    };
  };
}
