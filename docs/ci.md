# CI gates

The compatibility workflow validates the external Sindri project. The Pages workflow additionally validates the actual browser payload. A green export that only proves `index.html` exists is not sufficient; the JS/Wasm host must be present because the page imports it at runtime.
