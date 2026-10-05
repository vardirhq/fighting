# Fighter animation and facing

Agnes and Dad use the same `Player` Decay script. Scene properties select the
fighter name, opponent name, controls and initial facing; each pose and its nine
shadow taps remain under the fighter's transform. Dad's newly uploaded textures
are sliced by adjacent bottom-anchored 3x3 `.sheet` files.

Both fighters start on the carpet at Y -2.6 with scale 4.4. Initial X positions
are -1.65 and +1.65 to accommodate the portrait view.

| Animation | Seconds per frame | Nine-frame duration |
| --- | --- | --- |
| Idle | 0.11 | 0.99 s, looping |
| Start / stop / turn | 0.01 | 0.09 s |
| Walk / backward walk | 0.07 | 0.63 s, looping |
| Attack | 0.07 | 0.63 s |

Agnes uses A/D or arrows, Space and the existing touch stick/attack button.
Dad uses J/L and I. These keyboard controls are a way to test both fighters;
no AI or combat damage has been added.

Facing is decided by opponent X, independent of movement input. At equal X the
last facing is retained. Crossing sides triggers the existing 90 ms turn in place,
even while idle. A side change interrupts an attack so it cannot keep pointing
away from the opponent. Otherwise attacks keep their existing one-shot behavior.

Walking toward the opponent uses the matching facing-direction loop. Walking
away uses those same frames in reverse order, at the same speed; reversing
input during a loop switches playback immediately. Sindri does not accept
negative animation speed, so reverse loops are authored clips, including all
shadow taps. Start/stop transitions still use their original 90 ms clips.

Run the authored gameplay headlessly with sibling checkouts:

```sh
cargo test --manifest-path fighting/tests/runtime/Cargo.toml
```

CI additionally loads the exported game in pinned Playwright Chromium with
software WebGPU under Xvfb at
desktop and portrait mobile sizes. It compares rendered pink-clothing positions
after keyboard and touch movement, and saves idle, movement and retreat captures
as `fighting-browser-captures`. Native regressions check real Decay execution,
reverse frame progression, turn-in-place, attacks, ties, horizontal movement,
inherited pose transforms and frame-perfect shadow synchronization.

Software Chromium evidence does not replace checking a physical Android/iOS
device's WebGPU driver and touch feel.

Browser verification uses `npm ci` and the committed lockfile. Linux CI runs
headed Chromium under Xvfb with Vulkan/Mesa libraries and an explicit SwiftShader
WebGPU adapter. A 64-byte mapped-buffer probe runs before WASM startup and saves
adapter details alongside captures; adapter failures remain fatal.
