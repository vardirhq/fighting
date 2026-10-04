# Web deployment

Fighting is an external Sindri project and is published as a GitHub Pages project site at `/fighting/`.

The deployment has two build products:

- `sindri-export` produces `index.html`, the asset manifest, and hashed project assets.
- `wasm-pack build game --target web` in the pinned Sindri checkout produces the generic browser host (`pkg/sindri_causeway.js` and its Wasm module).

The browser host must be copied into the export before uploading the Pages artifact. The export must also be created with `--base /fighting/`; otherwise relative host imports resolve against the organization-site root.

CI verifies the Decay source, builds the Wasm host, exports with the Pages base, and checks that the page, manifest, JavaScript host, Wasm host, and expected asset kinds are present.
