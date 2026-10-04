# Fighting agent notes

Fighting is an external Sindri project. Keep gameplay in Decay and project assets, not in engine Rust.

Before changing the project:
- Read this file and `CLAUDE.md`.
- Treat `.sindri-engine` as the engine revision the project must compile and export against.
- Use the Decay surface documented by that pinned Sindri revision.

Before pushing gameplay or deployment changes:
- Run the Decay batch checker against the project.
- Build the browser host for `wasm32-unknown-unknown` when changing web deployment.
- Export with the correct non-root Pages base (`/fighting/`).
- Verify the export contains the browser JS/Wasm host as well as the manifest and project assets.

GitHub Actions is the execution gate when working through the connector.
