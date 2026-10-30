# Hikari free Clockbank windows and paid New Delsta advancement (2026-10-30)

Production Hikari EXP100 search now starts from the existing New Delsta return,
before Agnea's Chapter 2 opening. Extra paid switches happen there at 12 each,
in day-ending pairs to preserve the daytime Highroad encounter. Every candidate
replays the Highroad Escape, crafting, Clockbank movement and story events.
Hikari retains the nearest safe Highroad/Clockbank approach; additional RNG
advancement is allocated to town windows and late field controls. Free-window
frontiers are cached by exact world/RNG state, with bounded per-branch work.

Clockbank has seven optional free windows with capacities 2/2/1/2/2/2/3:
main town before Industrial entry; Industrial entry before Will/Ori; before
factory/Roque scenes; before headquarters scenes; before the first factory visit;
after its exit before the weaponsmith; after the weaponsmith before the second
factory visit. Zero through capacity are considered independently. Exact world
and RNG keys merge equivalent prefixes; no RNG offsets replace callbacks.
Odd windows and nighttime factory visits are allowed. If delivery ends at night,
a separate day switch before the existing Crackridge leg costs 12 and is shown.
Paid surplus advancement is never appended in Industrial District by production.
The old standalone Clockbank search remains only for historical fixture tests.

Shared weights: boost press 1, ordinary break 11, Fireball break 26, Defend 4,
round advance 6, time switch 12. The 14 measured Clockbank slots cost zero.
Mandatory early Throné breaks, Scholar guild, Hikari recruitment/Osvald bench,
Clockite errands and four field encounters remain. The reset warning stays
Galdera-only. Target estimates only rank candidates; acceptance requires actual
EXP100, victory, four survivors, successful escapes and full replay.

Tests: clockbank_free covers real night loads, normalization, zero/full windows
and exact frontier replay; identified_route's ignored Hikari full-route regression
checks required breaks and the production New Delsta/Clockbank plan. The viewer
shows each selected window count and its ending period.

The sections below describe historical search policies and benchmark weights.

Publication checks (2026-10-30): the two free-window tests, six Oresrush tests,
three viewer regression scripts, GUI compilation, and explicit production
New Delsta-to-EXP100 replay passed. That replay selected zero paid New Delsta
switches and all fourteen free Clockbank switches, with total input cost40.
The historical standalone `refinement_improves_known_plan_and_replays_full_fight`
test fails its191-input performance ceiling. This is an unresolved search-cost
regression in that historical entry path; do not report the entire suite as passing.

# Clockbank BG search: bounded improvement of a valid plan

## Any% Hikari backport (2026-10-23)

The shared engine already includes the Galdera-era battle corrections: BG
effect callbacks and voices (including the death-whisper Miss display), Defend
and Springy Boots ordering, camera timing, and first-turn-only preemptive Escape.
Conning's safe first-round Defend search, early rare-kill boosts, cooperative
cancellation and progressive route controls also already serve both routes.

Hikari now adds the target-ranked refinement previously restricted to Galdera.
After establishing its existing valid fallback, it measures the actual 0–7
safe-Defend draw curves and ranks partial allocations against EXP100 lottery
targets, including nearby later targets. The exact route and battle must still
replay to victory, EXP100 and four living characters. A computational cap stops
refinement while retaining the accepted fallback; this is not a global optimum
or an exhaustive search of all allocations.

The four sites remain Crackridge Harbor after Clockite, Southern Crackridge,
Western Crackridge before town, and Western Crackridge after town. There is no
Trader altar visit or fifth encounter. Clockbank remains the time-adjustment
location, and every departure has an even number of switches and daytime.
Each switch now costs12 input equivalents, each Defend3, and crossing a battle
round8. Earlier timing weights and benchmark costs below are historical.

Second-round Defends stop before an enemy action. Escape is checked using the
actual round, party levels and RNG; a preemptive opening cannot bypass the
second-round escape roll. The existing final Western Wind Soulstone option can
also follow Defends. Both Defend and final-action widgets show the actual turn,
Hikari party member and command-menu ordinal.

The refinement has at most600000 work units,4000 per candidate, and ranks up
to384 profiles per target. Hikari's fallback retains its old first-round control
range. Exact fight-cache failures record their effective fight cap and Guards
window; a small-cap failure must not reject a larger later query.

Route invariants remain unchanged: mandatory Throné breaks at the first
Highroad, Flats and post-Winterbloom Harbor encounters; Scholar license and
Western Winterbloom's second-encounter option; Hikari recruitment and Osvald's
bench slot; Agnea/Partitio story and Clockite delivery; original Gravell jobs,
skills and EXP100 routine; endpoint at Gravell. No Galdera errands or JP100
search are added, and the New Delsta reset warning remains Galdera-only.

Four checkpoint states, using the same engine and party before/after the search
backport, produced these selected Clockbank switch counts:

| Checkpoint CRT state | Before | After |
| --- | ---: | ---: |
|2447445413|38|4|
|806913878|14|0|
|3461349639|74|24|
|1820818104|172|120|

These are checkpoint samples, not a guarantee for every seed. Regression tests
replay the switches, all field Defends/Escapes and the complete Hikari EXP100
battle. `identified_route::tests::hikari_backport_preserves_required_breaks_and_route`
is an explicit optimized full-route test covering recruitment/Winterbloom
branches and the mandatory breaks. UI regressions are
`tools/test-progressive-controls.cjs` and `tools/test-gravell-action-labels.cjs`.

Validation: 209 assistant unit tests, the explicit full Hikari route replay,
four checkpoint replays, 135 core battle tests, cancellation and both UI suites
passed. Five Galdera fixture tests fail on both this change and unchanged
assistant HEAD438329f with core3c6185f (Temenos/Wellgrove/Flamechurch fixtures
omit Merchant setup; Winterbloom omits the Coast Merchant; direct Inventor
return omits Elemental Augmentation). The broad suite's legacy Flats Cait-fork
and Snows cached-layer comparisons were stopped after several minutes; they
are not claimed as passing. No production rules were relaxed for these tests.

The planner first establishes a fully replayed baseline, then tries cheaper
plans. Search work limits never turn a successful required outcome into an
unavailable outcome. They limit refinement quality, not battle execution.

## Baseline and refinement

- The baseline retains strict propagation through the first three field escapes:
  commit to the cheapest control preserving the next safe encounter. Last Western
  Crackridge may Escape or use Wind Soulstone L. The baseline retains the previous
  74-input per-field-action cap and 100 combined Guards/BG input cap.
- Scan EXP100 lottery states, but require an actually executed lottery, real
  victory, and four living players. Later children can invalidate EXP100.
- Refinement first expands the combined local input budget to 192 and removes
  the minimum 650-draw departure gap. It then considers Guards alternatives up
  to 12 additional inputs beyond the first finisher-ready Guards roll.
- Compare total inputs: actual boosts, safe Defends at 3 each, Wind L at 10,
  and time changes at 7 each. Keep the baseline unless a complete cheaper plan
  is found. Later lottery anchors can beat an earlier accepted anchor.
- Refinement receives 5,000 work units for the first pass and another 5,000 for
  Guards alternatives, carrying unused units forward. A Guard/BG candidate or
  Escape probe costs one work unit; a field Wind L simulation costs five.
  This deterministic effort budget is unrelated to game RNG or player input cost.
- Candidate departure windows still extend at most 1,900 CRT draws before an
  anchor. This and the local budgets are search heuristics. Global optimality
  across every possible route/control is not claimed.

## Exact reuse and pruning

- Boost frontiers retain the shortest input sequence per exact RNG/voice/BP
  state. Joint BG frontiers merge only equal full mutable battle states, including
  HP/SP, statuses, inventory, turn order, enemy knowledge, and voice history.
- Reject branches already past the target lottery using exact CRT-cycle distance;
  future draws cannot move backward within this search window.
- Cache battle queries by CRT state, situation, carried player configurations,
  enemy formation, and enemy knowledge. Caches are scoped to one lottery and
  search policy. A work-truncated failure is never cached as infeasibility.
- Field Escape cursors advance incrementally; each attempt runs on a clone.
- Search probes retain only lottery trace entries. Accepted battles replay from
  their original input with complete tracing before presentation/rewards.

## Validation

The fixed checkpoint-party fixture is `data/source/clockbank-search-party.json`.
This samples CRT states at the Clockbank checkpoint, not entire title-screen runs.
The first 128 states from the prior 1,000-state benchmark gave:

| Measure | Previous | Refined |
| --- | ---: | ---: |
| Mean input equivalents | 491.1172 | 454.5078 |
| Mean time changes | 59.2344 | 53.3594 |

52 improved, 76 unchanged, zero worse; mean input cost fell 7.45%.
Examples include 202 to 0 switches and 28 to 2. Runtime and search quality are
separate measurements; the bounded refinement adds computation.

The former 484-switch state's earlier sampled EXP100 candidates were followed by
an effect knocking out Agnea (recorded in `outputs/clockbank-search-witness.txt`).
Do not describe those candidates as valid merely because the multiplier rolled.
The search is not a proof that 484 is globally necessary.

Run the ignored `clockbank_switch_benchmark::sample_switch_counts` release test
with `OT2_BENCH_COUNT`, optional `OT2_BENCH_STATE`, and `OT2_BENCH_OUTPUT`.

## 484-switch outlier audit (2026-10-10)

The 484-switch plan is demonstrably not necessary. With the same checkpoint
world and CRT state1775700753, the offline audit found a fully replayed accepted
plan at276 switches, cost2146 rather than3434. EXP100 executes as BG child3 at
checkpoint+7504; all four survive and Thunder L finishes the encounter.

The isolated limit test holds departure276 and target7504 fixed:

| Local input budget | Guards alternative window | Accepted total cost |
| --- | --- | --- |
|192|12|none|
|384|12|2148|
|192|384|none|
|384|384|2146|

None of these four isolated queries exhausted its100000-unit audit work budget.
The2146 witness needs82 local field inputs plus132 Guards/BG inputs =214,
exceeding the production refinement's shared192-input cap. Baseline fight cap100
also excludes it. Thus local caps can discard a cheaper overall plan and force
hundreds of additional time switches. Earlier death witnesses are only rejected
candidates, not proof that a lottery target is unusable.

Raw EXP100 anchors through11976 are56,294,4280,7504,11976. The expanded4280
audit found no witness within its limits, but some departures exhausted their
20000-unit work allowance; infeasibility is NOT proven.

Audit tests: `audit_484_earlier_anchors`, `audit_484_limits` (ignored, explicit).
Logs: outputs/clockbank-484-expanded-audit.txt and clockbank-484-limit-audit.txt.
Accepted route: outputs/clockbank-276-witness-384-384.json.
These are offline diagnostic searches; production search limits remain unchanged.

## Adaptive total-cost refinement

Production now follows the original refinement with an adaptive rescue pass.
Its player-input ceiling is the incumbent total input cost minus actual switch
cost minus one. There is no separate local192-input ceiling, per-action74-input
ceiling, or650..1900 departure-gap restriction in this pass. All already generated
daytime departures before each target are eligible if their switch cost can win.
The1150/2200/3800 draw gaps ONLY order trials in successive sweeps; they do not
filter departures. Both Cait setup and the fight are simulated from each actual
candidate, with strict earlier field propagation retained.

The computational allowance scales with incumbent cost (30 work units per input,
at most90000). Each target receives at most12000 per sweep, each query6000, so a
hard earlier target cannot consume all effort. The Guards alternative window
remains12 extra inputs; the anchor horizon is the incumbent target at pass entry.
These are bounded search-quality choices, not proofs of global optimality.

The prior484-switch state now yields136 switches, total cost1217, executing the
previously missed target+4280. The regression validates full traced replay,
all four alive, EXP100 victory, and cost accounting. No battle RNG mechanics were
changed. Earlier audit statements that production limits remain unchanged are
superseded by this section.

Adaptive validation: compared with the previous refinement on32 identical states,
4 improved and0 worsened; mean input cost425.40625 ->420.28125. Four-worker test
runtime averaged1.995s, maximum4.179s (experimental release LTO off). Production
uses the normal thin-LTO build; these timings are not a production latency guarantee.
The two accepted-route regressions and sample benchmark passed.
