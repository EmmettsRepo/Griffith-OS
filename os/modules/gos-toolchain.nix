# GOS developer toolchain — "all languages pre-installed", the Nix way.
# Reproducible and baked into the image; no Homebrew/mise needed.
{ config, pkgs, lib, ... }:
{
  environment.systemPackages = with pkgs; [
    # core
    git curl wget jq ripgrep fd htop tree unzip
    gcc gnumake pkg-config
    # languages
    nodejs_22 # includes npm + npx
    python3
    go
    rustup
    # editors / terminals
    vim
    neovim
  ];

  programs.git.enable = true;
}
