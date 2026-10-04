# Base path

The export base is `/fighting/`, including the trailing slash. Sindri's exporter explicitly relies on that slash so relative `pkg` imports resolve inside the project path rather than at the host root.
