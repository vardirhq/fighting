# External project boundary

Fighting consumes the Sindri revision in `.sindri-engine`. Gameplay and controls belong in Decay. The engine checkout in CI exists to validate the project, build the generic web host, and export it; Fighting should not acquire engine-specific Rust gameplay code.
