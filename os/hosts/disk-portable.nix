# Bare-metal disk + boot for a PORTABLE GOS install — an external USB/SSD you plug
# into a PC (e.g. a ThinkPad X1 Carbon), or a PC's own disk. Unlike hosts/disk-vm.nix
# this targets real hardware:
#   * boots off USB / NVMe / SATA,
#   * Intel CPU microcode + redistributable firmware (Wi-Fi/GPU), and
#   * a REMOVABLE-media GRUB (writes EFI/BOOT/BOOTX64.EFI) so the drive boots on
#     any UEFI x86 machine straight from the boot menu — no NVRAM entry needed.
#
# It mounts by LABEL (`nixos` root, `ESP` boot), so the same config works on any
# disk; `gos-install` creates those two labeled partitions for you.
{ config, pkgs, lib, ... }:
{
  # Boot off USB/NVMe/SATA + Intel laptop essentials.
  boot.initrd.availableKernelModules = [
    "xhci_pci" "ehci_pci" "ahci" "nvme"
    "usb_storage" "uas" "sd_mod" "sdhci_pci" "thunderbolt"
  ];
  boot.kernelModules = [ "kvm-intel" ];

  # Removable-media GRUB so the drive boots on any UEFI machine.
  boot.loader.grub = {
    enable = true;
    efiSupport = true;
    efiInstallAsRemovable = true;
    device = "nodev";
  };
  boot.loader.efi.canTouchEfiVariables = false;
  boot.loader.timeout = 3;

  fileSystems."/" = {
    device = "/dev/disk/by-label/nixos";
    fsType = "ext4";
  };
  fileSystems."/boot" = {
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };
  swapDevices = [ ];

  # Real-hardware niceties.
  hardware.enableRedistributableFirmware = true; # Intel Wi-Fi / GPU firmware
  hardware.cpu.intel.updateMicrocode = true;
  services.fstrim.enable = true; # SSD longevity
}
