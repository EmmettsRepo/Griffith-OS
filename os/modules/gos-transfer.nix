# GOS encrypted device-to-device transfer. `croc` and `magic-wormhole` give
# instant code-word sends today; a persistent `iroh`-based GOS<->GOS mesh comes
# next (add via an overlay once pinned to a 1.x release).
{ config, pkgs, lib, ... }:
{
  environment.systemPackages = with pkgs; [
    croc
    magic-wormhole
  ];
}
