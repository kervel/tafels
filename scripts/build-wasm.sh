#!/usr/bin/env bash
set -euo pipefail

# Build the WASM client and place dist/ at the repo root for Docker context
cd "$(dirname "$0")/../client"
trunk build --release
# Clean and copy dist to repo root so Dockerfile can COPY it
rm -rf ../dist
cp -r dist ../dist
echo "WASM client built → dist/"
