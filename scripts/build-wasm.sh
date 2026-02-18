#!/usr/bin/env bash
set -euo pipefail

BINARYEN_VERSION=126
BINARYEN_DIR="/tmp/binaryen-version_${BINARYEN_VERSION}"
WASM_OPT="${BINARYEN_DIR}/bin/wasm-opt"

# Download binaryen if not cached
if [ ! -x "$WASM_OPT" ]; then
    echo "Downloading binaryen v${BINARYEN_VERSION}..."
    curl -sL "https://github.com/WebAssembly/binaryen/releases/download/version_${BINARYEN_VERSION}/binaryen-version_${BINARYEN_VERSION}-x86_64-linux.tar.gz" \
        | tar xz -C /tmp
fi

# Build the WASM client
cd "$(dirname "$0")/../client"
trunk build --release

# Optimize with known-good wasm-opt
for f in dist/*_bg.wasm; do
    echo "Optimizing $f with wasm-opt v${BINARYEN_VERSION}..."
    "$WASM_OPT" -Oz -o "$f" "$f"
done

# Clean and copy dist to repo root so Dockerfile can COPY it
rm -rf ../dist
cp -r dist ../dist
echo "WASM client built → dist/"
