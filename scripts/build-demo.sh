#!/usr/bin/env bash
set -e

echo "Building demo..."

# Build the docfind wasm first if needed
echo "Building docfind wasm..."
./scripts/build.sh

# Generate index from documents.json
echo "Generating index..."
node ./demo/build_index/main.js

# Build search demo
echo "Building search demo..."
npx --yes esbuild --bundle wasm/search/pkg/docfind.js --format=esm --outfile=demo/search/docfind.js --allow-overwrite
cp wasm/search/pkg/docfind_bg.wasm demo/search/docfind_bg.wasm

cp demo/build_index/index.bin demo/search/index.bin

echo "Demo build completed successfully!"
echo ""
echo "Generated files:"
ls -lh demo/search
