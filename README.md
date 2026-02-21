# Docfind

Docfind is a fast, client-side full-text search library built with Rust and WebAssembly. It provides fuzzy search capabilities with minimal memory footprint, making it ideal for embedding search functionality directly in web applications.

This is a fork of [microsoft/docfind](https://github.com/microsoft/docfind).

## Features

- **Client-side search**: Runs entirely in the browser using WebAssembly - no server required
- **Fuzzy matching**: Supports typo-tolerant search using Levenshtein distance
- **Prefix search**: Automatically includes prefix matches in results
- **Weighted results**: Support for weighted search terms
- **Small footprint**: Uses Finite State Transducers (FST) for efficient indexing
- **Fast**: Optimized for quick search response times

## Architecture

- `core/`: Core search library written in Rust
- `wasm/search/`: WebAssembly bindings for the search component
- `wasm/build_index/`: WebAssembly bindings for building search indexes

## Usage

### Building the WASM Modules

```bash
# Build the search WASM module (for web)
cd wasm/search && wasm-pack build --target web

# Build the index builder WASM module (for Node.js)
cd wasm/build_index && wasm-pack build --target nodejs
```

Or use the provided build scripts:

```bash
./scripts/build.sh
```

### Building the Demo

```bash
./scripts/build-demo.sh
```

### Running the Demo

Serve the `demo/search` directory with any web server:

```bash
# Using Python
python3 -m http.server 8000 --directory demo/search

# Using Node.js
npx serve demo/search
```

Then open http://localhost:8000 in your browser.

## API

The search module (`wasm/search`) is designed for web browsers, while the index builder (`wasm/build_index`) is designed for Node.js.

### Building an Index (Node.js)

```javascript
import { buildIndex } from './build_index.js';

const items = [
  {
    id: "item-1",
    searchTerms: [
      { type: "raw", value: "Rust programming guide", weight: 10 },
      { type: "tokens", value: ["rust", "programming"], weight: 5 }
    ]
  }
];

const indexData = await buildIndex(items);
// Save indexData to a file for later use
```

### Searching (Web)

```javascript
import { loadIndex, search } from './docfind.js';

// Load pre-built index
await loadIndex('index.bin');

// Search
const results = await search('rust', 10);
// Returns array of item IDs sorted by relevance
```

## License

MIT License - See [LICENSE](LICENSE) for details.
