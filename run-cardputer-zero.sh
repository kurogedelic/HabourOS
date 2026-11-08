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

# The Cardputer Zero display is framebuffer-native, not Wayland-native.
# SDL2 renders to an in-memory software canvas; HarbourOS writes that canvas
# to the kernel fbdev driver. Set SDL_VIDEODRIVER=wayland explicitly only when
# testing an external compositor. This never installs or replaces a service.
export SDL_VIDEODRIVER="${SDL_VIDEODRIVER:-dummy}"
if [[ "$SDL_VIDEODRIVER" == "dummy" ]]; then
  export HARBOUR_FBDEV="${HARBOUR_FBDEV:-/dev/fb0}"
  export HARBOUR_INPUT_DEVICE="${HARBOUR_INPUT_DEVICE:-/dev/input/by-path/platform-3f804000.i2c-event}"
elif [[ -n "${HARBOUR_FBDEV:-}" ]]; then
  echo "HARBOUR_FBDEV requires SDL_VIDEODRIVER=dummy." >&2
  exit 1
fi
export HARBOUR_WIDTH="${HARBOUR_WIDTH:-320}"
export HARBOUR_HEIGHT="${HARBOUR_HEIGHT:-170}"
export HARBOUR_FULLSCREEN="${HARBOUR_FULLSCREEN:-1}"
export HARBOUR_THEME="${HARBOUR_THEME:-assets/themes/cardputer-zero.yaml}"

if [[ "$SDL_VIDEODRIVER" == "wayland" && -z "${WAYLAND_DISPLAY:-}" ]]; then
  echo "Wayland selected but WAYLAND_DISPLAY is unset." >&2
  echo "Use SDL_VIDEODRIVER=dummy for direct fbdev mode or set the real Wayland session." >&2
  exit 1
fi
if [[ "$SDL_VIDEODRIVER" == "dummy" ]]; then
  if [[ ! -e "$HARBOUR_FBDEV" ]]; then
    echo "Framebuffer not found: $HARBOUR_FBDEV" >&2
    exit 1
  fi
  if [[ ! -w "$HARBOUR_FBDEV" ]]; then
    echo "Framebuffer not writable: $HARBOUR_FBDEV (check device permissions)" >&2
    exit 1
  fi
  if [[ ! -r "$HARBOUR_INPUT_DEVICE" ]]; then
    echo "Warning: keyboard unavailable: $HARBOUR_INPUT_DEVICE" >&2
  fi
fi

DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
mkdir -p "$DATA_HOME/harbour-os"
export HARBOUR_VFS_PATH="${HARBOUR_VFS_PATH:-$DATA_HOME/harbour-os/harbour_vfs.yaml}"
export HARBOUR_IMPORT_DIR="${HARBOUR_IMPORT_DIR:-$DATA_HOME/harbour-os/import}"
export HARBOUR_EXPORT_DIR="${HARBOUR_EXPORT_DIR:-$DATA_HOME/harbour-os/export}"

exec "$BIN" "$@"
