#!/usr/bin/env bash
#
# scripts/dev.sh — launch the Griffith OS app in dev mode.
#
# Ensures cargo/rustup are on PATH, then runs the UI-local Tauri CLI from the
# repo root. Equivalent to `just dev`.

set -euo pipefail

# Make sure the rustup/cargo toolchain is available even in minimal shells.
if [ -f "$HOME/.cargo/env" ]; then
  # shellcheck disable=SC1091
  . "$HOME/.cargo/env"
fi

# Resolve the repo root from this script's location, regardless of CWD.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

TAURI_BIN="./ui/node_modules/.bin/tauri"

if [ ! -x "$TAURI_BIN" ]; then
  echo "error: $TAURI_BIN not found." >&2
  echo "Run 'cd ui && npm install' (or 'just ui-install') first." >&2
  exit 1
fi

exec "$TAURI_BIN" dev
