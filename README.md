# Fighting

Tiny external Sindri project used to prove a family fighting-game prototype outside the engine repository.

## Run locally

Keep this repository and `vardirhq/sindri-engine` as sibling checkouts, then from the Sindri checkout run:

```bash
cargo run --package sindri-editor -- ../fighting
```

The project is pinned to the Sindri revision in `.sindri-engine`, following the same external-project pattern as Mujaffa Remaster.

## Current slice

- Play Agnes against an AI Dad in first-to-three balance battles.
- A/D or arrows to move, Space to attack, K to dodge; touch has movement, attack and dodge.
- P switches to practice mode with manual Dad controls; R restarts a match.
- Agnes and Dad share a Decay fighter controller, animation timings and shadows.
- Dad uses the new 256-pixel sheets; his left attack mirrors the right attack.
- Agnes: A/D or arrows to move horizontally; Space or the touch button to attack.
- Dad: J/L to move; I to attack.
- Fighters face their opponent; retreating plays the facing walk loop backwards.
- Movement starts/stops immediately; three-frame turns last 90 ms without pausing movement.
- Walk loops use 70 ms per frame at 5 units/s, scaling with actual movement speed.
- Held input at a room boundary returns to idle when movement is blocked.
- Movement stays on the carpet plane.

See [combat rules](docs/combat.md) and [fighter animation and verification](docs/fighters.md).

The PixelLab sheets are currently treated as 3x3 grids. If a sheet's authored layout differs from that, adjust its sibling `.sheet` document rather than baking crop logic into gameplay.

## Repository layout

- `textures/agnes/`: Agnes sprites and adjacent sheet definitions, including unused start/stop source artwork.
- `textures/dad/`: active 256-pixel Dad sprites and sheet definitions.
- `textures/dad/archive/`: earlier Dad artwork, preserved for reference.
- `textures/backgrounds/` and `textures/effects/`: room art and effect textures.
- `audio/` and `fonts/`: impact sound and licensed HUD font.
- `main.scene.json`: scene setup; `scripts/`: gameplay; `tests/`: runtime and browser checks; `docs/`: behavior and rendering notes.
