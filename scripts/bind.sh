#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)/.."

# Portable sed -i (macOS needs '', Linux doesn't)
sedi() {
  if sed --version >/dev/null 2>&1; then
    sed -i "$@"
  else
    sed -i '' "$@"
  fi
}

echo "Building contracts..."
(cd "$ROOT/contracts" && forge build)

echo "Generating Rust bindings..."
TMPDIR=$(mktemp -d)
(cd "$ROOT/contracts" && forge bind --alloy --bindings-path "$TMPDIR" --overwrite --crate-name bindings)

echo "Copying bindings to $ROOT/bindings/"
mkdir -p "$ROOT/bindings"
cp -r "$TMPDIR"/* "$ROOT/bindings/"
find "$ROOT/bindings/src" -name '*.rs' -exec sedi '/#\[automatically_derived\]/d' {} +
sedi 's/unused_imports/unused_imports, unused/' "$ROOT/bindings/src/lib.rs"
rm -rf "$TMPDIR"

echo "Verifying build..."
cargo build -p bindings

echo "Done."
