#!/bin/bash
# Harbour OS 実行スクリプト

# SDL2ライブラリパス設定
export SDL2_FRAMEWORK_PATH=/opt/homebrew/opt/sdl2
export LIBRARY_PATH=/opt/homebrew/opt/sdl2/lib:$LIBRARY_PATH
export LD_LIBRARY_PATH=/opt/homebrew/opt/sdl2/lib:$LD_LIBRARY_PATH

# ビルドして実行（デフォルトバイナリ harbour_shell を起動）
cargo build
cargo run --bin harbour_shell
