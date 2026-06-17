# Disk + boot for an installed GOS system (and for evaluating the config as a
# complete system). Generated images (nixos-generators) supply their own disk,
# so this file is only used by the `nixosConfigurations`, not the image packages.
{ config, pkgs, lib, modulesPath, ... }:
{
  imports = [ "${modulesPath}/profiles/qemu-guest.nix" ];

  boot.loader.systemd-boot.enable = true;
  boot.loader.efi.canTouchEfiVariables = false;
  boot.loader.timeout = 1;

  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };
  fileSystems."/boot" = {
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };
  swapDevices = [ ];
}
