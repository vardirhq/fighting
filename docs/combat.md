# Combat lab

This is a gameplay prototype: only the bedroom background is textured. Fighters,
shadows, hands, guards and attack telegraphs are native Sindri shapes. Source
sprite sheets remain in git, but no longer ship or control gameplay. All combat
runs in Decay, against the unchanged engine revision in `.sindri-engine`.

Play Agnes against Dad. First to three rounds wins. Each round lasts 45 seconds;
a knockout ends it early, otherwise the fighter with more balance wins. A tie
awards neither fighter a point. R restarts; P toggles manual training, where
knockouts reset both fighters without awarding points. Ready and round notices
freeze combat and preserve the knockout silhouette until the next round.

| Action | Keyboard | Touch |
| --- | --- | --- |
| Move | A/D or left/right arrows | Drag outside buttons |
| Jump | W or up arrow | JUMP |
| Crouch | S or down arrow | Drag down |
| High guard | Hold left Shift | Hold GUARD |
| Low guard | Hold crouch + guard | Drag down + hold GUARD |
| Jab / aerial kick | Space | JAB |
| Sweep | E | SWEEP |
| Dodge | K | DODGE |

Manual Dad in training: J/L movement, I jab, H sweep, Y jump, N guard,
M crouch, O dodge. Training retains the complete combat rules, disables AI,
and automatically restores both fighters after a knockout. It replaces the
old animation sandbox.

## Decisions and timings

| Move | Startup | Active | Total | Damage | Reach | Energy |
| --- | --- | --- | --- | --- | --- | --- |
| Jab | 120 ms | 100 ms | 340 ms | 12 | Agnes 1.25 / Dad 1.45 | 7 |
| Sweep | 320 ms | 100 ms | 780 ms | 26 | 1.70 | 24 |
| Aerial kick | 100 ms | 100 ms | 420 ms | 18 | 1.65 | 14 |

Orange outlined zones show windup; solid red-orange shows the active window;
faded outlines show recovery. Hits require facing, range and the same ground
lane. Every swing can hit once. Facing is committed throughout a move; walking
cannot cancel recovery. Holding an attack key does not repeat it. Inputs buffer
for 120 ms, including during impact freeze. A confirmed jab can cancel its
recovery into one sweep, at the sweep's full energy cost and startup. A blocked,
parried, missed or dodged jab cannot earn that cancel.

Attacking a recovering opponent deals 25% extra damage and displays PUNISH.
Jabs interrupt sweeps before they become active. Hits cause 55 ms impact freeze,
recoil and brief hitstun; sweeps hit harder and leave longer recovery. Accepted
hits play the bundled impact sound. Read these timings as initial tuning, not
finished balance.

Jumping costs 10 energy, uses a 7-unit/s launch and 18-unit/s² gravity. At more
than 0.35 units above the carpet a fighter clears sweeps. A jab can still hit
within 1.05 vertical units; an aerial kick reaches down/up within 1.65 units.
Aerial attacks are limited to one per jump. Airborne movement can cross the
opponent after rising above 0.5 units; grounded fighters have 0.68-unit body
separation. Crouching changes silhouette and movement speed, and selects low
guard; it does not make a fighter automatically invulnerable.

Guard blocks matching attacks from the front: standing guard stops jabs and
kicks, crouching guard stops sweeps. The first 85 ms of a newly raised guard
parry a matching attack, briefly stunning its attacker and restoring 8 energy.
Holding guard drains 8 energy/s; blocking costs 16 (jab/kick) or 38 (sweep).
A depleted guard breaks, takes damage, and suffers 600 ms stun. Wrong-height
guard gets hit. There is no health chip on a successful block.

Dodge costs 22 energy, translates at 7.8 units/s for 280 ms, and has a 650 ms
cooldown. Its first 120 ms are invulnerable; its end is punishable. It follows
held movement, otherwise retreats. Dodges pass through bodies but stay inside
the visible arena. Attacks, hitstun and airborne states cannot cancel into dodge.

Energy restores at 26/s while neutral and not guarding, after a 450 ms delay
following spending. Move, crouch and jump landing remain available according to
their state rules; insufficient energy prevents attacks/jumps/dodges. HUD bars
show health/balance and energy separately.

## Opponent

Dad makes decisions every 220 ms using visible distance, energy and current
attack phase, not player inputs. He approaches, backs away to recover energy,
varies jabs and sweeps, occasionally retreats/guards, jumps a telegraphed sweep,
and tries to punish recovery. He can kick during a jump. His deterministic
seven-choice cycle makes tests repeatable, but this is a baseline AI: difficulty
levels, adaptation, additional moves and personality are future experiments.

## Verification

Native tests run the real authored scene and scripts: timing and range, body
separation, jumping and aerial hits, high/low guard, parry, dodge crossing,
punish bonus, energy recovery, AI, timed rounds/rematch, training and touch.
Touch action pads use press-identity capture in the pinned engine (slider
interaction behind custom shapes), so a second finger can act while the first
owns the stick. Actions trigger once on press, guard stays held, and release
does not produce a second attack. Only one action pad can be captured at a time.

Browser checks cover desktop/mobile rendered movement, touch movement, shape
states, real AI balance loss and rematch. CI exports the project with no fighter
textures. Archived rendering notes in `docs/legacy/` describe the former sprite
implementation and are not instructions for this scene.
