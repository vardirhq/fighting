# Fighting

Tiny external Sindri project used to prove a family fighting-game prototype outside the engine repository.

## Run locally

Keep this repository and `vardirhq/sindri-engine` as sibling checkouts, then from the Sindri checkout run:

```bash
cargo run --package sindri-editor -- ../fighting
```

The project is pinned to the Sindri revision in `.sindri-engine`, following the same external-project pattern as Mujaffa Remaster.

## Current slice

- Agnes and Dad share a Decay fighter controller, animation timings and shadows.
- Dad uses the new 256-pixel sheets; his left attack mirrors the right attack.
- Agnes: A/D or arrows to move horizontally; Space or the touch button to attack.
- Dad: J/L to move; I to attack.
- Fighters face their opponent; retreating plays the facing walk loop backwards.
- Movement starts/stops immediately; three-frame turns last 90 ms without pausing movement.
- Walk loops use 70 ms per frame at 5 units/s, scaling with actual movement speed.
- Held input at a room boundary returns to idle when movement is blocked.
- Movement stays on the carpet plane.

See [fighter behavior and verification](docs/fighters.md).

The PixelLab sheets are currently treated as 3x3 grids. If a sheet's authored layout differs from that, adjust its sibling `.sheet` document rather than baking crop logic into gameplay.
