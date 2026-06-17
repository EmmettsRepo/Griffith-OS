# GOS desktop: GNOME (well-supported in VMs, clipboard sharing works), trimmed,
# with a dark + green GOS direction. Deeper theming comes later.
{ config, pkgs, lib, ... }:
{
  services.xserver.enable = true;
  # nixos-25.05 keeps GDM/GNOME under the services.xserver.* namespace
  # (the unprefixed services.displayManager.gnome paths land in a later release).
  services.xserver.displayManager.gdm.enable = true;
  services.xserver.desktopManager.gnome.enable = true;
  services.xserver.xkb.layout = "us";

  # Default to a dark, intentional look.
  environment.systemPackages = with pkgs; [
    gnome-tweaks
    adwaita-icon-theme
  ];

  # Trim GNOME defaults we don't want (privacy OS ships Tor/Mullvad browsers).
  environment.gnome.excludePackages = with pkgs; [
    gnome-tour
    epiphany # GNOME Web
    geary
  ];

  # Auto-login is OFF for a privacy OS; user authenticates each session.
  services.displayManager.autoLogin.enable = false;
}
