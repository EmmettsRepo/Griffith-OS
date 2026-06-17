# GOS Tor gateway — the Whonix-style pattern as a tiny NixOS VM. The workstation
# VM routes through this one (host-only network on eth1) so ALL of its traffic
# exits via Tor, fail-closed. Run as a second VM alongside the workstation.
{ config, pkgs, lib, modulesPath, ... }:
{
  imports = [
    ../modules/gos-base.nix
    ../modules/gos-hardening.nix
    ./disk-vm.nix
  ];

  networking.hostName = lib.mkForce "gos-gateway";
  networking.usePredictableInterfaceNames = false; # eth0 = external, eth1 = internal

  # Internal network facing the workstation VM(s).
  networking.interfaces.eth1.ipv4.addresses = [
    { address = "10.152.152.10"; prefixLength = 24; }
  ];

  # Tor transparent proxy + DNS for the internal network.
  services.tor = {
    enable = true;
    client.enable = true;
    settings = {
      TransPort = [ { addr = "10.152.152.10"; port = 9040; } ];
      DNSPort = [ { addr = "10.152.152.10"; port = 5353; } ];
      VirtualAddrNetworkIPv4 = "10.192.0.0/10";
      AutomapHostsOnResolve = true;
    };
  };

  # Redirect internal TCP + DNS into Tor; nothing leaks in the clear.
  networking.nftables.enable = true;
  networking.nftables.ruleset = ''
    table ip gos_tor {
      chain prerouting {
        type nat hook prerouting priority dstnat;
        iifname "eth1" udp dport 53 redirect to :5353
        iifname "eth1" tcp dport 53 redirect to :5353
        iifname "eth1" tcp flags & (fin|syn|rst|ack) == syn redirect to :9040
      }
    }
  '';

  # The gateway forwards for the internal net.
  boot.kernel.sysctl."net.ipv4.ip_forward" = lib.mkForce 1;
  networking.firewall.enable = lib.mkForce false; # nftables ruleset governs this host
}
