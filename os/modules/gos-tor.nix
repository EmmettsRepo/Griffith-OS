# GOS networking privacy on the workstation. The local Tor client provides:
#   * a SOCKS proxy (127.0.0.1:9050) for opt-in app torification, and
#   * a transparent TransPort (9040) + DNSPort (5353) that the anti-tracking
#     profile (modules/gos-anti-tracking.nix) redirects ALL traffic into.
# Without that profile GOS stays usable: clearnet works, DNS is encrypted (DoT),
# and you torify per-app via SOCKS. With `gos.torEverything.enable = true` the
# whole host is forced through Tor, fail-closed (Tails-style, single VM). The
# two-VM Whonix gateway (hosts/gos-gateway.nix) remains the higher-assurance path.
{ config, pkgs, lib, ... }:
let
  torEverything = config.gos.torEverything.enable;
in
{
  services.tor = {
    enable = true;
    client.enable = true; # SOCKS proxy on 127.0.0.1:9050
    settings = {
      # Transparent-proxy targets the anti-tracking nftables redirect points at.
      # Harmless (loopback-only) when torEverything is off; live when it's on.
      TransPort = [ { addr = "127.0.0.1"; port = 9040; } ];
      DNSPort = [ { addr = "127.0.0.1"; port = 5353; } ];
      VirtualAddrNetworkIPv4 = "10.192.0.0/10";
      AutomapHostsOnResolve = true;
    };
  };

  # Usable (non-torified) mode: encrypted DNS over TLS so the local network can't
  # read lookups. When the whole host is torified, DNS instead goes through Tor's
  # DNSPort (see gos-anti-tracking.nix), so this clear DoT path is turned OFF —
  # otherwise lookups would leak to Cloudflare/Quad9 tied to your real IP.
  services.resolved = lib.mkIf (!torEverything) {
    enable = true;
    dnssec = "true";
    dnsovertls = "true";
    fallbackDns = [
      "1.1.1.1#cloudflare-dns.com"
      "9.9.9.9#dns.quad9.net"
    ];
  };
  networking.nameservers = lib.mkIf (!torEverything) [
    "1.1.1.1#cloudflare-dns.com"
    "9.9.9.9#dns.quad9.net"
  ];

  environment.systemPackages = with pkgs; [ tor torsocks ];
}
