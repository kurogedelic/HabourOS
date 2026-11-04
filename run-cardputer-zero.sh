#!/usr/bin/env bash
# Experimental Cardputer Zero launcher. Run from an unpacked release bundle.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"  # assets/ and fonts/ are loaded relative to the working directory.

if [[ -n "${HARBOUR_BIN:-}" ]]; then
  BIN="$HARBOUR_BIN"
elif [[ -x "$ROOT/harbour_shell" ]]; then
  BIN="$ROOT/harbour_shell"
else
  BIN="$ROOT/target/release/harbour_shell"
fi

if [[ ! -x "$BIN" ]]; then
  echo "HarbourOS executable not found: $BIN" >&2
  echo "Use the ARM64 release bundle or build: cargo build --locked --release --bin harbour_shell" >&2
  exit 1
fi

# Cardputer Zero is AArch64 Linux, not the ESP32-based Cardputer.
# The stock display is a framebuffer; SDL2 does not support SDL1's 'fbcon'.
# This first-stage launcher tries the available Wayland compositor.
export SDL_VIDEODRIVER="${SDL_VIDEODRIVER:-wayland}"
export HARBOUR_WIDTH="${HARBOUR_WIDTH:-320}"
export HARBOUR_HEIGHT="${HARBOUR_HEIGHT:-170}"
export HARBOUR_FULLSCREEN="${HARBOUR_FULLSCREEN:-1}"

if [[ "$SDL_VIDEODRIVER" == "wayland" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  echo "Warning: WAYLAND_DISPLAY is not set; a running Wayland session is required." >&2
  echo "From SSH, XDG_RUNTIME_DIR and WAYLAND_DISPLAY must point to the actual user session." >&2
fi

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
mkdir -p "$DATA_HOME/harbour-os"
export HARBOUR_VFS_PATH="${HARBOUR_VFS_PATH:-$DATA_HOME/harbour-os/harbour_vfs.yaml}"
export HARBOUR_IMPORT_DIR="${HARBOUR_IMPORT_DIR:-$DATA_HOME/harbour-os/import}"
export HARBOUR_EXPORT_DIR="${HARBOUR_EXPORT_DIR:-$DATA_HOME/harbour-os/export}"

exec "$BIN" "$@"
