# Export responsibility

`sindri-export` packages project content and writes the browser page. The generic JS/Wasm host is built separately from Sindri's `game` crate and must be copied into the exported directory. The workflow now makes that distinction explicit.
