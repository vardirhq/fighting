# Testing

CI is expected to fail rather than deploy when Decay does not type-check, the Wasm browser host does not build, the project cannot export, the Pages base is wrong, or the browser JS/Wasm files are missing. A browser smoke test is the next useful hardening step beyond these structural gates.
