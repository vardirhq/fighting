# Why the Wasm build is separate

Exporting a Sindri project serializes what the game contains. Building the `game` crate produces the code that executes those contents in a browser. Pages needs both.
