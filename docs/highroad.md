# Eastern New Delsta Highroad: Al and first Escape

Galdera revision (2026-10-22): R8 targets a non-surprise opening without requiring
a Throné-breakable formation. Highroad compares direct Escape and optional
break-then-Escape by the existing input-cost model, jointly
preserving the Harbor walk and safe Flats opening. Flats has no breakability
target either. The Galdera flag is carried from Pirro before either search;
only selected breaks grant latent gauge. Any% Hikari retains its break requirements.
The older configurable break-policy descriptions below apply to Any%/legacy APIs.

`src/highroad.rs` continues the selected Pirro victory, using the carried world
state and the existing ending/map loader. Encounter selection/startup remains
source-derived. The 6002-menu Escape capture verifies the ten subsequent CRT
calls through the return to field control at 6012.

The route immediately changes Dawn to Day, refreshes NPC eligibility, skips
`SC_FC13_Tutorial_0100`, dismisses its tutorial and receives the bag quest.
The imported event executes its own FinishPre counter reset, tutorial flags
and quest flags. Its fixed dialogue and event-only character do not execute
the ordinary NPC creation/frame path. The time change evaluates the actual
Black Market flag lottery; no fixed stage advance is added. Existing event
and map imports are reproduced by `tools/import-town-loading.cjs`.

Continue to the first daytime land encounter using the current counter,
progress 1, actual party level, first-encounter flag and ordinary selector.
Walking itself consumes no CRT. No other actions or time changes intervene.
EXP comes from the imported second-fight/boss base rewards plus actual
break/overkill bonuses and the opening's recorded EXP. The resulting level-up
restores HP. The supported route keeps Throné's original equipment.

The battle core's `field_escape::FieldEscapeMenu` handles the first
player menu of normal/preemptive formations. It shares enemy initialization,
turn-order shuffle, voice traversal and Escape probability logic with the
battle core. Normal openings promote the first player; Surprise is rejected
instead of guessing preceding enemy actions. Preemptive Escape bypasses the
check. Failed attempts stop at the check, without invented field callbacks.
The private level-five single-enemy break profile reuses the normal attack,
accuracy, damage, shield, voice and turn code. It tries unboosted attacks first,
then 1 BP, preferring dagger over sword at equal BP. A predicted miss that leaves
shields intact is rejected. Successful breaks skip the enemy's remaining R1 and
R2 actions; Escape at the R2 player menu bypasses the lottery because the only
enemy is still broken. Enemy AI execution is explicitly disabled in this profile.
`tools/import-field-escape.cjs` in the core imports ordinary/possible low-tier
rare enemies and audits their AI and group-start gates.

Escape and surprise use different level calculations. Escape is
`max(sum(party levels)/4,1)` versus area danger; surprise divides by actual
party count, then compares enemy levels. Solo level 5 versus danger 5 has
40% first Escape. Audit: `outputs/mansion-escape-audit/CheckEscapeSuccess-rechecked.txt`
and `JudgeBattleSituation-rechecked.txt` in the enclosing research workspace.

## Independent searches

- **Pirro R8:** freeze the chosen R3 and all upstream inputs. Both no-grape
  branches end the first R8 menu with 3 BP swords. Early latent activates
  before boosting; late latent activates on R9. Preview and execution share
  that conditional activation/voice path. BFS minimizes actual button inputs,
  then actual CRT draws; each candidate replays combat, ending, Al and field
  selection. Require victory, no grape, and Normal/Preemptive. A healing
  branch uses an item on R8 and is explicitly unavailable for this search.
- **Highroad menu:** search `[1,0]` cycles before the first Escape, independently
  of R8. This menu has one BP. Increasing cycles is minimum input order. If no
  solution fits the configured limit, test whether the untouched opening menu
  can break a lone enemy immediately, then Escape next turn. If not, continue
  the boost search beyond the limit automatically. No failed trial check or
  discarded boost is executed in the selected route. Boost only as the initial
  setting searches directly for Escape without taking the break fallback.
- **Flats walking / returned-Flats surprise:** first solve all required outcomes
  jointly with Escape, then try optional improvements in route order. Required
  non-surprise includes safe walking to Harbor even if walking has a finite
  budget. Replay the shared field loader and all map/encounter events; the
  walking counter must be >=550. An optional miss preserves the required plan.
  Every candidate strategy, including immediate break, must satisfy all required
  conditions before it is chosen. No failed trial actions or fixed offsets are
  applied. See [search policies](route-searches.md).

Each numeric value defaults to 4 and accepts 0..256. Outcomes have a Required
outcome toggle; escape strategies instead have Boost only. An R8 capped failure keeps the earlier valid plan and reports no adjustment.
The Highroad Escape number is a threshold before trying the break fallback,
rather than an absolute search cap. Its export includes the chosen method,
exact boost inputs or weapon/BP/break action, next menu and Escape trace.
Never borrow R3 allowance. Export/API uses `pirro_r8_boost_inputs` for exact
replay; it preserves the normal final boost instead of applying it again.
Manipulation shows boost inputs, or the break weapon/BP followed by Escape;
composition/chance/CRT trace are in
Detailed log and JSON.

Settings/API key `highroad_flats_max_boost_repeats` controls the separate
Flats allowance; `escape.flats_search` records baseline/selected counters,
budget, added repetitions and success. The GUI displays the complete selected
boost path followed by the Flats running allowance. See new-delsta-flats.md for
the separate measured full-walk/full-run costs and mixed-distance estimate.

## Regression checks

Seed 0/title 4942/shared off retains the validated ending at 5962. Source
projection then gives Al 5965 (`0xD12B9CDF`), first menu 6002 (`0x5D64BAEA`),
40% baseline failure. Five 0->1->0 cycles succeed, ending the check at 6008.
A four-repeat threshold falls back automatically to five because there are two
enemies. Unlimited also finds five. The 2026-09-29 capture verifies menu6002,
all five boost draws and the successful check at6008. Field restoration then
runs the two imported gondola BeginPlay callbacks before counter B/A, ending
at6012 / 0x2E4773FC, counter1448. The exact ten-call regression checks states,
caller categories and actor identities. See data/source/live-highroad-escape-6002.json.

Seed 0/title 4972 needs the R8 path `[1,0,1,2,3]` to replace a Surprise opening
while preserving R3, victory and all pre-R8 actions. API replay reproduces it.
Core tests separately exercise early/late latent R8 preview against execution,
preemptive bypass, illegal BP paths and unsupported enemy rejection.
