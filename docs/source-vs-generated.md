# Source versus generated web files

Fighting commits game source and deployment instructions. The Sindri browser package is generated from the pinned engine during CI and copied into the export. Committing generated Wasm/JS here would duplicate engine artifacts and make the pin less meaningful.
