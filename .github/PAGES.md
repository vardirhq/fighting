# Pages deployment contract

Fighting is served at `/fighting/` on the organization Pages host.

The exported `index.html` dynamically imports `./pkg/sindri_causeway.js`, so the export must carry `<base href="/fighting/">` and the workflow must copy Sindri's wasm-pack output into `pkg/`. Verifying only `index.html` is insufficient: that allowed a deployment to succeed while the browser host module was absent and the page could not start.
