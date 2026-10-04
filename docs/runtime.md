# Runtime packaging

The exported page is a shell around Sindri's generic browser runtime. The runtime JavaScript loads its Wasm module and the project's generated manifest/assets. Shipping the shell without `pkg/` is therefore analogous to shipping a desktop shortcut without the executable, an impressively useless success state.
