# First playable family bout

Play Agnes against an AI Dad. Move with A/D or arrows (drag the touch stick),
attack with Space or ATTACK, and dodge with K or DODGE. A dodge follows held
movement; without movement it retreats from the opponent. Balance begins at
100. Emptying the opponent's balance scores a tumble; first to three wins.
The winner screen offers a rematch, also available with R. P or the bottom
practice button switches to the original animation sandbox, with manual Dad
controls J/L, I to attack, and O to dodge.

| Fighter | Reach | Balance damage | Attack duration | Windup | Knockback speed |
| --- | --- | --- | --- | --- | --- |
| Agnes | 1.15 | 25 | 0.47 s | 0.16 s | 3.5 |
| Dad | 1.8 | 34 | 0.74 s | 0.33 s | 5.0 |

Attack poses retain their authored frame counts; playback rate sets the timings.
A punch has an 80 ms contact window after its windup, hits only once, and must
face the target on the same ground line. Combat attacks commit to their initial
side, allowing a dodge through the opponent. Missing leaves the rest of the
animation as recovery. Dodges last 220 ms with invulnerability and translation;
the 850 ms cooldown prevents repeated dodges. Attacks cannot cancel into a dodge.
Hits interrupt the victim, briefly pause both fighters and synchronized shadows,
flash the victim, burst particles only on contact, play a generated impact sound,
and push the victim within the visible arena. Fighters stay inside portrait and
desktop views during combat. Existing walk cadence still follows actual speed.

Dad approaches until within his longer reach, retreats if crowded, and commits
to visible punches rather than tracking Agnes during windup. Every third attempt
has a longer hesitation. He does not read future inputs. AI is disabled in
practice mode. Rounds freeze during the tumble notice, then reset positions,
balance and combat timers. A double tumble awards neither fighter a point.

The shared `Game` state and `Bout` script own scores, phase and HUD; live
`Player` fields own each fighter's combat state. Hits use typed script messages.
All gameplay lives in Decay. The current artwork stands in for dodge/hurt/tumble
poses with tint, flash, pause and knockback; dedicated reaction sprites and camera
shake remain future polish. `audio/hit.wav` is a short synthesized impact, and
the bundled Chakra Petch font carries its license in `fonts/Fonts-OFL.txt`.

CI runs the original controller regressions in practice configuration alongside
combat tests for startup/range, single hits, dodge cooldown, AI and full matches.
Browser checks cover desktop/portrait movement and touch, then the real AI bout.
