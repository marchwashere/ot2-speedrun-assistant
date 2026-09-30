# Early clerics and Osvald Cait fork

Active branch: `strategy/abandoned-village-fork`. The old route is on `main`.

The last controllable RNG event for this segment is boosting at the first Flats
encounter after tagging Harbor. Candidate inputs are replayed from the unchanged
first command menu, including Escape or the real single-enemy break fallback.
Every candidate must escape and leave enough normal post-encounter counter for
the remaining run to the first Abandoned Village visit (170 total run updates).

From that entrance, `early_cait::project` runs:

1. Village load/tag and fast-travel tutorial dismissal through shared Loader.
2. Return to daytime Flats. Fresh first-encounter counter, including halving.
3. Safe approach to clerics; night switch, refresh tavern gate/NPC placements.
4. Require flag 25503 AND both actual cleric actors; ordinary soulstone purchases
   consume no CRT. Immediately switch back to day and refresh placements again.
5. Finish the existing 72 walking / 68 running update leg to Southern Cape Cold
   Snows. User confirmed these total distances also cover the cleric stop.
6. Load Snows, walk 70 updates to Osvald, help/recruit him, skip his introductory
   story and dismiss tutorials through imported MS_GAK_00_0100.
7. Predict the first daytime land encounter in the SAME Snows map, with carried
   counter and FirstEncount, actual Throné route level and Lv1 Osvald. Require an
   actual CAT_SNOW replacement enemy and Normal or Preemptive situation.

The two shop switches consume data-derived lottery and NPC creation draws, but
do not reset the counter. Movement stays daytime before and after the shop;
there is no movement while at night. Shop position therefore does not change the
endpoint RNG or total movement cost if the whole leg stays encounter-free.
The existing 42-update cleric approach is used only to split the walking-reference
counter (remaining 30). It adds no distance to the 72-update total. The whole-leg
run percentage uses existing speed-dependent overshoot measurements and remains
an estimate. All safe mixtures share CRT and the destination load resets counter.

Search policies are under the Flats encounter control point, in this order:

| Outcome | API setting |
|---|---|
| Early clerics | early_clerics_max_boost_repeats |
| Safe Flats walk via clerics | flats_snow_max_boost_repeats |
| Safe Osvald walk | flats_osvald_max_boost_repeats |
| Cait without surprise | early_cait_max_boost_repeats |

Required means no candidate budget: all required goals gate Escape/break jointly
before optional improvements. Numeric budgets add repetitions relative to the
selected preceding plan; they never waive required goals or achieved outcomes.
`flats_escape_max_boost_repeats` separately chooses when to try a break;
`boost_only` bypasses that fallback. Displayed inputs come from the selected
replayed path. Earlier Highroad and Pirro adjustments are frozen.

The fork GUI sends `route_end=osvald_cait`. Legacy endpoints `full` (API default)
and `first_village` remain available. The projection is exported as
`post_battle_field.flats_escape.early_cait`; its encounter checkpoints are local
to the post-recruitment RNG and must be offset once when rendering route counts.
The endpoint includes group/situation/party timing/rare selection, and stops
BEFORE enemy actor initialization, Study Foe and opening command-menu draws.

Assumptions: unchanged equipment, no rare bonuses, daytime travel, no intervening
RNG-consuming actions, sufficient money for purchases. Purchases do not yet
update a gold/inventory ledger. Current tests verify input replay, shared loader
state and CRT, no resets on the shop switches, recruitment party/progression,
actual replacement slots, independent policies and exclusion of Cape Cold.
This strategy is source-derived and has not been live-validated.
