# Fighter animation and facing

Agnes and Dad use the same `Player` Decay script. Scene properties select the
fighter name, opponent name, controls and initial facing; each pose and its nine
shadow taps remain under the fighter's transform. Agnes assets live in
`textures/agnes/`; Dad uses the `_256` textures in `textures/dad/`,
sliced by adjacent bottom-anchored 3x3 `.sheet` files into 180x256 frames.

Both fighters start on the carpet at Y -2.6. Agnes uses scale [3, 3, 1];
Dad uses [4.21875, 6, 1], making him twice her authored height. His 180x256
frames require width/height = 180/256, so the narrower X scale preserves the
original artwork rather than stretching it into a square. Dad's 256-pixel frame
height at scale 6 now matches Agnes's 128-pixel frame height at scale 3. Every pose and its
shadows inherit that same sizing. Initial X positions are -1.5 and +1.1 to
keep both resting fighters inside the portrait view.

Fighter overlap uses the root ground Y, never a pose's animated feet or head.
Lower Y draws in front (layer 11 versus 10). Within 0.02 world units of the
same carpet line, the shorter fighter draws in front; equal heights use the
primary fighter as a deterministic final tie-breaker. All seven poses receive
the same layer, while shadows remain on layer 5 beneath both fighters. Agnes
also has layer 11 in the authored scene, before scripts start. If jumping is
added, keep ground Y separate from the visual jump offset.

| Animation | Seconds per frame | Duration |
| --- | --- | --- |
| Idle | 0.11 | 0.99 s, looping |
| Turn (three frames) | 0.03 | 0.09 s |
| Walk / backward walk at 5 units/s | 0.07 | 0.63 s, looping |
| Attack | 0.07 | 0.63 s |

Agnes uses A/D or arrows, Space and the existing touch stick/attack button.
Dad uses J/L and I. These keyboard controls are a way to test both fighters;
no AI or combat damage has been added.

Facing is decided by opponent X, independent of movement input. At equal X the
last facing is retained. Crossing sides triggers a three-pose 90 ms turn,
even while idle, without delaying or pausing held movement. An attack can
interrupt the visual turn immediately. A side change interrupts an attack so it
cannot keep pointing away from the opponent. Otherwise attacks keep their existing one-shot behavior.

Walking toward the opponent uses the matching facing-direction loop. Walking
away uses those same frames in reverse order, at the same speed; reversing
input during a loop switches playback immediately. Sindri does not accept
negative animation speed, so reverse loops are authored clips, including all
shadow taps. Movement enters the loop on the first input frame and returns to
idle on the first release frame; there are no start/stop animation states or
scene entities. The original start/stop PNGs and sheet definitions remain as
source artwork for possible future reuse. Earlier Dad assets are kept in
`textures/dad/archive/`. Turn clips select the existing start,
front-facing pivot and end frames, with identical clips on their shadow taps.

Walk playback speed is actual distance travelled divided by frame time and
`walk_reference_speed` (5 world units/s by default). At half that velocity the
cycle runs at half speed; faster configured movement increases the cadence.
The same multiplier applies to all nine shadow taps without restarting their
playheads. A clipped final step slows proportionally, and held input at the
room boundary shows idle when no distance is travelled. Idle, turns and attacks
reset to their authored timing when shown.

Dad's three-pose turn uses frames 0, 4 and 7 (reversed for left-to-right);
frame 8 is deliberately excluded because the uploaded final pose turns back
toward the camera. All nine frames in his other sheets are used. His left attack
reuses the right attack sheet with local X scale -1 on that pose only. Shadow
children inherit the mirror; root movement, layering and attack effect placement
still use the true facing direction. This also mirrors the asymmetric jacket
pocket during the left attack; authored left-facing idle/walk/turn stay intact.
Dad's attack spark starts at frame 4, where the new punch extends, while Agnes
keeps frame 3.

Run the authored gameplay headlessly with sibling checkouts:

```sh
cargo test --manifest-path fighting/tests/runtime/Cargo.toml
```

CI additionally loads the exported game in pinned Playwright Chromium with
software WebGPU under Xvfb at
desktop and portrait mobile sizes. It compares rendered pink-clothing positions
after keyboard and touch movement, and saves idle, movement and retreat captures
as `fighting-browser-captures`. Native regressions check real Decay execution,
reverse frame progression, immediate starts/stops, movement through turns,
attacks, ties, horizontal movement, velocity-scaled playback, inherited pose
transforms and frame-perfect shadow synchronization.

Software Chromium evidence does not replace checking a physical Android/iOS
device's WebGPU driver and touch feel.

Browser verification uses `npm ci` and the committed lockfile. Linux CI runs
headed Chromium under Xvfb with Vulkan/Mesa libraries and an explicit SwiftShader
WebGPU adapter. A 64-byte mapped-buffer probe runs before WASM startup and saves
adapter details alongside captures; adapter failures remain fatal.
