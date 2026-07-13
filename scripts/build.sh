#!/usr/bin/env bash
set -euo pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 <deploy-dir>"
  echo "Example: $0 /opt/mns"
  exit 1
fi

DEPLOY_DIR="$1"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

mkdir -p "$DEPLOY_DIR"

# ── Install prerequisites if missing ──

if ! command -v cargo >/dev/null 2>&1; then
  echo "==> Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "==> Installing wasm-pack..."
  cargo install wasm-pack
fi

# ── Build everything ──

echo "==> Building WASM..."
(cd "$ROOT/mns-server/wasm" && wasm-pack build --target web --release)
mkdir -p "$ROOT/mns-server/static/mns-wasm"
cp "$ROOT/mns-server/wasm/pkg/mns_wasm_bg.wasm" "$ROOT/mns-server/static/mns-wasm/"
cp "$ROOT/mns-server/wasm/pkg/mns_wasm.js" "$ROOT/mns-server/static/mns-wasm/"

echo "==> Building server binary..."
(cd "$ROOT" && cargo build --release -p mns-server)

# ── Deploy ──

echo "==> Copying to $DEPLOY_DIR..."
cp "$ROOT/target/release/mns-server" "$DEPLOY_DIR/"
cp -r "$ROOT/mns-server/static" "$DEPLOY_DIR/"

echo "==> Done."
echo "Binary: $DEPLOY_DIR/mns-server"
echo "Run:    $DEPLOY_DIR/mns-server --listen 3000"
