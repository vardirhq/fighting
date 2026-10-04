# Reproducibility

The web build derives the engine checkout from `.sindri-engine` and pins the wasm-pack version, making the runtime portion of the external build reproducible enough for CI to be a meaningful compatibility gate.
