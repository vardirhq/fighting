# Bedroom visual effects

This slice uses capabilities in the pinned Sindri browser host:

- Window glow: twelve very faint nested additive world ellipses, behind Agnes.
  This approximates a soft warm halo without applying a filter to the artwork.
- Window dust: two tiny flecks every 0.35 seconds, living for 3 seconds.
  Positions follow a deterministic pattern local to the atmosphere script, so
  cosmetic ambience never consumes the game's random stream.
- Step puffs: two short-lived flecks every 0.14 seconds of actual horizontal
  travel. Turns, attacks, idle time and movement clamped at the arena edge do
  not emit them.
- Attack sparks: one twelve-fleck accent when the attack reaches frame 3, on
  the facing side. This is a visual accent, not collision or hit confirmation.

Effects use the runtime particle pool and fade/drag, with scene-authored budgets.
The character's movement speed, animation state machine and shadow timing are
preserved. World layer 1 is the halo, 2 ambient dust, 5 shadow, 7 step dust,
10 character and 12 attack sparks. The interface stays above these.

The 8x8 fleck texture is copied unchanged from
vardirhq/sindri-engine/games/orbital-baked/assets/textures/fleck.png at
dae1b14df254413d24679556e88156a907e5909c.

## Bloom limitation

The pinned runtime contains a Bloom renderer and authored environment settings,
but game/src/browser/mod.rs calls encode_prepared_frame rather than
encode_lit_frame. Setting environment.bloom therefore does not enable browser
bloom. The warm halo here is an authored glow, not post-process bloom. A real
bloom integration belongs in the shared runtime and needs native/browser render
coverage before changing the engine pin.

## Visual verification

On desktop and mobile, check dust near the window, subtle puffs while walking,
one brief burst per attack on the facing side, and clean character/control edges.
Compare with effects disabled before increasing intensity. Compiler/export checks
cannot establish these visual results.
