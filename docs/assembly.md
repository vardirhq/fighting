# Web assembly step

After export, CI creates the export's `pkg/` directory, copies the wasm-pack output from the pinned Sindri checkout, and adds `.nojekyll`. Only that assembled directory is eligible for Pages upload.
