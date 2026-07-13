#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")" && pwd)/.."

if sed --version >/dev/null 2>&1; then
  SED_I=(sed -i)
else
  SED_I=(sed -i '')
fi

echo "Building contracts..."
(cd "$ROOT/contracts" && forge build)

echo "Generating Rust bindings..."
TMPDIR=$(mktemp -d)
(cd "$ROOT/contracts" && forge bind --alloy --bindings-path "$TMPDIR" --overwrite --crate-name bindings)

echo "Copying bindings to $ROOT/bindings/"
mkdir -p "$ROOT/bindings"
cp -r "$TMPDIR"/* "$ROOT/bindings/"
find "$ROOT/bindings/src" -name '*.rs' -exec "${SED_I[@]}" '/#\[automatically_derived\]/d' {} +
"${SED_I[@]}" 's/unused_imports/unused_imports, unused/' "$ROOT/bindings/src/lib.rs"
rm -rf "$TMPDIR"

echo "Verifying build..."
cargo build -p bindings

echo "Done."
