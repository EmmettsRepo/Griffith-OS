# GOS base: identity, user, locale, Nix settings, branding.
{ config, pkgs, lib, ... }:
{
  networking.hostName = lib.mkDefault "gos";

  # Privacy: report UTC so the local timezone isn't a fingerprint.
  time.timeZone = "UTC";
  i18n.defaultLocale = "en_US.UTF-8";
  console.keyMap = "us";

  users.mutableUsers = true;
  users.users.gos = {
    isNormalUser = true;
    description = "Griffith OS";
    extraGroups = [ "wheel" "networkmanager" "video" "audio" ];
    initialPassword = "gos"; # change on first login
  };
  security.sudo.wheelNeedsPassword = true;

  nix.settings.experimental-features = [ "nix-command" "flakes" ];
  # Tor Browser / Mullvad Browser are free, but allow unfree to be safe.
  nixpkgs.config.allowUnfree = true;

  environment.etc."gos-release".text = ''
    Griffith OS (GOS) 0.1 — privacy-hardened NixOS
  '';
  users.motd = ''
    Welcome to Griffith OS — strong, Tails-class privacy, honestly scoped.
    It hides your traffic; it does not make you invisible. See /etc/gos-release.
  '';

  system.stateVersion = "25.05";
}
