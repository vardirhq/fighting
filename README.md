# Fighting

Tiny external Sindri project used to prove a family fighting-game prototype outside the engine repository.

## Run locally

Keep this repository and `vardirhq/sindri-engine` as sibling checkouts, then from the Sindri checkout run:

```bash
cargo run --package sindri-editor -- ../fighting
```

The project is pinned to the Sindri revision in `.sindri-engine`, following the same external-project pattern as Mujaffa Remaster.

## Current slice

- Bedroom arena background from the committed artwork.
- Agnes as the playable character.
- `A` / `D` or left/right arrows move horizontally.
- `W` / `S` or up/down arrows move on the shallow arena depth.
- Idle and walk animations switch automatically.
- Moving left mirrors the right-facing sprite art.

The PixelLab sheets are currently treated as 3x3 grids. If a sheet's authored layout differs from that, adjust its sibling `.sheet` document rather than baking crop logic into gameplay.
