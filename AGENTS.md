# Active project

- User suspended Cait manipulation. Default early-route goal is any first Snows
  encounter WITHOUT surprise; preserve clerics, safe village run, safe Flats
  movement and safe daytime Osvald recruitment. GUI uses osvald_encounter and
  early_approach_max_boost_repeats; old endpoint/budget names are aliases and
  stored numeric/Required preferences survive. early_require_cait=true is an
  API-only opt-in for retained Cait tests. Do not silently require Cait or drop
  the non-surprise goal. Time controls and weighted optimization remain active.

- Early Cait fork now permits post-purchase time cycles (end DAY) and post-Osvald
  toggles (end either DAY or NIGHT; user confirmed both). Minimize extra boost
  inputs +7 per actual toggle, including the constant two shop toggles. Weighted
  cost layers compare all allocations across these control points; Required
  goals gate every candidate. Immutable route projections are cached. Numeric
  optional allowances keep their two-input-unit/pair scale but now bound extra
  WEIGHTED cost (4 =8 units), not merely boost repeats. Settings are separated
  into Flats encounter, After cleric purchases and After recruiting Osvald.
  Replay every shared time lottery/bar/NPC refresh. Toggles preserve counter and
  FirstEncount; final encounter uses the actual DayTime/Night volume and movement
  decrement. Separate post-recruitment trace precedes encounter checkpoints;
  offset local encounter counts from after_osvald_switches exactly once.
  Old fixed-phase 4807-repeat prediction is superseded by weighted search.

- Active branch: `strategy/abandoned-village-fork`. GUI now selects
  `route_end=osvald_cait`: first village tag/return -> Flats night near clerics
  -> buy soulstones -> immediate DAY -> Southern Cape Cold Snows -> recruit
  Osvald, skip story/tutorials -> first land encounter IN Snows. No Cape Cold.
  `src/early_cait.rs` replays shared Loader callbacks and imported encounter
  lottery. Search at Flats Escape jointly requires safe village run, clerics,
  encounter-free 72 walking/68 running Flats updates, 70 walking updates to
  Osvald and an actual Cait replacement with Normal/Preemptive opening. Each
  target has independent Required/numeric optional policy across the three
  control points. Numeric misses retain required/achieved outcomes. Shop
  time switches do not reset the counter; shop location only splits the safe
  total movement budget. Confirmed same total distances, day after buying.
  No rare equipment/support bonuses; party actual Throné route level + Lv1
  Osvald. Endpoint is composition finalized BEFORE enemy actor initialization.
  No Cait battle simulation yet. Source-derived, awaiting live validation.
  See docs/early-clerics-cait.md. FirstVillage and Full APIs remain compatible.

- Previous fork boundary (still accessible via API): `main` preserves the complete
  previous strategy. This fork's GUI sends `route_end=first_village` and stops
  after the FIRST Abandoned Village entry/tag and tutorial dismissal, before
  returning to Flats. Flats Escape search requires escape and a safe village
  run only; it must not gate on Snows, Osvald, Mug or later outcomes. Shared
  Loader carries the endpoint's CRT and world state in `flats_escape.village_visit`.
  Future settings are hidden without deleting stored preferences. Full-route
  APIs remain available with `route_end=full` (the API compatibility default).
  The following older notes describe the preserved full strategy.

- Crackridge first encounter is IN Fld_Wld_1_2 (Crackridge Harbor: Anchorage),
  ENT_FLD_WLD_1_2_LND, daytime progress1/danger5. User confirmed location.
  crackridge::plan first compares day switching in Flats after cleric purchases
  before fast travel versus Harbor after travel, preferring established Harbor
  if safe. Only if both Surprise, replay Mug R2/Cape Cold weighted search jointly
  for clerics and no surprise. Earlier plans are frozen. New independent
  crackridge_approach_max_boost_repeats defaults Required; numeric optional
  ceiling adds2 weighted input units per repeat above clerics baseline. Core
  new_party uses actual Throné route level + Lv1 Osvald, first traveler and
  Study Foe per DISTINCT enemy type. Search minimal boost cycles for first
  successful Escape (mandatory), restore field with shared Loader. Actions
  only in Manipulation, CRT checkpoints/trace in Detailed log. Source-derived,
  not retail-validated. See docs/crackridge-voyage.md and core party audit.

- Added crackridge::after_market: ordinary cleric purchases -> fast travel to
  tagged New Delsta Harbor at night -> immediate day switch with shared NPC
  refresh -> SC_LinerShip_Cty_to_Wld (MapID189 ocean Fld_Ocn_1_1) -> cmd500
  MapID85 Fld_Wld_1_2 -> FinishPre counter B/A (EncountReset=true). Importer
  includes both new maps, placements, streamed levels and event commands.
  Source audit tools/audit-crackridge-voyage.cjs and docs/crackridge-voyage.md.
  GUI appends crackridge_voyage after selected black_market route and shows
  its endpoint/checkpoints in Detailed log only. No fixed draw offsets. Assumes
  skipped voyage, sufficient money, no encounter/path actions. Awaiting retail
  validation. Shop transactions do not track inventory/gold in WorldState.

- Cleric approach measured40 counter updates at either speed from daytime
  Flats entry (from Abandoned Village) to the cleric spawns. black_market::Route
  exports cleric_reach and at_clerics; use42 updates (40 measured +2 player-error
  leeway), zone-derived walk9/run12 =>378/504 cost. Mixed budget counts whole
  running updates, reserves1point.
  Carry the walking reference counter through night switch; safe mixtures share
  identical CRT. Manipulation shows start/destination, run/walk limit and night
  switch instruction; diagnostics in Detailed log. Supersedes unspecified-walk
  notes below. Pirro Manipulation no longer displays early/late latent labels.

- Repeat budget unit changed: one repeat = cancel/reboost pair, TWO extra
  button presses beyond normal action boosting. extra_repeats=(len-finalBP)/2.
  Larger climbs get no budget discount. All shared capped searches and Pirro's
  conditional-voice BFS use this unit. Minimum-input paths prefer single-level
  repeated boosts before minimizing instruction lines. For15 total CRT calls,
  finalBP2: [1,2,1,2,1,2,1,2,1,2,1,2,1,2] =14 presses,6 extra pairs.
  Numeric defaults stay4 (=8 extra presses), so capped choices may change.
  Settings explain this definition. This supersedes all old climb-count notes.


- Clerics search now jointly enumerates Mug R2 boost inputs and post-Mug Cape
  Cold time switches. Objective: button presses +7 per individual time switch,
  INCLUDING village day/final Flats night switches. Odd Cape switch count ends
  daytime: skip village time change AND its placement callback entirely.
  Shared loader executes each toggle and NPC refresh; never fixed CRT offsets.
  Search increasing weighted-cost layers, prefer fewer Cape switches on ties.
  Numeric cleric setting caps boost repeats only; rotations remain candidates.
  Entry6154 now selects +2 boost draws [1,2,1,2], one Cape switch to day,
  Mug6231 -> final night6268. Current title4916 is separately verified via GUI.
  Source-derived repeated-time-switch endpoints await retail validation.


- Mansion policies are separately configurable: mansion_run_max_boost_repeats
  is Required(None) by default or a numeric optional counter-search budget.
  mansion_escape_max_boost_repeats is additional direct-Escape boost budget
  beyond the counter plan (default4), then dagger break; None means Boost only.
  Both strategies preserve achieved/required counter safety. Optional safety
  miss exports reaches_pirro=false and stops downstream Pirro prediction.
  Removed automatic prerequisite-budget promotion: later Required choices must
  never silently rewrite other user settings. Current defaults still solve the
  seed0/title4916 full route. Settings expose independent controls; JSON exports
  both settings. Tests cover required/optional, strategy caps, break replay and
  unsafe optional escape blocking downstream prediction.

- Boost path tie-break: shared optimal_inputs/with_cap minimize button presses,
  then grouped instruction lines on the shortest-path graph. Render exact chosen
  path; do not reorder labels alone. Mug +7 now displays 0→1 ×2, 1→2 ×4.


- Black-market route added in src/black_market.rs: Mug R2 explicit boost inputs,
  fast travel to tagged Abandoned Village, switch DAY, enter Flats, then switch
  NIGHT near the clerics. Shared Loader evaluates map/time/NPC callbacks;
  target flag25503 AND both cleric placements0900/1000. Both time switches reroll.
  No approach distance given: assume no intervening encounter and do not treat
  carried counter as remaining walking budget. Source-derived, not live-validated.
  Independent mug_clerics_max_boost_repeats defaults Required; numeric optional.
  Search replays entire Mug and route, minimizes total button inputs, preserves
  upstream outcomes, updates the conditional R3 finisher. Baseline entry6154:
  +7 draws via [1,0,1,2,1,2,1,2,1,2], Mug ends6236, final night6268.
  GUI shows boost adjustment only; full checkpoints/trace in Detailed log.
  See docs/black-market.md and data/source/black-market-fast-travel-audit.json.

- Mug completion fixed and live-validated to6299/0xDC86178D. R3 staff hit
  triggers kill voice6293 and delayed JustBreak timer6294 even on a kill.
  Drop6296 -> battle end6297 -> field B/A6299. Osvald attack voice duration
  is5689 ticks, shorter than the9600-tick hit. Tracer detached/registers restored.
  Fixture cape-cold-mug-completion-live.json retains134 continuous calls.


- Enemy initialization now uses imported initial_idle_frames + sync flags, not
  a Mug exception. NPC SnwSm001_Mle lacks BAT_IDLE_ENEMY130; RandomInteger(0)
  makes no CRT call. See OT2BattleEngineRust/docs/enemy-initialization.md.

- Resolved Mug initialization discrepancy: Cape Cold NPC uses BP + timing draws,
  without the ordinary enemy sprite-frame draw. Core derives this from the initial resource and synchronization flags. Replay2 captured 181 calls including reload; last
  continuous segment6154..6191 validates night through first menu6192.
  Tracer detached with original debug registers restored. Compact fixture
  cape-cold-mug-first-menu-live.json preserves callsites and raw-trace SHA256.
  Later battle actions remain source-derived, not live-validated.

- Cape Cold Mug prediction: src/cape_cold_mug.rs and standalone core `mug`.
  User confirmed seed0+6154 on daytime arrival and original-gear Lv1 Osvald.
  Shared set_time(4) MUST be followed by refresh_npcs: black-market1 plus
  eight night placement frames =>6163. Preview timer1 and Rob opening-line
  lottery1 =>6165 battle initialization. Both before-line events and success
  event have EncountReset=false. First menu6192 retail-validated. Later source prediction: menus6218/6289,
  damage53/226/11, victory/drop6297, field B/A return6299 (0xDC86178D). GUI now appends post_battle_field.cape_cold_mug after selected
  Cape Cold arrival via after_cape_cold (no endpoint if town is not reached).
  Detailed log has damage, command-menu checkpoints, final CRT and merged
  chronological field/battle trace. Core Mug counts already include route
  offset: add only crt_setup.battle_entry_advances when displaying totals.
  Manipulation shows only the R3 staff finisher when needed, or an error.
  See docs/cape-cold-mug.md and tools/audit-cape-cold-mug.cjs. General
  per-character EXP/JP ledger absent; current route computes Throné EXP only.

- Manipulation UI rule (2026-09-29): show only instructions that change the
  player's actions for this prediction: actual boost adjustments, break/Escape
  alternatives, early/late latent, and run/walk limits. Hide event cards when
  no adjustment is needed. RNG states/counts, counters, damage-less/encounter
  reports, and other status-only cards belong in Detailed log. Keep route
  events and calculations intact; card visibility must not skip simulation.
  Required-outcome failures must remain visible so an invalid plan cannot look
  complete. This supersedes older Manipulation diagnostic-card instructions.

- Continue completed Osvald recruitment with src/cape_cold.rs to Cape Cold
  (Twn_Snw_1_2_A, map169), carrying party[7,6], skipped-story flags and CRT.
  User measured52 walking/49 running updates; add2 to BOTH =>54*9=486 walking,
  51*12=612 running. Reserve1, interpolate route-distance cost486+126*f,
  round share down0.1% and cost up; label estimate, not exact switch timing.
  All first counters>=500 can walk, so this adds no boost-search target.
  Current title4942/one Flats cycle: post-recruit6132/counter552 -> run51.5%,
  walk48.5% -> Cape Cold6154/0xA3471B82. Shared Loader evaluates5 levels,
  13 eligible NPC frames and4 counter draws; no fixed+22. BarEventLabel=None,
  no additional CRT BeginPlay in actor/parent audit. Endpoint source-derived,
  awaiting retail validation. Expose post_battle_field.cape_cold separately
  after selected Flats plan; do not run extra town loads during boost search.
  Compact running/entryCRT card, full trace/checkpoints in Detailed log.
  See docs/cape-cold.md and tools/audit-capecold.cjs.

- Extend village_return through src/osvald.rs: load Fld_Snw_1_2 with shared
  Loader, then 70 measured walking counter updates to Osvald (daytime9, require
  631). User confirmed help/recruit, skip introductory story, keep him active,
  dismiss tutorials. Execute imported MS_GAK_00_0100 with choices15Yes/61Yes/
  70No; FinishPre refreshes placements and B/A, tutorials66/71 add no draws.
  Carry party[7,6], flags and native integer event variables (7410 add,7420 ==).
  No running measurement, no extra padding, no hypothetical endpoint if unsafe.
  flats_osvald_max_boost_repeats is independent, defaults Required, under the
  Flats encounter control point. Jointly satisfy Escape/village run/both walks
  before optional budgets, gate breaks too and retain achieved prior targets.
  Compact recruitment CRT + safe walk in Manipulation; full callbacks in log.
  2026-09-29 loader fix: package lookup must be case-insensitive like UE FName.
  The cooked Fld_Snw_1_2_sound was silently omitted by an exact _Sound lookup.
  Live inventory confirms seven new levels, three NPCs and retained shared level;
  ec60c0 -> 11c0100 consumes its LevelColor CRT draw before incoming counter B/A.
  Six-cycle route now matches reported Snows6135 (arrival6121, +14). This is
  endpoint/inventory validation, not a full per-call capture. Re-optimized current
  seed0/title4942 route needs ONE Flats cycle: Snows6130/counter727, 70 walking
  updates leave97, recruitment6132/0x0DF53D34. Keep measured distance70 unchanged;
  old one-cycle counter627/69-only conclusion was based on the missing call.
  Recruitment remains source-derived, not live-validated.
  See docs/osvald-recruitment.md and tools/audit-osvald-recruit.cjs.

- Extend Flats Escape through the Abandoned Village tag/return using
  src/village_return.rs and shared Loader (Twn_Cty_1_2_A -> Fld_Cty_1_2).
  Returned Flats has FirstEncount=true; never reuse Escape's normal counter.
  User measured 70 walk / 66 run updates to Southern Cape Cold Snows; add two
  safety updates to each: 72*9=648 walk, 68*12=816 run, requiring 649/817.
  Estimate running DISTANCE with cost=648+168*f, round share down/cost up,
  leave one point, label estimate. End before loading Snows. The same Flats
  encounter boost menu controls Escape, safe village run AND post-return walk.
  flats_snow_max_boost_repeats defaults Required (None), or optional additional
  repeats beyond the actual Escape/village plan. Every direct/break candidate
  must preserve required targets. Preserve earlier route plans. Show a compact
  post-village walking/running card with returned CRT; full callbacks in log.
  Source audit is tools/audit-abandoned-village.cjs. No new BeginPlay CRT calls;
  first-town fast-travel tutorial 62 sets flags without RNG/counter reset, bar
  event None. Endpoints await live validation. Baseline uniform-state odds:
  72.1550324466% for the padded walk; 78.4661458805% for measured70. See
  docs/abandoned-village-return.md and examples/village_snow_odds.rs.

- Search policy update (2026-09-29, user approved): outcome checkboxes are
  **Required outcome**, not Unlimited. Numeric 0..256 budgets remain optional
  improvements. Solve all required targets and their prerequisites together
  at the same control point BEFORE optional improvements; preserve required
  outcomes if an optional budget fails. A required second fight includes safe
  checkpoint walking; required Pirro no-grape includes victory; required Flats
  non-surprise includes safe Harbor walking. Escape strategy thresholds are
  separate: **Boost only** disables the break preference, while a numeric
  threshold tries a complete valid break plan before further direct boosts.
  Every strategy must meet all required route targets. Preserve old saved
  `unlimited` values/API aliases as required (or boost-only for strategies).
  Report unreachable required outcomes explicitly. This supersedes older
  Unlimited/finite-prerequisite descriptions below. See docs/route-searches.md.

- Group Settings by the last controllable event: First battle (boost inputs for
  checkpoint avoidance and the one-enemy-action second fight), Second fight
  (boost inputs for a non-Surprise first mansion encounter), Mansion encounter
  (boost inputs before Escape for first-try success and the remaining run).
  Each card names the searched actions; retain independent target caps and
  Unlimited options. The mansion-approach cap defaults to 4. Searches propagate
  forward from a frozen earlier plan; never spend the first-fight budget to
  solve the mansion opening. src/second_manipulation.rs replays candidate inputs
  through the independent battle core and all town/story/steal/encounter events.
  Preserve any achieved one-enemy-action result, minimize extra button presses,
  then CRT draws, then choose the latest menu. Normal and Preemptive pass;
  Surprise fails. Carry the selected second boost plan into GUI instructions,
  checkpoints, export/replay and the subsequent Escape search. Keep an explicit
  limit-failure fallback. Hide the redundant successful Escape status sentence.

- This repository owns the speedrun assistant, field encounters, route state,
  local GUI, and prediction exports.
- Import the sibling `OT2BattleEngineRust` library through Cargo. Never copy its
  battle implementation or bundled battle data here. The dependency is one-way.
- The battle repository must continue to build and run independently for sweeps.
- All simulation is headless and tracks the main CRT stream only. The viewer is
  a separate inspection layer; no game graphics/audio runtime belongs here.
- Opening GUI/API inputs are at TITLE SCREEN before New Game as Throné, with
  both opening cutscenes skipped. Add the captured 12 CRT draws, or 13 when
  Btl_SinExpose is recreated. This transition belongs to the assistant; the
  battle library still accepts the state immediately before initialization.
  Never add only the optional shared draw or count the transition twice.
- Keep RNG boundaries explicit. Field predictions end after enemy composition,
  before enemy actor initialization, turn order and command-menu voice draws.
- Prefer source-derived rules and captured regression fixtures. Missing data
  must report an error rather than silently approximate an encounter.
- The supported fresh-new-game Throné post-battle frame resolves its map, zone,
  night period, progress 0, FirstEncount, three-member party and flag 25304 off.
  It is valid after MS_TOU_10_0300 and before MS_TOU_10_0400. Never confuse task
  progress 30 with global main progress or draw an extra initial counter reset.
- Other map membership/story flags remain caller inputs until supported.
- Town-loading evaluates persistent world state in src/world.rs, not fixed
  per-map offsets. The fresh Throné initializer follows actual event branches;
  do not union flags from unexecuted branches. Black Market is a time-change flag lottery, not an extra
  draw on every map load. All 33 FlagRandomControlDB rules, callback gates/order,
  party/bench, NPC state and resident levels matter for general support. See
  docs/town-loading-audit.md. Imported data covers the sewer and New Delsta;
  unsupported maps/commands/tavern branches must fail. Keep the live +110 only
  as a regression result, never as a runtime offset or pre-counted stage list.
- The measured post-opening walk is 830 -> 146: 684 points = 57 night updates.
  Repeated user measurements confirm 57 walking updates. Running currently
  estimates the same count, costing 912 (16 per update), requiring 913; this
  is not a measured running minimum. The first counter max is 899. Target
  walking (counter >= 685) when searching boost cycles to avoid encounters.
  For mixed movement, replace at most min(57, floor((counter - 685) / 4))
  walking updates with running, only when counter >= 685. This is an estimate
  pending separate opening running/mixed measurements. UpdatePlayerStep discards
  excess displacement when assigning the current position to PrePlayerPos;
  speed and switch locations can change update counts. Percentages refer to
  assumed updates, not measured distance/time; round down and retain a positive
  counter. Never apply the Flats 61/56 ratio to another route.
  Re-simulate from the chosen player menu through both resets; never model
  battle manipulation by offsetting the final RNG without replaying combat.
  Field encounter forecasts stop at the cutscene trigger if reached before an
  encounter. Opening results separately project MS_TOU_10_0400: two new NPC
  frame draws, then counter B/A, no extra tutorial reset. Mark this +4 projection
  as source-derived/awaiting live validation. Do not project it if walking is
  unsafe, and do not reuse the earlier encounter zone after flag 25304 is set.
- From that reachable checkpoint, project the first guard's Ambush with one
  normal path-action menu opening (+1 CRT timer draw), scripted dialogue (+0),
  0410 counter B/A (+2), then skipped rejoin event 0500 counter B/A (+2).
  Keep the +5 projection source-derived pending live trace validation. The
  final counter comes from the second pair, guests 15/16 rejoin, 8004 is set,
  25304 clears and the regular encounter actor is restored. Do not mutate or
  replace the earlier post-battle field frame; expose separate event boundaries.
- Assess enemy-free opening victories from actual enemy turn reports, including
  misses. Do not change the fixed targeting routine or rely only on order slots.
- Manipulation combines both battles into one compact run view. The
  Run simulation action always searches and applies the optimal first-fight
  boost adjustment for a safe checkpoint walk, then seeks a second-fight victory
  with all three order conditions met and exactly one enemy action. Settings has
  a separate saved field for each EVENT: encounter avoidance and the second
  fight, each defaulting to 4. Never use one global cap for all events or reset
  a cap per action. Establish the cheapest encounter-dodge plan within its own
  cap; report no solution if none fits. Fight two may add its independently
  configured number beyond the actual dodge cost, not the unused allowance, summed over
  the plan. If no combined plan fits, retain the encounter-dodge plan and label
  the skipped second-fight target. Normal action boosts are excluded; 0->2 x5
  has four extra repeats. Count climbs, not individual BP presses or CRT draws.
  Optimize inputs under the repetition constraint; a longer path may use fewer
  repeats. Preserve the selected plan's repeat_cap in replay/export queries.
  Each event has its own Unlimited checkbox, saved independently. Use None/null
  for unlimited, never a large numeric cap. Finite settings accept 0..256.
  Unlimited searches grow past the initial 256-draw window until an optimal
  match is found or all CRT offsets are exhausted; preserve fewest-input ranking.
  A full CRT cycle is 2^32 draws. Capped searches cover all paths permitted by
  their repeat allowance. Exports must preserve unlimited settings and any
  selected plan beyond 256 draws. Keep the previous numeric value when toggling
  Unlimited off. Migrate the legacy
  single setting to fight two only. Replay the
  full route with candidate-specific EXP, levels and resources. A safe first
  counter alone is insufficient for the combined target, but is valid fallback.
  Apply this search on initial page load too. If no adjustment is needed,
  keep the zero-extra-boost plan. Never switch this button to a baseline run or
  require a separate Find adjustment click. Exports replay the selected plan.
  The First battle card contains damage-less status, boost inputs, then popup CRT and
  counter as small diagnostics. Safe running limit has checkpoint-popup CRT
  as secondary information. The final card shows whether the second fight has
  only one enemy action, action/hit counts, and CRT after the fight (after drops).
  The requested order conditions are R1 Throné before right grunt, R1 Scaracci
  before left grunt, R2 Throné before leader; expose individual checks in Detailed
  log and count actual enemy actions, including misses. Double Thrust's two
  damage hits still count as one enemy action. Probability estimates must specify
  their seed population and must not imply route-conditioned odds from uniform
  second-battle entry samples. Do not add separate battle-exit, checkpoint-arrival,
  Ambush/rejoin or second-entry CRT widgets. Keep callback traces, checkpoints,
  action history, resource tables and long explanations in Detailed log.
  Damage-less means victory with zero damage received; an enemy miss may still
  be damage-less even though the separate no-enemy-actions check fails.
  Order the run view by event timing: damage-less, dodge inputs, popup state
  and counter, movement budget and checkpoint state, second-fight exit. Seed advance counts are
  primary; hex states secondary. Show the whole optimized boost routine,
  including normal final boost, as grouped climbs (e.g. 0->2 x2), one per line.
  Label each adjustment with round, actor and command-menu ordinal within that
  round (e.g. R2 · Scaracci · 2nd command menu). Count actual player menus,
  including Defend; exclude enemy actions and restart the count each round.
  Omit x1 and the Extra label. Show explicit reductions when needed and the
  exact action/target afterward. Repeated climbs return to their starting BP
  between repetitions, ending at the last destination. Optimize total added button presses, counting reductions
  and normal final boost; then fewer extra CRT draws, then the latest menu.
  Replay the complete chosen input path in the battle core without adding the
  automatic normal boost a second time. Respect available BP (opening max 2).
  Keep the run view free of repeated setup summaries and explanatory prose.
  Display and copy CRT states in hex only across all viewer screens; keep
  advance counts, damage and encounter counters in ordinary decimal notation.
- Second Chapter 1 battle ENG_EVE_THI_C01_020 follows skipped 0600 after rejoin.
  Assume the user-specified short walk is encounter-free. 0600 starts battle
  inside its rich sequence, without FinishPre: +0 before initialization, NOT
  +2 from its EncountReset flag. The serial/skip handoff stops before cleanup.
  Keep event cleanup separate from battle entry; see data/source/throne-second.json.
  src/second.rs supplies EXP/level/inventory and calls the independent battle
  core. Default party is level 2; a level-up restores HP/SP. Its state and trace
  counts are local to second entry; entry/after route counts are relative to
  first entry, so add second.entry.draws exactly once in the viewer. End after
  drops without inventing field-return resets. Keep second-fight logs in Detailed log.
- After a completed second victory, expose a separate source-derived city
  projection (never change the battle's after-drops boundary). Skip 0700 and
  0800, load New Delsta main then backstreets, dismiss radar tutorial 7:
  +110 CRT, matched by the 2026-09-27 live capture. Includes 11 streamed-level
  draws, 59+26 NPC frames, and seven B/A counter pairs. The backstreets count
  includes the off-map Epilogue_PLAYER_IS_Throne NPC enabled by flag 2206;
  never infer its eligibility from its name. Keep the shared level resident;
  Night->Night and the popup add no draws. See docs/throne-city.md and its
  reproducible source audit. Add only a compact expected post-city-popup
  diagnostic to Manipulation, with the full breakdown in Detailed log.
- The 2026-09-27 live capture resolves the city discrepancy: second drops at
  5264, city +110, radar popup dismissed at 5374 (seed 0). The earlier 5270
  battle exit was wrong. Core fixes gate startup voice on BattleStartEvent,
  use AI ordinary slot 1/charged slot 0 and account for Darkest Night reaction
  rolls. Preserve live fixtures and do not add a fixed minus-six correction.
  All second-battle calls/menus and city caller categories match this capture;
  other route/save states remain source-derived predictions.
- Parlor continuation is: enter parlor -> first cutscene -> run to next trigger
  -> Yes -> two further cutscenes -> New Delsta backstreets outside the entrance.
  src/parlor.rs carries city.after_popup and city.world_after into the same
  world Loader. Import/run 0900, explicit Yes branch of 0910, then 1000/1100.
  0910 has EncountReset=false; 1000 changes Night to Day, invoking the Black
  Market lottery, and FinishPre occurs before its serial 1100 transition.
  Use town-loading for both maps, never fixed +50 or pre-counted NPCs/levels.
  ed9bc0 unloads the outgoing field AND BattleMapNameList even if the next
  field uses the same Btl_Twn_Cty_B. Only the separately held Btl_SinExpose is
  retained. The live replay matches every one of 50 calls and 24 NPC identities,
  5374 -> 5424 / 0x080E80B0. Keep these checkpoints/callbacks in Detailed log;
  Keep mansion entry in Detailed log, without a Manipulation card. Preserve the resulting world state.
- The next parlor visit carries that state through 1200 -> serial 1300 map
  arrival, fixed dialogue and Steal tutorial 31 dismissal. Scripted Day->Dusk refreshes
  existing placements before assigning time, ages timers by one, and does not
  dispatch the Black Market lottery. Flag 8011 enables NPC_TOU_10_0500_D000.
  The prompt-skipped source prediction is 5424 -> 5467 / 0xB8E6F54D (+43), not
  yet live-validated. Evaluate both map loads; never bake in that offset. Keep
  arrival distinct from after_popup: both are 5467 for this route, since 1300
  has EncountReset=false and its fixed dialogue/tutorial adds no draw. The final
  world state sets 8012 and tutorial flags 1029/1329 and clears 514. Evaluate the
  implicit FinishPre placement refresh; do not infer a popup counter reset.
  See docs/parlor-revisit.md; arbitrary time delays are outside this prediction.
- Continue from parlor_revisit.after_popup through src/mansion.rs: steal clothes
  (1400 dialogue/Save popup), Boy's Shadow Soulstone, load New Delsta main,
  steal all three Artist soulstones in one menu visit, then skip 1500 -> 1600.
  Preserve both parlor stages in calculation, Detailed log and JSON; hide their
  compact cards. Mansion entry and lookout Ambush also stay in Detailed log
  and JSON only; do not show a Mansion entry widget in Manipulation.
  User confirmed mansion control at seed-0 advance 5562 / 0xDDCC3032 on 2026-09-28.
  Evaluate map50 and progression, never bake in the +27 from Artist or +95 from
  the second parlor popup. FinishPre refreshes dirty placements before counter
  rolls, including the city dummy enabled by 8014 and five mansion NPCs by 8015.
  Steal preview draws once per menu, attempts once per item even at 100%, and
  generic success dialogue once on closure; clothes uses the story override.
  No extra menu openings or automatic time changes are assumed. See
  docs/throne-mansion-arrival.md for the confirmed endpoint and source evidence.
- Mansion entry now has a separate nested lookout_ambush stage. Open the normal
  menu once, then run 1700; flag 8016 replaces N000 with sleeping placement N010,
  and 25305 clears. The placement evaluator supplies the new NPC-frame draw.
  Expected 5562 -> 5564 / 0x5121903C: menu +1, replacement +1, no success/dialogue
  lottery, no counter reset. Keep 1800 separate and unexecuted. This endpoint is
  source-derived, not yet live-confirmed. Preserve mansion.after_entry/world_after
  at the confirmed entry boundary; carry lookout_ambush.world_after onward.
  The Ambush operation preserves the caller's remaining counter. The approach
  is included in the measured full mansion route below; do not deduct it twice.
- src/mansion_escape.rs continues from lookout Ambush using 161 total running
  counter updates from mansion entry to the Pirro trigger, measured 2026-09-28.
  Original equipment, solo Throné level 2, full HP/SP from 0700/1100 command 2310,
  night talent enabled by 1600 clearing flag 652. Active zone is
  ENT_DNG_CTY_1_1_LND at Night/progress 0/danger 3, decrement 12 when running.
  First encounter is one Diamante Guard I. Keep field selection and combat entry
  boundaries separate; party timing was already drawn by the field selector.
  Use the independent core's escape::FirstEscapeBattle; handle normal,
  preemptive and surprise starts, including attacks before the first player menu.
  Search boost inputs for BOTH first-Escape success without attacking/breaking
  and post-return counter > remaining_updates * 12. Escape return rolls B/A
  with FirstEncount=false; do not use the earlier trigger reset or add drops.
  This event has its own repeat cap (default 4) and Unlimited option. The full
  mansion-entry-to-Pirro distance is hardcoded at 161 running counter updates;
  do not expose a setting or API override. Minimize boost button presses then CRT draws;
  end at zero selected BP before Escape. Carry JSON settings, exact input path,
  route-relative trace and world state. Stop before 1800/Pirro cutscene.
  Keep the compact Escape card after the second fight, full trace in Detailed log. This
  stage is source-derived, awaiting live Escape verification; do not call it
  validated merely because continuity/regression tests pass. See docs/mansion-escape.md.

- Continue a successful mansion Escape through src/pirro.rs. 1800 sets 8017,
  then RI_MS_TOU_10_1500 starts ENG_BOS_THI_C01_010 directly: +0 CRT before
  initialization, no map reload, no premature EncountReset/FinishPre. Preserve
  Escape HP/SP; reset BP to 1. Copy second-fight inventory plus mansion's four
  stolen soulstones. The battle core owns Pirro mechanics/routine; this repo
  only orchestrates. R7 chooses the heal branch from the R8 preview. Display
  only boost adjustment and "early latent" / "late latent" in Manipulation.
  Keep outcome, rounds, HP and victory-after-drops CRT in Detailed log/export.
  Omit the known turn-by-turn routine from Manipulation; keep all
  actions/menu checkpoints and source limitations in Detailed log and export.
  Show errors/defeat instead of invented winning endpoints. Preserve the battle's
  after-drops boundary; the separate post-Pirro stage below handles 1900/2000.
  Pirro/Escape live CRT alignment remains unverified;
  see docs/throne-pirro.md. The previous stop-before-1800 boundary describes
  mansion_escape alone; pirro is a separate following stage.
  Pirro's reported seed-0 startup boundary 5595 -> 5631 now matches: +0 serial
  cutscene and +36 core initialization. TalkPlayerID fixes Throné as the start
  speaker, bypassing the random-speaker draw. Do not add a route offset to
  compensate. This endpoint does not validate later boss actions or Escape.
  The 2026-09-28 +5678 capture validates all 203 core CRT calls and six command
  menus, ending this defeat at5881 / 0x1FB4FF1B. Fixes preserve weakness-reveal
  and phase-frame draws and suppress the broken-hit timer when the command's
  ability-camera flag is active. The item builder overrides USE_ITEM's flag.
  No route offset. Winning/boost-search paths still need live coverage;
  do not generalize this captured loss into full boss validation.
  Entry5595 R7 menu is5798: core excludes the normal-enemy full-BP reset and
  reroll from Pirro's scripted AI. Keep serial route propagation; no offsets.
  Latent unlock grants55/80. The no-grape policy checks readiness at R8;
  if unavailable, max-boost swords still occur at R8, followed by R9 latent
  Wind + Ice and R10 sword. Search and exact replay both use this core policy.
  Show "late latent" for that delayed branch, "early latent" otherwise;
  no round label or full routine. Failed searches show adjustment unavailable,
  not an invented latent instruction.

- Post-Pirro prediction in src/pirro_aftermath.rs is wired into the GUI/API as
  post_battle_field.pirro_aftermath and is also available through
  examples/pirro_aftermath.rs. Carry the boss's
  actual after_victory and world_at_entry through return B/A, skipped 1900,
  then 2000/backstreets and map46/Eastern New Delsta Highroad. Never add a
  separate 1800 cleanup. 2000's final normal mission completes scenario700
  before loading backstreets; remove active story/prologue placement gates.
  Its 1550 opt0 is Night -> Dawn, old-phase NPC refresh and one timer tick,
  no Black Market lottery. The 2026-09-29 live replay verifies boss exit5900
  and endpoint5962 / 0x6E0026C2 (+62: 12 streamed levels,34 NPCs,14 counter
  calls,2 canoe BeginPlay calls), counter862. Earlier5959 omitted the boss
  death-slow timer in the core (+1) and canoe initialization in the world (+2).
  Import canoe actors from the actual owning Level actor array, preserving order;
  draw once per actor on level creation and on field restoration after battle;
  the live Highroad Escape below verifies the latter. Never execute presentation.
  All305 battle and62 ending calls match states and caller categories; other
  seeds/progression remain source-derived. No fixed offset.
  See docs/pirro-aftermath.md and tools/audit-pirro-aftermath.cjs. This extends
  the earlier stop-at-drops scope. Preserve data/source/live-pirro-aftermath-5900.json.
  Continue only completed victories, after choosing/replaying the final boost
  plan. Preserve route-relative draw counts; the viewer adds the initial-seed
  offset once. Show Eastern New Delsta's after-cutscenes CRT beneath the compact
  Pirro boost/latent instructions, advances primary and hex secondary. Put all
  callbacks, endpoint and validation note in Detailed log/export. On unsupported
  aftermath display its error while keeping the battle result; no endpoint on defeat.

- Pirro has its own Settings card with independent victory and no-grape caps
  (each defaults to four with its own Unlimited option). Secure the cheapest
  victory within its budget first; no-grape gets its additional allowance
  beyond the actual victory repeat cost, never unused victory allowance.
  Skip no-grape if victory cannot be secured. Keep the winning plan when
  no-grape fails. Legacy pirro_max_boost_repeats/storage migrate to no-grape
  only; victory defaults to four. Exports preserve both caps and exact inputs.
  The victory/no-grape search uses Throné's R3 menu, ending at 3 BP for Darkest Night, for
  victory AND Throné first on R8 / no grape. Freeze the upstream route and
  label target failures explicitly. Use actual button-path BFS with core voice previews:
  the boss-specific 50% max-boost voice makes 2->3 cost three OR four CRT calls.
  Do not use the first/second-fight fixed-cost offset optimizer here. Minimize
  total input count, then actual draws. Export/replay exact pirro_boost_inputs;
  command-menu checkpoints precede them. The displayed path includes the normal
  final boost and names R3 / Throné / 1st command menu. No extra automatic boost.

- src/highroad.rs continues the chosen Pirro victory through the ending loader,
  immediate Day switch, skipped Al introduction and first highroad land encounter.
  Run imported SC_FC13_Tutorial_0100 commands/flags and actual field selection;
  never hardcode the +3 Al projection. Derive level from route EXP and preserve
  unchanged equipment. Highroad selection/startup remains source-derived; the
  6002-menu Escape and field-return calls are live-verified below. Keep new
  encounter info only in Detailed log/export; Manipulation
  shows Highroad Escape boost inputs and its field-return CRT. The first-menu core rejects Surprise
  combat and stops at the Escape check. See docs/highroad.md.
  Add a separate Pirro R8 boost-path search for a non-surprise highroad opening,
  with its own default-four/Unlimited cap. Both early/late no-grape branches use
  3 BP swords at R8; early latent activates before boost, late latent on R9.
  Freeze R3 and replay all later battle/world calls; preserve victory/no-grape.
  Do not spend R3's budget or apply R8 swords to the healing branch. Preserve exact
  pirro_r8_boost_inputs in export/replay. The highroad encounter's Escape search
  has another independent cap and its own Settings card. Escape divides party
  level sum by four; surprise divides by actual party size, never conflate them.
  Highroad Escape now tries boost repeats within its configured threshold first;
  if none succeeds, try a source-simulated immediate break of a single enemy at
  the untouched opening menu, then guaranteed R2 Escape. If no such break works,
  continue the boost search without a limit automatically. Initial Unlimited
  searches directly for Escape. Check actual hit rolls, not just weakness and
  shields. Show the selected boosts or weapon/BP then Escape, no enemy info in
  Manipulation. Keep the R8 surprise budget independent; candidate evaluation
  for that search stops at field selection without running downstream Escape.

- Successful Highroad Escape includes field_return: imported field environment
  BeginPlay callbacks, then counter B/A with FirstEncount=false, after the
  selected Escape attempt. Use Loader::restore_field_after_battle; resident
  levels avoid constructor draws but not these field actor callbacks. Exclude
  battle levels and do not rerun NPC placements. src/flats.rs then uses
  the carried world in Loader.load_map("Fld_Cty_1_2"), assuming no second
  Highroad encounter or intervening CRT actions. Keep both boundaries distinct;
  no fixed +14 load offset and no duplicate return reset. Show field-return CRT
  under Highroad Escape and Flats entry CRT in the next compact card, advances
  first and hex secondary. Include full traces and state in Detailed log/export.
  Do not project either endpoint when Escape is unavailable. Seed0/title4942's
  five-cycle plan is live-verified from menu6002 through five boost draws,
  Escape check6008, two gondola callbacks6010 and counter B/A6012 / 0x2E4773FC,
  counter1448. Preserve data/source/live-highroad-escape-6002.json and its importer;
  regression compares every state, caller category and gondola identity. The
  earlier6010 prediction omitted field restoration's gondola callbacks, not a
  voice or counter reset. Flats now predicts6026 / 0xF0F1E302, counter717;
  this downstream endpoint still awaits live validation. All56 running counter
  updates to harbor cost672 and leave45. See docs/new-delsta-flats.md.
  User separately measured61 walking updates on Flats, costing549 and leaving168
  at counter717. src/flats.rs owns both full-route measured budgets; GUI/JSON
  and examples/flats_trace.rs use these independent counts. Strictly positive
  exit thresholds are550 walking and673 running. Uniform-state odds are
  96.8431569636% walking and62.4802573584% running. Do not infer an exact mixed
  percentage from the two totals. Mansion's161 is already a running measurement.
  Highroad now has a separate Flats-walking allowance, default4 additional
  repeats beyond the actual Escape plan, with independent Unlimited. API key
  highroad_flats_max_boost_repeats. Freeze earlier route inputs; replay R1 boost
  cycles through successful Escape, shared field restoration and Flats loading,
  requiring counter>=550. Keep already-safe plans. If capped search fails, retain
  Escape and explicitly flag the Flats target failure. A safe break fallback is
  retained; an unsafe one can be replaced by a direct R1 Escape candidate within
  this allowance. Do not execute abandoned trial attacks. Preserve settings,
  selected inputs and escape.flats_search metadata through GUI/export/replay.
  Flats displays estimated running fraction of route DISTANCE using separate
  measured densities: cost(f)=61*9*(1-f)+56*12*f=549+123*f. Reserve1 counter point,
  round running share down to0.1%, cost up. Counter>=673 permits100% measured
  running; intermediate shares must say estimate because switch-phase overshoot
  is not simulated. Do not reuse the opening's equal-update-share formula.

- src/harbor.rs continues reachable daytime Flats through an on-foot harbor tag
  and immediate return. Both loads use the same world Loader; import map44,
  evaluate progression/NPC gates and the two ordered harbor gondola callbacks.
  Loader::visit_map records ordinary visit/icon metadata with zero CRT calls;
  scripted load_map callers retain their own visit gates. Do not skip a map's
  initialization because it was visited or shares the previous battle package.
  Harbor is a field, so the first-town fast-travel tutorial is ineligible.
  Returned Flats gets a fresh first-encounter counter. Expose compact return
  advances/hex/counter in Manipulation, both traces/world states in Detailed log
  and post_battle_field.harbor_return. Omit projection when walking is unsafe.
  Seed0/title4942 predicts6026 ->6039 ->6053 /0x04A0B097, counter515. The +13/+14
  totals are evaluated results, never fixed offsets; flag6717 alone enables nine
  more daytime harbor NPC draws. This endpoint remains source-derived pending
  live verification. See docs/harbor-return.md and tools/audit-harbor.cjs.

- Flats -> Abandoned Village uses src/flats_escape.rs, beginning at harbor
  return. Fixed170 running counter updates (user measured~165 plus allowance),
  daytime decrement12. Subtract ceil(entry_counter/12) to get remaining updates;
  require a strictly positive counter on arrival. Ends BEFORE loading/tagging
  the village. No distance setting. Use imported field selection and battle-core
  FieldEscapeMenu/first_action_break, never copy battle logic into the assistant.
  Highroad settings add highroad_flats_approach_max_boost_repeats: four extra
  repeats beyond the actual safe-harbor plan, with independent Unlimited. Keep
  Escape and walking safe; replay both harbor transitions before testing surprise.
  Keep prior plan and flag failure when capped. Do not spend earlier budgets.
  flats_escape_max_boost_repeats is a separate default-four/Unlimited setting:
  limited direct Escape plus safe remaining run, then immediate single-enemy
  break plus safe run, otherwise unlimited direct boosts. Reject a break whose
  reset is too short. Every candidate starts at the original menu; no trial draws
  leak into the final route. Shared field restoration uses Flats callbacks.
  Manipulation gives inputs/run/field-return CRT; full encounter and trace live
  in Detailed log/export. At seed0/title4942 the old Highroad five-repeat plan
  yields Surprise; the new joint plan uses seven repeats and one Flats repeat,
  returning from harbor at6055/counter609 and escaping with1503, leaving75
  after119 remaining updates. New endpoints await live verification. Preserve
  historical Highroad/harbor fixtures with the new approach allowance set to0.
  Baseline second-encounter estimate37.0678% at170 updates (25.9763% at165)
  assumes independent uniform reset starting states, with within-pair CRT
  correlation preserved. Do not present it as exact route-conditioned odds.
  See docs/flats-escape.md and examples/flats_second_encounter_odds.rs.
