# Web workflow

The Pages workflow intentionally mirrors Sindri's external-project deployment pattern.

`sindri-export` writes the page, manifest, and project assets. It does **not** replace building the browser host. `wasm-pack build game --target web` produces the `pkg/sindri_causeway.js` and WebAssembly files that the exported page dynamically imports. Those files are copied into the export before it is uploaded.

Because this repository is a GitHub Pages project site, the export base is `/fighting/`, not `/`. Root-based exports will request `/pkg/...` from `vardirhq.github.io` instead of `vardirhq.github.io/fighting/`.
