# Weighted Escape strategies

User timing model: every boost up/down input costs1, the additional break action
costs10 input-equivalents, and a time switch costs7. A required BP up/down press
still costs1; the ten-unit break cost covers the attack/extra turn. Direct Escape
and break→Escape share the final Escape action, so its common cost is omitted.

Pirro R8 now has independent `highroad_break_max_boost_repeats` Required/numeric
policy, alongside Highroad non-surprise. It targets the real rare-replaced
formation: one City Ratkin I or Night Raven, non-surprise, with a successful
unboosted dagger break. Both Required R8 goals are searched jointly, preserving
selected R3 inputs, victory and no-grape. Optional caps add input pairs to the
selected required plan and preserve achieved goals. The existing Highroad→Flats
unboosted-break target remains independent.

`escape_search::FieldSearch` enumerates complete candidates by increasing cost:

- R1 boost/cancel→direct successful Escape.
- R1 boost/cancel→actual single-enemy sword/dagger break→next-menu boost/cancel
  →guaranteed Escape. Compare both R1 and R2 allocations.

The core executes the actual hit, break callbacks, skipped enemy turn, BP
regeneration, next-order rolls and voices before exposing the next menu. An
unboosted R1 attack retains initial1BP and regenerates to2BP. A 1BP R1 attack
spends it and suppresses regeneration, leaving0BP; no next-menu boosting is
allowed. Each candidate clones battle/voice state and replays legal BP inputs.
Cancelled boosts consume their real voice rolls and do not spend BP. Broken
enemy Escape bypasses its probability lottery. Field restoration and all later
map loads use the shared Loader, never fixed offsets.

The separate Mansion adapter compares actual direct/break battle snapshots with
the same cost model. Its normal first-menu1BP dagger break leaves no BP at the
next menu. Any later first-menu BP is evaluated from actual state. Both plans
retain the configured no-second-encounter target and carry actual resources to
Pirro. Its route distance remains161 running updates.

Flats jointly searches both battle menus, a fixed post-purchase switch to day and
post-Osvald time switches in weighted cost layers. Every required outcome gates
each candidate: Escape, safe village run, clerics, safe Flats→Snows walking,
safe recruitment and a non-surprise first Snows encounter. Numeric optional
outcome budgets remain additional input-equivalent pairs. Boost only explicitly
excludes breaking. Former numeric *strategy* thresholds are compatibility
aliases for enabling the minimum-cost break comparison; their obsolete numbers
are hidden in the GUI. A one-time migration enables breaking for the requested
strategy without altering outcome budgets.

Current seed0/title4916, shared level retained, Required early-route goals:
Mansion direct Escape costs10 boost inputs; Pirro R8 uses `[1,2,3,2,3,2,3]`;
Highroad chooses1BP break, cost11, with no R2 boosts; Flats chooses direct Escape
with7 boost/cancel pairs. Flats continuation cost28 includes14 boost inputs and
the mandatory shop night/day switches (14). All Required goals pass. A separate
regression proves a 0BP break plus6 R2 inputs wins a target at cost16, beating
equivalent R1-only adjustment. Plans are source-derived, pending live validation.

The optimizer is local to each latest controllable event. It does not globally
trade extra Pirro R8 inputs against every downstream plan. Each event freezes
the upstream plan and minimizes its complete own configured outcome cost.

Southern Cape Cold Snows now searches its own opening Escape/break inputs for
Cait plus no surprise in the first Eastern Cape Cold Snows land encounter.
Each trial propagates field restoration, actual recruitment when needed, safe
onward travel (+2 updates per speed), shared map loading and encounter generation.
The current progression-1 day/night tables both have FirstEncount=0 and a 2% Cait
rate: the first base group is weighted, and rare replacement remains eligible.
Selected timing branches are independent complete plans. The search preserves
first-try Escape and safe onward movement, minimizing input cost +10 per zero-BP
break. It assumes no intervening time changes or RNG-consuming actions in Eastern.
The predicted Eastern endpoint stops before battle initialization; no Cait battle
or post-Cait route is simulated yet. Retail verification remains outstanding.

The early route now includes Gold Ring (Brothel Receptionist, Backstreets) and
Antique Coin (Boy, New Delsta main) after Artist's three soulstones. The former
uses the user's 80% chance with StealItemExec's inclusive 1..100 draw; the latter
has ProperSteal=-15 and is guaranteed at the route's level. Both NPCPurchaseData
rows specify generic success dialogue (StealSuccessTalkID=None). Their single
menu sessions use action-timer, success-check and closing-dialogue draws.

Second-fight manipulation compares both Ring/clothes orders at every candidate,
requiring Ring success plus no surprise in the mansion. Equal-valid-cost orders
prefer Ring first. Preserve the original battle policy and achieved fight-two
order/action targets. Failed trials are never carried into selected execution.
Mansion entry, lookout, escape, Pirro and every later route step consume the
selected real state, including any changed RNG-driven world flags. Sale valuables
are carried in route acquisitions, outside the combat consumable catalog.
Historical captures without the two added steals are separate regression inputs;
new route endpoints await retail verification.
