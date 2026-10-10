Current policy: after buying soulstones, switch exactly once from night back to
day. Extra post-cleric cycles are removed from prediction and search. Battle
boost/break controls and post-Osvald switches remain available. Older cycle-search
examples below describe historical plans.

The current rare goal is OR, not AND: Cait after recruiting Osvald in
Southern Cape Cold Snows; Cait in Eastern Cape Cold Snows; or one
Eastern encounter followed by a full 167-update run without a second encounter
and a rare in Western Winterbloom. Joint Southern/Eastern boost and zero-BP
break searches use actual input cost, preserving earlier required outcomes.
Regular Octopuff is excluded from Southern/Eastern rare targets only; encounter
generation still includes it. Western Queen Octopuff remains eligible.
An earlier rare stops later rare searching. Every selected formation must die
to one Shadow Soulstone (M), including its companions. Source-derived core
item/voice/victory/drop callbacks carry the shared field return, after which
Osvald learns two skills and equips Evasive Maneuvers. The carried world records
that support for subsequent movement at 50%; it is absent before the rare kill.
New completion and Western endpoints await live validation.

Cait destination now accepts either Southern Cape Cold Snows **after Osvald
recruitment** or Eastern Cape Cold Snows, always without surprise. A Southern
candidate may use the actual safe 70-update walk to Osvald instead of the
64-update run. Preserve required/achieved upstream targets and compare its
adjustment cost against the existing Eastern plan including Southern Escape.
The GUI shows the selected approach. A Southern Cait endpoint stops before
battle initialization; it must not be escaped or assigned invented combat RNG.
The suspended-Cait and Eastern-only descriptions below are historical.

# Early clerics and Osvald Cait fork

Active branch: `strategy/abandoned-village-fork`. The old route is on `main`.

Current goal: Cait manipulation is suspended. Accept any enemy group in the
first Snows encounter, but require Normal or Preemptive (never Surprise).
Clerics and all encounter-free travel targets remain unchanged. The GUI uses
`route_end=osvald_encounter` and `early_approach_max_boost_repeats`; old names
remain API aliases. Cait can be explicitly requested with
`early_require_cait=true` for the retained experimental route below. The default
is false, and its search/required-outcome reporting both use this same goal.

Upstream Highroad control: `highroad_flats_break_max_boost_repeats=required`
jointly targets a safe harbor walk, an unsurprised Flats opening and an actual
unboosted first-action break. The daytime progress1 shared encounter table has
weight20 each for lone City Ratkin I and lone Night Raven (40 of100 base weight).
Both have one shield and dagger weakness. Rare replacements, multiple enemies
and Surprise do not qualify; the core replays the hit, rejecting a Night Raven
that dodges the unboosted attack. Numeric limits are optional additional input
pairs and preserve earlier required/achieved outcomes. GUI defaults Required;
API omission retains legacy behavior (optional0).

Pirro R8 now also targets an unboosted Highroad break. Both field encounters
compare direct Escape and actual break plus R2 boosts by time cost. See
[weighted escape strategy](weighted-escape.md); it supersedes the historical
Boost only regression counts below.

With seed0/title4916, shared level retained, Required earlier targets and Flats
Boost only, enabling this target selects21 Highroad repeats, a Normal lone
City Ratkin I with a successful unboosted dagger break available, and61 Flats
repeats (previously123). All required cleric/travel/non-surprise targets pass.
This plan still uses direct Flats Escape due to the saved Boost only preference.

Historical seed-0/title-4916 regression before adding the Highroad break target: retained shared level and Required targets,
Flats Boost only select 123 boost repeats, the normal one post-purchase switch
back to day, and two post-Osvald switches ending day. Weighted cost274; first
Snows encounter is Normal and has no Cait. The remaining 123 repeats satisfy
Escape/counter/clerics targets; they are not a Cait adjustment.

Exhaustive candidate audit for that configuration (0..123 inclusive): 71
candidates fail first Escape, 34 more fail the remaining village run, leaving
19 distinct valid nighttime shop checks. Of these, 17 have no clerics. At120,
clerics spawn but the returned Flats counter is560, below649 required for the
72-update walk. At123, clerics spawn with counter660, leaving12 after the walk.
The shop check is therefore conditional on earlier Escape and counter safety;
123 boost repetitions are not123 eligible independent cleric checks. The
imported weights are50/20/15/15; native FUN_140ea7520 draws inclusive0..100 and
uses <= cumulative comparisons, selecting clerics for86..100 (about14.85%).
The current route searches time changes AFTER purchases; it cannot reroll a
failed cleric selection at the shop before buying.

Replay the audit against an Export JSON response with:
`cargo run --offline --example audit-early-clerics -- <export.json> 123`.
It replays each candidate from the same first command menu through the shared
loader and reports stage rejections, actual shop rolls and counter values.

The three controllable RNG events are boosting at the first Flats encounter,
time cycles after buying from the clerics, and time switches after recruiting
Osvald. Candidate inputs are replayed from the unchanged
first command menu, including Escape or the real single-enemy break fallback.
Every candidate must escape and leave enough normal post-encounter counter for
the remaining run to the first Abandoned Village visit (170 total run updates).

From that entrance, `early_cait::project` runs:

1. Village load/tag and fast-travel tutorial dismissal through shared Loader.
2. Return to daytime Flats. Fresh first-encounter counter, including halving.
3. Safe approach to clerics; night switch, refresh tavern gate/NPC placements.
4. Require flag 25503 AND both actual cleric actors; ordinary soulstone purchases
   consume no CRT. Switch back to day, optionally performing more complete
   day/night cycles, and refresh placements on EVERY switch.
5. Finish the existing 72 walking / 68 running update leg to Southern Cape Cold
   Snows. User confirmed these total distances also cover the cleric stop.
6. Load Snows, walk 70 updates to Osvald, help/recruit him, skip his introductory
   story and dismiss tutorials through imported MS_GAK_00_0100.
7. Optionally toggle time after recruitment, refreshing the actual new party's
   placements on each toggle. Predict the first land encounter in the SAME
   Snows map using the final DayTime/Night table and decrement, with carried
   counter and FirstEncount, actual Throné route level and Lv1 Osvald. Require an
   Normal or Preemptive situation. Only when explicitly opting into Cait also
   require an actual CAT_SNOW replacement enemy.

Shop switches consume data-derived lottery and NPC creation draws, but
do not reset the counter. Movement stays daytime before and after the shop;
there is no movement while at night. Shop position therefore does not change the
endpoint RNG or total movement cost if the whole leg stays encounter-free.
The existing 42-update cleric approach is used only to split the walking-reference
counter (remaining 30). It adds no distance to the 72-update total. The whole-leg
run percentage uses existing speed-dependent overshoot measurements and remains
an estimate. All safe mixtures share CRT and the destination load resets counter.

The optimizer enumerates increasing weighted cost: 2 per cancel/reboost pair,
7 per individual time switch. Extra cleric switches are even so the trip stays
daytime; post-Osvald switches can be odd or even. The mandatory initial shop
night switch and return-day switch add a constant 14. Ties prefer fewer switches,
then fewer after the clerics. Every state is cloned and projected through the
shared Loader; rejected candidates do not consume RNG in the chosen route.
Immutable projections are cached across cost layers. The first selected layer
minimizes estimated manipulation cost across all three controls (not the full
timing of a configured break/Escape fallback).

Historical opt-in Cait regression example: seed 0, title advances 4916, shared level retained,
Required route outcomes and Flats Boost only select 123 boost repetitions,
23 post-purchase switches (including the normal return to day), and 4
post-recruitment switches. Cost: 246 boost inputs +28 switches *7 =442 units.
The final daytime encounter contains Cait + Snow Fox and is Preemptive;
composition is finalized at advance 6351 / 0x9C0C45D1. This replaces the old
fixed-phase 4807-repeat plan. The imported Snows volumes specify a base Cait
rate of 2% for both day and night; the joint search also has to preserve the
earlier counter, Escape and shop outcomes.

Search policies remain individually configurable:

| Outcome | API setting |
|---|---|
| Early clerics | early_clerics_max_boost_repeats |
| Safe Flats walk via clerics | flats_snow_max_boost_repeats |
| Safe Osvald walk; post-purchase cycles | flats_osvald_max_boost_repeats |
| First encounter without surprise; post-recruitment switches | early_approach_max_boost_repeats (old alias: early_cait_max_boost_repeats) |

Required means no candidate budget: all required goals gate the combined plan
before optional improvements. Numeric budgets add 2 WEIGHTED input-cost units
per allowance relative to the selected preceding plan (4 =8 units); a complete
extra shop cycle costs14, one post-Osvald toggle costs7. Legacy API names retain
"boost_repeats", but budget scale now includes time costs. They never waive
required goals or achieved outcomes. Settings are grouped by their latest control
point: Flats encounter, after cleric purchases, after Osvald recruitment.
`flats_escape_max_boost_repeats` separately chooses when to try a break;
`boost_only` bypasses that fallback. Displayed inputs come from the selected
replayed path. Earlier Highroad and Pirro adjustments are frozen.

The fork GUI sends `route_end=osvald_encounter`. Legacy endpoints `full` (API default)
and `first_village` remain available. The projection is exported as
`post_battle_field.flats_escape.early_cait`; its encounter checkpoints are local
to `after_osvald_switches` and must be offset once when rendering route counts.
The endpoint includes group/situation/party timing/rare selection, and stops
BEFORE enemy actor initialization, Study Foe and opening command-menu draws.

Assumptions: unchanged equipment, no rare bonuses, daytime travel, no intervening
RNG-consuming actions, sufficient money for purchases. Purchases do not yet
update a gold/inventory ledger. Current tests verify input replay, shared loader
state and CRT, no resets on the shop switches, recruitment party/progression,
actual replacement slots, independent policies and exclusion of Cape Cold.
This strategy is source-derived and has not been live-validated.
# Mandatory encounter safety and continued travel after a Southern rare

Every expected encounter must be Normal or Preemptive. This is a hard rule,
independent of numeric optional budgets. Search at the preceding controllable
event; boost inputs at an encounter cannot repair its opening situation.

A Southern rare completes the rare target, but not the travel route. Replay
Shadow Soulstone (M), actual victory/drop callbacks and field restoration.
Carry prior route EXP plus the rare reward/earned EXP bonuses into both party
levels; learn two Osvald skills and equip Evasive Maneuvers. The next Eastern
map load, encounter selector, actor initialization and Escape use these new
levels and the 50% movement support. Reject upstream Southern rare candidates
that would cause an Eastern surprise. Search minimum-cost successful Escape;
show its instructions even when zero adjustment is needed. Western rare
selection is absent from this branch.

For the daytime 167-update Eastern run, Evasive Maneuvers truncates decrement
to 6. A fresh half counter triggers the first encounter at updates 84..150;
the maximum remaining cost is 83*6=498, below the minimum normal post-battle
counter of 1000. Thus running the rest cannot cause a second encounter.
Night branches still evaluate their actual decrement/counter instead of
assuming the daytime guarantee. New continuation endpoints await retail tracing.
