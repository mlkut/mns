#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)/.."

echo "Building contracts..."
(cd "$ROOT/contracts" && forge build)

echo "Generating Rust bindings..."
TMPDIR=$(mktemp -d)
(cd "$ROOT/contracts" && forge bind --alloy --bindings-path "$TMPDIR" --overwrite --crate-name bindings)

echo "Copying bindings to $ROOT/bindings/"
mkdir -p "$ROOT/bindings"
cp -r "$TMPDIR"/* "$ROOT/bindings/"
find "$ROOT/bindings/src" -name '*.rs' -exec sed -i '' '/#\[automatically_derived\]/d' {} +
sed -i '' 's/unused_imports/unused_imports, unused/' "$ROOT/bindings/src/lib.rs"
rm -rf "$TMPDIR"

echo "Verifying build..."
cargo build -p bindings

echo "Done."
