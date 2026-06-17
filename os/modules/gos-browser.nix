# GOS browsers: ship the real hardened, anti-fingerprinting browsers — never a
# custom webview (a bespoke browser makes users a UNIQUE fingerprint). Mullvad
# Browser pairs with GOS's own proxy/Tor; Tor Browser adds the Tor network.
{ config, pkgs, lib, ... }:
{
  environment.systemPackages = with pkgs; [
    mullvad-browser
    tor-browser
  ];
}
