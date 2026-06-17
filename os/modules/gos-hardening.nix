# GOS hardening: pragmatic, dev-friendly (not the full `hardened` profile, which
# breaks user namespaces / many dev tools). Firewall default-deny, sysctl, AppArmor.
{ config, pkgs, lib, ... }:
{
  networking.firewall = {
    enable = true;
    allowedTCPPorts = [ ];
    allowedUDPPorts = [ ];
  };

  boot.kernel.sysctl = {
    "kernel.kptr_restrict" = 2;
    "kernel.dmesg_restrict" = 1;
    "net.ipv4.conf.all.rp_filter" = 1;
    "net.ipv4.conf.default.rp_filter" = 1;
    # No IPv6 by default — avoids IPv6 leaks (mirrors the app's IPv4 DoH choice).
    "net.ipv6.conf.all.disable_ipv6" = 1;
    "net.ipv6.conf.default.disable_ipv6" = 1;
  };

  # Don't write core dumps (they can contain secrets).
  systemd.coredump.enable = false;

  security.protectKernelImage = true;
  security.apparmor.enable = true;

  # No remote login surface by default.
  services.openssh.enable = lib.mkDefault false;
}
