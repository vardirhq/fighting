# Fighting

Follow `AGENTS.md` first.

This repository is intentionally an external consumer of Sindri. Do not solve game-specific behavior by patching the engine from here. Gameplay belongs in Decay.

The web build is a real deployment target, not merely an export-format check. A valid Pages payload needs:
1. a Decay-clean project,
2. the Sindri browser host built with `wasm-pack`,
3. `sindri-export` run with `--base /fighting/`,
4. the generated `game/pkg` JS/Wasm copied into the exported `pkg/` directory,
5. verification before deployment.

Mobile controls should use Sindri's Decay input surfaces (`Stick`, `Touch`, `Pointer`, UI) rather than browser-specific JavaScript gameplay.
