# GOS anti-tracking core — the heart of GOS.
#
# Two always-on protections (independent of the Tor toggle):
#   * MAC-address randomization, so your hardware address isn't a network tracker.
#   * No clear-NTP time sync, which would leak your real IP / enable correlation.
#
# Plus an opt-in `gos.torEverything.enable` profile that forces the WHOLE host
# through Tor, fail-closed (Tails-style, single VM) — the difference between
# "a privacy app" and a privacy OS. It reuses the same transparent-proxy idea as
# the two-VM gateway (hosts/gos-gateway.nix), applied locally:
#
#   nat OUTPUT  : all new TCP  -> Tor TransPort 9040
#                 all DNS (:53) -> Tor DNSPort  5353   (leak-proofing)
#   filter OUTPUT (policy DROP): only loopback, the `tor` user, DHCP, and the
#                 redirected-into-Tor packets may leave — everything else is
#                 dropped, so if Tor stops, traffic is BLOCKED, never clear.
#
# Honest scope: single-host transparent Tor is Tails-class. The two-VM gateway
# stays the higher-assurance option (a compromised workstation still can't see
# your real IP there). GOS hides traffic; it does not make you invisible.
{ config, pkgs, lib, ... }:
let
  cfg = config.gos.torEverything;
in
{
  options.gos.torEverything.enable = lib.mkEnableOption ''
    forcing ALL host traffic through Tor with a fail-closed kill switch
    (single-VM transparent torification). Off by default so GOS stays usable;
    turn it on for "anonymous mode"'';

  config = lib.mkMerge [
    # ---- Always-on anti-tracking (both modes) -------------------------------
    {
      # Randomize MAC on scan and per-connection so the NIC isn't a stable ID.
      networking.networkmanager.enable = lib.mkDefault true;
      networking.networkmanager.wifi.macAddress = "random";
      networking.networkmanager.wifi.scanRandMacAddress = true;
      networking.networkmanager.ethernet.macAddress = "random";

      # Clear-text NTP reveals your real IP and is a correlation vector. Disable
      # it; the VM gets time from the hypervisor clock. (Follow-up: Tor-based
      # time à la Whonix sdwdate.)
      services.timesyncd.enable = lib.mkForce false;
    }

    # ---- Anonymous mode: force everything through Tor, fail-closed ----------
    (lib.mkIf cfg.enable {
      # Marker so `gos-leaktest` (and the user) can tell anonymous mode is on
      # without needing root to inspect nftables.
      environment.etc."gos-anonymous".text = "Griffith OS anonymous mode: all traffic forced through Tor, fail-closed.\n";

      # DNS goes ONLY through Tor's DNSPort. systemd-resolved forwards every
      # query to 127.0.0.1:5353 (Tor), with DoT/DNSSEC off so nothing is sent to
      # a third-party resolver tied to your real IP.
      services.resolved = {
        enable = true;
        dnssec = "false";
        dnsovertls = "false";
        fallbackDns = [ ];
        domains = [ "~." ];
        extraConfig = ''
          DNS=127.0.0.1:5353
        '';
      };

      # NetworkManager's connectivity ping would try to reach a URL in the clear
      # (bypassing Tor). The kill switch drops it anyway; disable it to be clean.
      networking.networkmanager.settings.connectivity.interval = 0;

      # Replace the iptables firewall with an nftables ruleset that both
      # default-denies inbound AND torifies + fail-closes outbound.
      networking.firewall.enable = lib.mkForce false;
      networking.nftables.enable = true;
      networking.nftables.ruleset = ''
        # Transparent torification + kill switch for the local host.
        table ip gos_anon_nat {
          chain output {
            type nat hook output priority -100; policy accept;

            # Never redirect Tor's own traffic (it must reach real relays) …
            meta skuid "tor" return
            # … nor loopback, nor anything already heading into local Tor ports.
            oifname "lo" return
            ip daddr 127.0.0.0/8 return
            # Don't torify RFC1918 LAN destinations (Tor can't reach them); the
            # filter chain below DROPs them instead, so the LAN can't be probed.
            ip daddr 10.0.0.0/8 return
            ip daddr 172.16.0.0/12 return
            ip daddr 192.168.0.0/16 return
            ip daddr 169.254.0.0/16 return

            # Everything else: DNS into Tor's DNSPort, all other TCP into TransPort.
            udp dport 53 redirect to :5353
            tcp dport 53 redirect to :5353
            meta l4proto tcp redirect to :9040
          }
        }

        table ip gos_anon_filter {
          # Default-deny inbound (replaces the stock firewall).
          chain input {
            type filter hook input priority 0; policy drop;
            iifname "lo" accept
            ct state established,related accept
          }

          # Fail-closed egress: only these may leave the box.
          chain output {
            type filter hook output priority 0; policy drop;
            oifname "lo" accept
            ct state established,related accept
            # Tor itself reaching its guards/relays (your IP sees the guard — normal).
            meta skuid "tor" accept
            # DHCP so the VM/PC can obtain an address at all.
            udp sport 68 udp dport 67 accept
            # Redirected DNS/TCP — now destined for local Tor (127.0.0.1) — pass.
            ip daddr 127.0.0.0/8 accept
            # Anything else (clearnet, LAN, stray DNS) is dropped → no leak.
          }
        }
      '';
    })
  ];
}
