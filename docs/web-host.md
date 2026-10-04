# Web host build

The pinned Sindri checkout builds the generic host with:

`wasm-pack build game --target web --release --out-dir pkg`

The resulting `game/pkg` contents are copied into the exported Fighting directory before upload.
