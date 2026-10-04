# Pages path

GitHub serves this project below the organization site at `/fighting/`. Root-relative `/pkg/...` requests therefore target the wrong location. `sindri-export --base /fighting/` writes the correct base into the exported page so `./pkg/...` resolves inside the Fighting project site.
