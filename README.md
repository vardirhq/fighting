# Fighting

A standalone Sindri fighting-game combat lab. Agnes is a small pink shape;
Dad is a taller blue shape. The bedroom background stays while we work out
which movement, attacks and reactions actually deserve authored graphics.

Play against AI Dad: first to three rounds. Move with A/D, jump W, crouch S,
hold left Shift to guard, Space to jab (kick in the air), E to sweep, K to dodge.
Touch has a drag stick and JAB, SWEEP, JUMP, GUARD and DODGE buttons.
Jump sweeps, defend high or low, bait misses and punish recovery. Energy limits
attacks and defense. R restarts; P opens manual training.

See [combat rules and timings](docs/combat.md).

## Run locally

Keep this repository and `vardirhq/sindri-engine` as sibling checkouts. Check out
the engine revision in `.sindri-engine`, then from the engine directory run:

```bash
cargo run --package sindri-editor -- ../fighting
```

## Layout

- `main.scene.json`: background, shape fighters, combat HUD and touch controls.
- `scripts/`: artwork-independent combat and round rules in Decay.
- `tests/`: actual runtime combat regressions and desktop/mobile browser checks.
- `docs/combat.md`: current controls, design decisions, balance and limitations.
- `docs/legacy/`: historical sprite/rendering implementation notes.
- `textures/agnes/`, `textures/dad/`, `textures/effects/`: preserved source artwork,
  unused by the combat lab and excluded from its export.
- `textures/backgrounds/`: the retained bedroom background.
- `audio/`, `fonts/`: impact sound and licensed HUD font.

Graphics can be added later as a presentation layer; move startup, active windows,
recovery, collision, jump physics and AI do not depend on sprite frame counts.
