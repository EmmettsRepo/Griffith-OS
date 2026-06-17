# Guest integration for running GOS as a VM "over" macOS (UTM / Apple
# Virtualization.framework). Gives the cross-over the user wanted — but for a
# REAL OS: VirtioFS = files cross over, SPICE vdagent = clipboard crosses over.
{ config, pkgs, lib, ... }:
{
  services.qemuGuest.enable = true;
  services.spice-vdagentd.enable = true; # shared clipboard under SPICE/UTM

  # Shared folder from the host, mounted at /mnt/host (VirtioFS tag "share").
  # `nofail` so the system still boots if the host doesn't export it.
  fileSystems."/mnt/host" = {
    device = "share";
    fsType = "virtiofs";
    options = [ "nofail" "x-systemd.automount" ];
  };

  environment.systemPackages = with pkgs; [ spice-vdagent ];

  boot.initrd.availableKernelModules = [
    "virtio_pci"
    "virtio_blk"
    "virtio_scsi"
    "virtio_net"
    "virtiofs"
  ];
}
