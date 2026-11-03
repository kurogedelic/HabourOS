#!/bin/bash
# Harbour OS Web 版ビルド（Emscripten + Three.js 筐体）
# 必要: emcc, rustup target add wasm32-unknown-emscripten
set -e
cd "$(dirname "$0")/.."

cargo build --release --target wasm32-unknown-emscripten --bin harbour_shell

OUT=target/wasm32-unknown-emscripten/release/deps
cp "$OUT/harbour_shell.js" "$OUT/harbour_shell.wasm" "$OUT/harbour_shell.data" web/
echo "built: web/harbour_shell.{js,wasm,data}"
echo "serve: python3 -m http.server 8080 --directory web"
