# Griffith OS — task recipes.
#
# `just` is OPTIONAL. Every recipe below is a thin wrapper around plain
# cargo / npm / Tauri commands you can run by hand. Install it from
# https://github.com/casey/just if you want the shorthand.
#
# All recipes are meant to be run from the repo ROOT.

# Default: list available recipes.
default:
    @just --list

# Run the full Tauri app in dev mode (Rust core + Vite UI, hot reload).
# Uses the UI-local Tauri CLI installed by `just ui-install`.
dev:
    ./ui/node_modules/.bin/tauri dev

# Produce a production build of the native app.
build:
    ./ui/node_modules/.bin/tauri build

# Static checks: Rust workspace + TypeScript types.
check:
    cargo check --workspace
    cd ui && npx tsc --noEmit

# Run the Rust test suite across the workspace.
test:
    cargo test --workspace

# Exercise the PrivacyEngine end-to-end (bootstrap Tor, fetch exit IP, leak test).
engine-check:
    cargo run -p gos-privacy --example check -- --tor

# Install UI dependencies (also provides the local Tauri CLI).
ui-install:
    cd ui && npm install
