#!/usr/bin/env bash
set -e

# Build wasm/search (for web to search the index)
wasm-pack build wasm/search --out-name docfind --release --target web

# Build wasm/search (for ssr to search the index)
wasm-pack build wasm/search --out-name docfind --out-dir pkg-node --release --target nodejs
# Embed the wasm in a js file as a base64 string using node
node scripts/embed.js wasm/search/pkg-node/docfind_bg.wasm wasm/search/pkg-node/docfind.js

# Build wasm/build_index (for node to build the index)
wasm-pack build wasm/build_index --out-name docfind_build_index --release --target nodejs
# Embed the wasm in a js file as a base64 string using node
node scripts/embed.js wasm/build_index/pkg/docfind_build_index_bg.wasm wasm/build_index/pkg/docfind_build_index.js
