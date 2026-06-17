# `gos-install` — a guided installer that lays GOS down on a disk (an external
# USB/SSD, or a PC's internal disk). Shipped ONLY in the live/installer ISO.
#
# It wipes the chosen disk, creates the two labeled partitions the portable host
# expects (`ESP` FAT32 + `nixos` ext4), and runs `nixos-install` against the GOS
# flake. Boot the ISO on the target PC, connect Wi-Fi, then run: sudo gos-install
{ config, pkgs, lib, ... }:
let
  gos-install = pkgs.writeShellApplication {
    name = "gos-install";
    runtimeInputs = with pkgs; [
      coreutils util-linux parted dosfstools e2fsprogs systemd nixos-install-tools
    ];
    text = ''
      FLAKE="github:EmmettsRepo/Griffith-OS?dir=os"

      if [ "$(id -u)" -ne 0 ]; then
        echo "Run as root:  sudo gos-install" >&2
        exit 1
      fi

      echo "=== GOS installer ==="
      echo
      echo "Available disks:"
      lsblk -dpno NAME,SIZE,MODEL,TRAN | grep -vE 'loop|/dev/sr' || true
      echo
      printf 'FULL device to ERASE and install GOS onto (e.g. /dev/sda): '
      read -r dev
      if [ ! -b "$dev" ]; then
        echo "Not a block device: $dev" >&2; exit 1
      fi

      echo
      echo "!! EVERYTHING on $dev will be PERMANENTLY ERASED."
      printf 'Re-type the device path to confirm: '
      read -r dev2
      if [ "$dev" != "$dev2" ]; then
        echo "Mismatch — aborting, nothing was changed." >&2; exit 1
      fi

      cfg="gos-portable-x86"
      printf 'Force ALL traffic through Tor by default (anonymous mode)? [y/N]: '
      read -r anon
      case "$anon" in
        y|Y) cfg="gos-portable-anon-x86" ;;
      esac

      echo
      echo ">> Partitioning $dev (GPT: 512 MiB ESP + rest root) ..."
      wipefs -a "$dev"
      parted -s "$dev" mklabel gpt
      parted -s "$dev" mkpart ESP fat32 1MiB 513MiB
      parted -s "$dev" set 1 esp on
      parted -s "$dev" mkpart nixos ext4 513MiB 100%
      udevadm settle

      # Partition node suffix: /dev/sda -> sda1 ; /dev/nvme0n1 -> nvme0n1p1
      case "$dev" in
        *[0-9]) p="p" ;;
        *) p="" ;;
      esac
      esp="''${dev}''${p}1"
      root="''${dev}''${p}2"

      echo ">> Formatting ($esp = ESP, $root = nixos) ..."
      mkfs.vfat -F32 -n ESP "$esp"
      mkfs.ext4 -F -L nixos "$root"
      udevadm settle

      echo ">> Mounting ..."
      mount "$root" /mnt
      mkdir -p /mnt/boot
      mount "$esp" /mnt/boot

      echo ">> Installing GOS ($cfg) from $FLAKE — this downloads + builds, give it time ..."
      nixos-install --flake "''${FLAKE}#''${cfg}" --no-root-passwd

      echo
      echo "=== Done. ==="
      echo "Remove the installer, reboot, and pick the GOS drive in your boot menu."
      echo "Log in as  gos / gos  — then run 'passwd' to change it, and 'gos-leaktest'."
    '';
  };
in
{
  environment.systemPackages = [ gos-install ];
}
