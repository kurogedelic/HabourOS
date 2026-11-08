#!/usr/bin/env bash
# Local SDL2 window for the Cardputer Zero 320x170 HarbourOS interface.
# M5's LVGL simulator is a separate application; this exercises HarbourOS's
# exact same Rust UI and keyboard handling without claiming hardware emulation.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

unset HARBOUR_FBDEV HARBOUR_INPUT_DEVICE
export HARBOUR_WIDTH=320
export HARBOUR_HEIGHT=170
export HARBOUR_FULLSCREEN=0
export HARBOUR_THEME=assets/themes/cardputer-zero.yaml
export HARBOUR_CARDPUTER_KEYS=1

if [[ -n "${HARBOUR_BIN:-}" ]]; then
  exec "$HARBOUR_BIN" "$@"
fi
exec cargo run --bin harbour_shell -- "$@"
