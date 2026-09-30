# OT2SpeedrunAssistant

This branch, `strategy/abandoned-village-fork`, is the starting point for a new
strategy. The GUI simulates through the first Abandoned Village entry, tag and
tutorial dismissal, then stops. Later events and their search targets are not
evaluated. The village's complete CRT and world state are available in Detailed
log and the JSON export, ready for the next route event.

`main` preserves the previous route through Crackridge Harbor. The backend also
retains it via `route_end=full`; the fork GUI selects `route_end=first_village`.

Route state, field encounter prediction, and a local GUI. This repository imports
the standalone **OT2BattleEngineRust** library through Cargo; it does not copy its
battle implementation. The battle repository has no dependency on this project.

```text
OT2SpeedrunAssistant (route state, encounters, GUI)
    └── imports OT2BattleEngineRust (headless battles, routines, CRT primitives)

Independent battle search tool
    └── imports OT2BattleEngineRust directly
```

The two repositories are currently adjacent directories, linked by a Cargo path
dependency. This is a normal static Rust dependency: no HTTP, subprocess, JSON
serialization, or dynamic dispatch is required between simulation cores. For
distribution, replace the path with a pinned Git revision or versioned crate.
No dependency on the original decompilation is needed at runtime or build time.

The **Manipulation** screen continues a reachable opening route through the
checkpoint, Ambush and skipped rejoin/0600 cutscenes into Throné's second
Chapter 1 sewer battle. Party level comes from opening EXP (normally level 2,
with full HP/SP recovery). It runs the specified two-round routine, including
Pirro's **Pierce Through** and 2-BP **Slice**. Command menus and CRT spans are
in Detailed log and JSON exports, with advance counts rebased to the initial
seed. The short walk is assumed encounter-free. If the first checkpoint is
unreachable, use **Find adjustment** before predicting this continuation.

The 2026-09-27 live replay matches all 144 second-battle CRT calls and all six
player command menus. Its serial event handoff consumes **zero** extra draws: 0600 starts
battle before `FinishPre`, so its `EncountReset` flag does not justify a
pre-battle reset. The result ends **after drops, before returning to
the field**; it does not reuse the first battle's popup resets. See
`data/source/throne-second.json` and the battle core's `docs/throne-second.md`.

After a second-fight victory, a separate route projection skips `0700`/`0800`,
loads New Delsta main and backstreets, and stops after the radar tutorial.
The world evaluator derives **110 CRT draws** for the captured state
(11 level colors, 85 NPC frames, 14 counter draws),
matching the live replay from **5264 after drops to 5374 after the popup**.
The earlier six-draw mismatch came from second-battle startup, enemy skill
selection and reaction handling. The fixed city profile has now been replaced
with runtime event/flag/placement/level evaluation. Detailed log shows individual
draws; exports include skipped-branch decisions and the persistent world state.
Other route/save states remain unvalidated.
See [the city source audit](docs/throne-city.md).

The route then enters the parlor, skips the first cutscene, runs to the next
trigger, selects Yes, skips two further cutscenes and returns to backstreets.
Both map changes use the town loader with carried world state; the event
interpreter applies the time change, flag lottery and counter resets. All 50
calls and 24 NPC identities match the captured 5374 → 5424 replay. The final
CRT checkpoints remain in Detailed log and exports.
See [the parlor replay](docs/parlor-live.md).

The second parlor visit, clothes theft, Boy's Shadow Soulstone, New Delsta main
load, Artist's three soulstones (one menu visit), and two skipped scenes lead
to **Mansion entry**. That card replaces the two parlor return cards in
Manipulation; both parlor stages still run and remain in Detailed log/exports.
The user-confirmed endpoint is **5562 / 0xDDCC3032** for the recorded route.
All map changes evaluate persistent progression and resident actors/levels.
See [the mansion arrival audit](docs/throne-mansion-arrival.md).
The mansion card also shows the following lookout Ambush: one menu-timer draw
and one sleeping-placement frame draw, with no encounter-counter reset.
Its expected endpoint is **5564 / 0x5121903C** for that route. The short approach
is assumed encounter-free; its walking cost is not yet modeled. See the
[lookout audit](docs/mansion-lookout-ambush.md).

## Run

Double-click **Open Speedrun Assistant.cmd**, or:

```sh
cargo run --offline --release --bin ot2-gui
cargo run --offline --release --bin ot2-gui -- --no-open --port 54345
cargo test --offline
```

The old battle viewer launcher forwards here for compatibility. Rebuild after
editing the embedded viewer files. The application binds only to localhost.

**Manipulation** is the compact run view for both fights: seed, title advances
and shared-level toggle, followed by route cards. **First battle** combines
damage-less status and the boost adjustment routine, with after-popup CRT and
counter as small diagnostics. **Safe running limit** includes checkpoint-popup
CRT as secondary information. **Second fight** shows the one-enemy-action verdict,
action/hit counts, and CRT after drops. Intermediate checkpoints and its routine stay
in Detailed log. After-popup CRT includes both the first battle's counter resets.
Damage-less uses damage received throughout the fight (including damage later
healed), rather than assuming every enemy turn causes damage.

**Detailed log** holds the latest completed simulation's input context, routine,
accuracy notes, route calculations, party resources, command-menu checkpoints,
action log and CRT trace. The trace includes the title-transition phases and
uses absolute seed advances for every callback. Editing inputs hides the run
verdict until rerun; the previous completed detailed log remains available.
**Field encounters** retains the full field prediction controls.

## Title-screen inputs

Enter the **initial seed and CRT advance count at the title screen**, immediately
before New Game as Throné. The GUI/API now add the captured opening transition
automatically; do not include that transition in the input count. Both opening
cutscenes are assumed skipped.

**Recreate shared level** on means Btl_SinExpose is created during this transition
(13 total draws). Off means it is already retained (12 draws). The phases are:
one timer, five regular streamed-level initializations, optionally one shared
level initialization, one NPC starting frame, a two-draw counter reset, one flag
lottery, and another two-draw counter reset. This is CRT-only, with no rendering.

Seed 0 at title advance **4916 + recreation on** reaches battle entry **4929**
(`0x0EA79703`) and Pirro's first menu **4959** (`0xF485D421`). The retained capture
starts at observed advance **4917 + 12**, reaching those same two checkpoints.
The repeat capture had an unrecorded state change before its continuous interval;
the model does not silently apply a reseed. Use the actual current title state.
Watching cutscenes, extra menu activity and other loader states are not covered
by these two captures.

The transition lives in `src/opening.rs`, in the assistant. The battle library's
input remains the CRT state immediately before battle initialization. The
`/api/crt`, `/api/simulate` and `/api/export` query parameter `advances` now means
**title advances**; field-prediction inputs keep their existing field boundary.
Exports identify `input_boundary: title_screen`, the route profile, title state,
transition draws, per-phase checkpoints and the resulting battle-entry position.
Export filenames use `title-advances` to distinguish older pre-battle-input files.
`data/source/title-transitions.json` keeps compact excerpts and source hashes;
`tools/import-title-transitions.cjs` regenerates them from the archived captures.

## Field encounters

The detailed log and field view show route checks for the opening routine. **Enemies never act** is
derived from the entire simulated fight's executed enemy turns, including misses.
It requires victory; it does not infer success just from the first two slots.
The current routine still boosts enemy 1 on the right, so an enemy 2 turn before
the other players can make this check fail. No targeting policy was changed.

The next-reset check uses the user's medium-speed walk measurement **830 → 146**:
684 points / 12 per update = **57 walking updates**, confirmed by repeated measurements.
The opening's running budget currently assumes the same count:
**57 × trunc(12 × 1.4) = 912** points. This is an estimate pending a separate
running measurement. Walking requires **685**; estimated running requires **913**.
Zero remaining is not certified safe (`IsExecEncount` checks `StepCount <= 0`).
The first-encounter counter can only reach 899, so no opening exit qualifies for
running under that estimate. The game discards excess displacement at each
decrement; walking, running and mixed movement need not use the same count.
This budget does not simulate the movement path or sampling schedule.
The measurement and source evidence are in `data/throne-reset-walk.json`.

For the mixed estimate, each running update costs 4 more points than walking.
Assuming 57 total updates and a counter of at least 685, the running count is
`min(57, floor((counter - 685) / 4))`; walk the remaining updates. The run view,
Detailed log, field summary and JSON show this mixed budget. Its percentage is
the assumed share of the 57 counter updates, rounded down to 0.1%, not measured
distance or elapsed time. Speed-switch locations can affect actual update counts.
For example, counter 775 permits 22 running + 35 walking updates (38.5% shown),
costing 772 and leaving 3. At counter 684 or below, no mix is safe; at 685–688,
walking alone is safe. Extra movement beyond the measured minimum costs more.

Flats has independent measurements: **61 walking updates** versus **56 running
updates** to harbor. The daytime costs are **549** and **672**; the GUI and JSON
use separate budgets. This ratio is not applied to the opening or mansion.

**Run simulation** automatically searches and applies first-fight boost inputs
that satisfy both targets: a final popup counter >= 685 for walking, and victory
in the second fight with all three requested order conditions met and exactly
one enemy action. Each eligible candidate replays the checkpoint, Ambush/rejoin
and second fight with EXP, level and resources from that adjusted first fight.
This also runs on initial page load. **Settings** has two independent fields,
each defaulting to **4**: **Encounter avoidance** limits the extra repetitions
used to find a safe walk; **Second fight · one enemy action** limits additional
repetitions beyond the actual cost of the cheapest encounter-dodge plan.
Unused encounter allowance is not transferred to fight two. If no encounter
plan fits its own cap, report no solution. Neither budget resets per command menu. A repeat is
one extra upward climb (e.g. `0->2 x5` has four extra repeats); the normal action
boost is excluded. If no combined plan fits, keep the cheapest encounter-dodge
plan and show that the fight-two target was skipped. Both settings accept
0–256 as optional budgets, or **Required outcome**, and persist independently across reloads. Required targets and their prerequisites are solved jointly before optional improvements; required fight two cannot be blocked by a finite checkpoint budget. Each toggle preserves its previous numeric value. See [search policies](docs/route-searches.md).
The old single setting migrates
to fight two only. Zero extra boosts are used when the selected
plan needs none; rerunning recalculates from the current inputs. The search
uses one first-fight player command menu. Capped searches cover all draw counts
permitted by their repetition allowance. Required searches start with 256 extra
CRT advances and expand until an optimal match is found or all distinct CRT
offsets have been covered (the stream repeats after 2^32 draws). An input-cost
bound rules out cheaper paths outside the current range before accepting a match.
It chooses the fewest added boost-button presses, including reductions
and reaching the normal action's final BP. Ties use fewer extra draws, then the
latest menu. A shortest-path search includes every available BP level, capped at 3.
It does not search combinations across menus or require a damage-less fight;
the resulting damage-less verdict is shown separately. Each candidate replays
combat, drops and both resets through the imported battle core. The result
replaces the displayed prediction and includes the exact round/actor/menu,
extra draws, final CRT and counter. Boost adjustment headings show the command
menu's position within that round (e.g. `R2 · Scaracci · 2nd command menu`),
counting player turns including Defend and excluding enemies.
Detailed log and JSON exports include the
chosen inputs and spans. Search metadata distinguishes a combined success from
an encounter-only fallback; the UI reports when even encounter avoidance fails.
Input optimization respects the event cap, including longer input paths that
produce the same CRT draws with fewer repetitions.

`GET /api/boost-search?seed=0&advances=4916` returns the search and replayable
prediction. `encounter_max_boost_repeats` and `second_fight_max_boost_repeats`
set their respective event budgets (each defaults to 4). Pass `required` for
either outcome parameter; legacy `unlimited` remains accepted. JSON represents
it as `null` and also exports explicit `search_policies` and `required_outcomes`. The legacy query
parameter `max_boost_repeats` remains a fight-two-only alias.
`/api/simulate` and `/api/export` accept `boost_menu` (one-based,
1–6) and `boost_advances` (positive u32), with both zero for the baseline. Search returns
an explicit null solution if the bounded search finds none. `boost_repeat_cap`
preserves the chosen total repetition constraint when replaying/exporting the
exact input path; omit it for an unconstrained explicit plan. This is a
source-derived prediction; repeated boost inputs still need live validation.
A `0 -> 1` boost draws once; `1 -> 2` twice; `2 -> 3` three times; decreases draw
nothing. This opening only supplies up to 2 BP. The planner shares setup inputs
with the ordinary action instead of requiring an unnecessary return to zero.
The viewer shows the whole boost routine, including the normal final boost,
as grouped climbs followed by the exact action. For a 2 BP action, +1 extra
draw displays `0->1 x2`, then `1->2`; +2 displays `0->1`, then `1->2 x2`;
+3 displays `0->2 x2`. Single climbs omit x1. Between repetitions return to
the stated starting BP; after the last repetition stay at its destination.
Any remaining reductions are shown explicitly. Exports retain every button
input, and the battle core executes that same full sequence once.

Manipulation shows damage-less before dodge inputs, then popup CRT/counter,
movement budget with checkpoint CRT, and second-fight exit CRT. Advances since
initial seed are primary; hex states are secondary. If the route is safe, the
detailed checkpoint log shows arrival (no movement CRT) and the expected +4 draws through 0400 and its Ambush
tutorial. This event projection awaits live validation; see
`docs/throne-checkpoint.md`. No post-checkpoint state is offered for an unsafe walk.
Detailed log also shows CRT and counter after Ambush and the skipped rejoin
cutscene. This projects one path-action menu opening (+1), the scripted Ambush
event's counter reset (+2), and the rejoin cutscene's counter reset (+2). Detailed
log and JSON preserve each boundary, both counters, the restored party and
encounter state. This +5 projection awaits live trace validation; see
`docs/throne-ambush.md`. Extra menu openings are not part of this fixed routine.

If the selected movement reaches the next cutscene first, the route prediction
ends **at its trigger, before event/reset callbacks**. It preserves CRT apart
from explicitly supplied intervening calls, and never consumes a hypothetical
encounter's reset/selection draws. Otherwise it predicts the intervening encounter.
JSON exports include the battle assessment and both movement budgets in battle
results, and both movement budgets plus the reached boundary in field results.

Open **Field encounters**. The default **After Throné’s opening battle** frame
automatically continues the latest opening simulation into Underground Waterway.
It carries the final CRT position and the counter already drawn by post-battle
event `MS_TOU_10_0300`; it does not draw another initial counter reset. Rerunning
the battle refreshes this field prediction. Walking/dashing and explicitly
supplied intervening CRT calls remain adjustable.

This fresh-new-game frame uses night, main progress **0**, FirstEncount active,
and Throné/Pirro/Scaracci at level 1. Task progress **30** is not main progress.
The active actor is `ENT_DNG_CTY_3_1_LND_1`, with flag 25304 off. The preset expires
at `MS_TOU_10_0400`, which sets that flag, selects `ENT_EMPTY_1`, and removes the
two guests. Other story stages require a different route frame.

At seed 0 + 4918 at title (shared level retained; battle entry 4930), the model exits at advance 5083,
state `0x194611CD`, counter 549. The next encounter is two Pursuers, after 46
walking counter updates or 35 dashing updates. Selection consumes 6 CRT calls
(trigger reset B/A, situation, three party timings), ending at advance 5089.
This is a source-derived regression, not a new live-game validation.

Select **Custom area** to supply a seed, advance count, map, active zone,
day/night/world state, progress points, and party levels manually. Modifiers
are aggregate effects from currently eligible equipment/supports.

- **Counter reset → next encounter**: draws the initial field counter, counts
  movement updates, advances any explicitly supplied intervening CRT calls, then
  predicts the encounter. Includes the separate reset at the encounter trigger.
- **Existing counter → next encounter**: uses an existing remaining counter;
  there is no initial reset. **Use Throné post-battle frame** restores the full
  route preset, including its map, story progression, first flag and party.
- **Encounter trigger, before its reset**: starts just before the two counter
  rolls at an encounter trigger. Suitable for comparing a recorded encounter.
- **Counter reset only**: predicts only the map's next counter reset.

Results show counter formulas, decrement per movement update, ordinary group,
fixed first encounter where applicable, normal/preemptive/surprise situation,
Cait/Octopuff replacements, and CRT checkpoints including absolute seed advances.
Export JSON preserves the exact setup and large advance counts as strings.

The encounter endpoint is **after rare replacement selection, before enemy actor
initialization**. It is not a command-menu or post-battle state. Movement update
counts are not animation footsteps or time estimates. Other map activity can
consume CRT; enter those calls explicitly. Counter resets, zone changes and
story events must be separate route steps rather than assumed absent.

## Data and accuracy

Imported from the local cooked tables: 229 maps, 304 zone definitions, 721
volumes, 1086 weighted encounter tables, 3372 groups and 1689 enemy records.
The placement audit links 136 maps, with 197 placed and 93 inferred map/zone
links. Twelve active zones have unresolved links. Manual zone selection is
available and identified as an override. Story flags are resolved for the
supported Throné frame; arbitrary custom-area flags remain caller inputs.
Some cooked rows and zone variants reference missing groups. Selection of incomplete data
returns an error without committing any RNG changes.

The Deserted Highroad capture at seed 0 + 4940 is verified through all eight
selection checkpoints: 1471 counter, group roll 16, preemptive roll 3, Chubby Cait
+ City Ratking III, final position 4951 and state 2246832313. Use party levels
84,85,77,84, Cait equipment bonus 2, rare support bonus 2, day, and trigger mode.
This verifies that captured case, not every map and game configuration.

Provenance is in `data/source/encounters.json` and
`data/source/live-deserted-highroad-4940.json`. `tools/import-encounters.cjs`
reimports the original local tables and existing placement audit; it is optional
build-time tooling requiring the parent research workspace. Prediction uses
binary32 game arithmetic and the main CRT stream only.

`data/source/throne-field.json` records the story-boundary sources, hashes and
native function evidence. `tools/import-throne-field.cjs` reproduces the preset
from EventList, event scripts and the placed encounter actor. The encounter
catalog still selects the progression-dependent table; the route does not
hardcode enemy formations. Tests cover progress 0 versus 1 in this same map.

The opening and second battle results are combined in **Manipulation**.
The second-fight card predicts **Only one enemy action?** and displays actual
action/hit counts. Detailed log checks R1 Throné before the right grunt,
R1 Scaracci before the left grunt, and R2 Throné before the leader. The round-two
comparison uses the cached next-round queue; broken grunts are skipped when
executing it. A Double Thrust is one enemy action with up to two damage hits.

The current Lv2 estimate is **31.2773%** (312,773 / 1,000,000 sampled uniform
32-bit CRT states immediately before second-battle initialization). All three
order checks and actual one-action victories agreed on every sample. Individual
condition frequencies were 95.1538%, 50.6198%, and 66.6906%; they are correlated,
so the joint rate was measured directly. This is not conditioned on the opening
battle, encounter-dodge boost search or a particular title-screen timing range.
Sampling uncertainty is approximately ±0.09 percentage points at 95%; it does
not account for model inaccuracies or replace live validation.

Reproduce with `cargo run --offline --release --example second-fight-odds -- 1000000 0`.
The example uses a deterministic SplitMix64 sampler and reuses the independent
battle core's immutable scenario. It runs full fights and includes failed fights
in the denominator. This broader sample also found 113 cases where the routine
did not finish within two rounds (all outside the three-condition success set).
The first, entry `0x8F0D5BC7`, has Scaracci knocked out before his round-two action;
the current model therefore cannot support an unconditional always-win claim.

Its existing accuracy limits still apply: the historical routine matches its
154-call capture, the revised first-actor boost routine needs a live comparison,
and the reported Throné first-hit damage discrepancy remains unresolved.

## Route boundary

`RouteState` owns persistent CRT position and the current field counter. It can
advance known callbacks, predict an encounter, and accept a separately simulated
completed battle via `finish_battle(entry_state, battle.state())`. It checks CRT
continuity and consumes the battle's exit resets exactly once. Missing exit
counter information becomes unknown, never a guessed counter.

This is the route-state foundation, not a route editor or full-game simulator.
The next layer should be a serialized ordered list of explicit route steps
(movement, events, encounter selection, battle plus player routine). The route
runner should dispatch those steps; the battle library should remain unaware of
routes, map selection, GUI controls, or searches over field RNG.

Arbitrary field encounters cannot yet be played through by the battle library.
When adding that support, its initialization entry point must honor the field
predictor's endpoint: party timing has already consumed CRT, enemy initialization
has not. Do not feed this intermediate state into an initializer that repeats
party initialization. The current tested battle handoff is a completed supported
battle into the field predictor, not a complete field-battle-field loop.

The serial route now continues from a successful mansion Escape into Pirro's
Chapter 1 boss battle. Manipulation shows the fixed routine's selected branch,
outcome and post-victory CRT; Detailed log includes the full fight. See
[the Pirro route audit](docs/throne-pirro.md) for boundaries and validation status.
