#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$ROOT_DIR/dist"
BUNDLE_NAME="harbour-os-linux-x86_64"
STAGE_DIR="$DIST_DIR/$BUNDLE_NAME"

cd "$ROOT_DIR"

cargo build --release --bin harbour_shell

rm -rf "$STAGE_DIR"
mkdir -p "$STAGE_DIR"

cp "$ROOT_DIR/target/release/harbour_shell" "$STAGE_DIR/"
cp "$ROOT_DIR/run.sh" "$STAGE_DIR/"
cp "$ROOT_DIR/README.md" "$STAGE_DIR/"
cp -R "$ROOT_DIR/assets" "$STAGE_DIR/"
cp -R "$ROOT_DIR/fonts" "$STAGE_DIR/"

cat > "$STAGE_DIR/launch.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
export LD_LIBRARY_PATH="${LD_LIBRARY_PATH:-}:$(pwd)"
exec ./harbour_shell "$@"
EOF
chmod +x "$STAGE_DIR/launch.sh"

mkdir -p "$DIST_DIR"
tar -C "$DIST_DIR" -czf "$DIST_DIR/$BUNDLE_NAME.tar.gz" "$BUNDLE_NAME"

echo "Created $DIST_DIR/$BUNDLE_NAME.tar.gz"
