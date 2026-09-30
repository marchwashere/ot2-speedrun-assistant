# Clerics after Cape Cold Mug

Route: dismiss Mug loot, fast travel to tagged Abandoned Village, switch to day,
enter Flats, approach the black market without an encounter, switch to night.
The Flats entry → cleric spawn approach is measured at 40 counter updates at
either speed; predictions use 42, adding two for player error. Daytime zone
data gives 9 points per walking update and 12 per running update: 378 or 504
points for the full approach. The predictor keeps a positive counter, reports the maximum
number of running updates and their percentage, and carries the walking
reference counter through the night switch. All safe mixtures consume no CRT.

`black_market::predict` searches Osvald's round-two boost/cancel inputs ending
at 2 BP, replays combat in the standalone core, and passes its field-return
state through the shared loader. No post-battle RNG offsets are substituted.
The minimum total button count is selected; numeric budgets limit additional
boost climbs, Required searches until a qualifying result is found. A numeric
budget failure retains baseline play and is explicitly reported.

The imported GMK_BLACKMARKET_CONTROL row triggers on each day/night change.
The inclusive 0..100 roll chooses flags25500/25501/25502/25503 using cumulative
weights50/20/15/15. Clerics are flag25503, but success additionally requires
both NPC_Fld_Cty_1_2_TALK_0900_N000 and 1000_N000 to be present at night.
The daytime village lottery is superseded by the final nighttime Flats lottery.

Fast-travel source audit is saved in data/source/black-market-fast-travel-audit.json.
BP_WorldMapFastTraveler.OnStartFastTravel resolves destination then calls
SetNextMapInfoByName (native140ed7e40), which writes transition state without
a CRT call. Village has no destination override; its WorldMapTable entry
permits fast travel to Twn_Cty_1_2_A, TriggerID2. Shared map loading retains
progression, residency, placements, environment callbacks and counter resets.

Seed0, Cape Cold entry6154: selected path [1,0,1,2,1,2,1,2,1,2]
adds seven CRT draws over ordinary 0→2. Mug ends at6236 after round two,
and the final Flats night switch ends at6268 with both clerics present.
These new endpoints are source-derived and still require a retail trace.
Regression tests replay exact inputs, verify earlier menus, verify both time
lotteries, and exhaust shorter candidate paths to check minimum input count.


## Combined time/boost search

The optimizer now searches post-Mug Cape Cold day/night toggles jointly with
R2 boost inputs, by increasing weighted cost: boost button presses + 7 per
individual time switch. A full night/day/night rotation costs14. The objective
includes the village day switch and final Flats night switch. Ending Cape Cold
on day skips the village switch completely; map loading uses that carried time.
Every toggle uses shared set_time, tavern gate and NPC refresh; no constant
advance count is assumed. Numeric boost-repeat caps still constrain boost
sequences; time-switch candidates remain available.

Entry6154 now chooses [1,2,1,2] (two extra CRT calls), followed by one switch to
day after Mug, no village switch, and the usual Flats night switch. Mug field
return6231; final night6268. Cost18: four boost inputs and two switches. This
supersedes the boost-only plan above. Tests exhaust lower-cost combinations
and verify both odd/even Cape switch parity and skipped village callbacks.
