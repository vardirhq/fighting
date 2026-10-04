# Pages regression

The exact failure that prompted this change is now represented by CI assertions: the base must be `/fighting/`, and both `sindri_causeway.js` and its Wasm module must exist in `pkg/` before deployment.
