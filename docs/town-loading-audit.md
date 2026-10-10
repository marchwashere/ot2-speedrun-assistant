# Town-loading scope and robustness audit

Audited 2026-09-27 against the cooked data and native decompilation.

2026-09-29 package enumeration fix: cooked names are not consistently cased.
Resolve package names case-insensitively, as UE FName does, while preserving
actual asset names in the catalog. Exact `_Sound` lookup omitted Snows'
`Fld_Snw_1_2_sound`, losing its streamed-level color draw and shifting the final
counter. Read-only live level/NPC enumeration and the reported 6135 endpoint
confirm this omission. The generic correction also imports
`Twn_cty_1_2_B_Coll`; earlier route maps are unchanged. No map-specific RNG offset.

2026-09-29 extension: Eastern New Delsta's two canoe Blueprint actors also
consume CRT in BeginPlay to randomize the initial bobbing time. The importer
reads those actor instances in the owning Level's actor-array order; the loader
draws after streamed-level construction and before NPC placement on a fresh
load. It also runs these callbacks when the resident field is restored after
battle, before counter B/A. The 6002->6012 Highroad Escape capture verifies
both actor identities in this second path; level residency does not suppress
their BeginPlay callbacks. Field restoration excludes battle-level actors and
does not repeat streamed-level constructor or NPC placement draws. See
`source/canoe-initialization.json` and `source/live-highroad-escape-6002.json`. The post-Pirro
capture matches all62 world calls (5900 -> 5962), including both actor identities.
Level/NPC enumeration alone is insufficient for new zones; audit BeginPlay
callbacks for other CRT-consuming environment actor classes when adding them.

## Current guarantee

The fixed profile has been replaced by `src/world.rs`. The importer supplies
478 unfiltered placements across the sewer and six New Delsta zones, available
streamed packages, all 33 flag-lottery rules and fifteen Throné event scripts.
No per-map NPC count or total draw count is a runtime input.

`src/city.rs::after_second_victory` constructs the known fresh-route world state
by following 0100..0600 event branches, then executes 0700 and its serial 0800
continuation. `with_world_state` accepts an existing state; `world_after` carries
flags, party/bench, active scenarios, NPC actors, level residency and the counter
to the next transition. Bootstrap consumes no RNG from the supplied boundary.

The parlor continuation carries this state into 0900, then the explicit Yes
branch of 0910, then serial 1000/1100. Its complete 50-call capture also matches,
including all 24 NPC identities. See [the parlor replay](parlor-live.md).
The second visit evaluates 1200 through serial 1300's Steal tutorial dismissal,
including scripted Day-to-Dusk refresh and event cleanup. Its +43 result is source-derived and
awaits live validation; see [the second visit](parlor-revisit.md).

Every draw is evaluated and attributed to a callback, level or NPC. Skipped
branches are also recorded. The result still matches every caller category and
CRT state in the 110-call city capture, including all 85 NPC identities in order.
See [the route](throne-city.md).
This validates that route; it does not imply every town or arbitrary save mode
has been imported and live-validated.

## Black Market and other global flag lotteries

`Flag/Database/FlagRandomControlDB` has 33 rows: 18 use `eLevelChange`,
15 use `eTimeZoneChange`. `GMK_BLACKMARKET_CONTROL` is in the latter group,
with no begin/end flag gates. Its candidates are flags 25500..25503 with
weights 50, 20, 15, 15. These are weights, not an assertion of exact probabilities.

The Black Market row consumes **one CRT draw when the eligible time-change
callback executes**, regardless of the player's map. It is not an unconditional
draw on every map load. In the captured New Delsta sequence, Night is set to
Night again, so there is no time-change lottery and no extra Black Market draw.

Native sources in `Octopath_Traveler2-Win64-Shipping.exe.c`:

- `FUN_140eaeac0`: binds the level/time-change delegates.
- `FUN_140eaec70`: level-change callback, trigger 0, with runtime suppression.
- `FUN_140eaecb0`: time-change callback, trigger 1, with runtime suppression.
- `FUN_140ea7520`: evaluates the rules and performs the CRT lottery.
- `FUN_140f6aa40` / `FUN_140f6eb70`: time transition processing. Equal old/new
  time returns without the lottery event. Other phases and runtime gates affect
  event 0x27 dispatch; do not infer dispatch solely from a requested time value.

For each matching row, the native lottery checks the begin flag (zero or on)
and end flag (zero or off). It sets positive candidate flags to the inverse of
`ApplyValue`, sums candidate weights, draws an inclusive integer from 0 through
the total using CRT-backed float32 scaling, then applies the first positive flag
whose cumulative weight is at least the roll. Later rules observe updated flags.
The importer preserves cooked row order, matching the native sparse-element
walk in `FUN_140ea0270`. A regression exercises a later rule opening an earlier
rule's gate: the earlier rule must wait until the next callback.

There is no single-candidate optimization: an eligible 100%-weight rule still
draws. For example, `SUB_TSe21_0100_GRAPE` consumes one draw on a level-change
callback when flag 20313 is on and 20314 is off, and sets flag 20314. That changes
eligibility for later callbacks. A fixed per-map count cannot model this.

## Implemented evaluation and remaining coverage

The evaluator belongs to the assistant, separate from the battle crate. It uses:

- Full relevant flags, main/side-story and event progress, including mutations
  made by flag lotteries and scripted transitions.
- Current/destination maps, old/new time phase, transition path and event stage.
- Party/bench membership and the map's tavern gate. All imported New Delsta
  maps have `BarEventLabel=None`; that actual gate suppresses the lottery.
- Persistent NPC state and existing actors, distinguishing spawn from refresh.
- Resident streamed levels, including shared levels, to distinguish fresh
  initialization from reuse.
- Encounter configuration and counter-reset callbacks even on non-encounter maps.

Progression alone is insufficient: different party composition, NPC state, time
and level residency can produce different draw sequences at the same story point.

The runtime executes source callbacks and writes flags immediately, using the
existing CRT float32 integer-range helper. Map callbacks reset FirstEncount
before rolling B/A. Event resets use EventList.EncountReset and actual FinishPre
commands. NPC refreshes retain existing actors and do not repeat spawn draws.
Shared levels already resident do not repeat constructor draws.
Outgoing field-owned levels, including BattleMapNameList entries, are unloaded
before destination loading even when the destination uses the same package.
Only separately retained shared levels bypass this unload cycle.

The importer checks FlagOverrideDB against every queried placement/lottery/branch
flag. None intersect in this data set; importing a new dependency fails instead
of silently ignoring override semantics. Unknown event commands/maps and
dawn/dusk transitions other than scripted Day-to-Dusk and Night-to-Dawn fail explicitly.
Tavern appearance/member/item setup draws now follow imported per-map data,
including hidden placeholders for an empty bench; see `tavern-loading.md`.

Remaining coverage: other towns and their special callbacks, tavern conversations
and receiving gifts, other dawn/dusk transitions and timer-driven time changes,
theater/reminiscence and darkness save modes. Do not advertise
the API as a complete arbitrary-save emulator.

## Validation needed before expanding the guarantee

The existing +110 capture is a per-call regression. Automated variants cover
the off-map Throné flag (-1 draw), shared-level recreation (+1), retained NPC
refreshes, single-outcome progression rules, ordered flag mutations, same time
versus day/night changes and continuation into the next city zone.
Add per-call live fixtures for
fresh load versus revisit, resident versus recreated shared level, both time
directions versus unchanged time, active versus completed flag-lottery quests,
changed NPC save states, and tavern maps with different eligible bench rosters.
Test identical starting CRT states under different world states. Compare caller
sequence, flag/actor changes and intermediate CRT boundaries, not just final
draw totals. No finite capture set alone proves every scenario; coverage must
also follow the relevant native branches and cooked data.

## Conning Creek hidden backdrop correction (2026-10-09)

The re-entry capture `outputs/conning-load-live-20261009.json` includes an
outgoing trip followed by a manual CRT rewind. Only its final 79-call segment,
19,287,194 -> 19,287,273, is the Conning load fixture. It contains four counter
calls, six streamed-level constructors, 33 field environment callbacks (two
flags and 31 lamps), 33 NPC frame calls, and three tavern calls. The latter are
one appearance lottery and two placeholder item rolls; there are no eligible
benched-character selection rolls.

The previous loader also executed the hidden `Btl_Twn_Sea_B` backdrop's two
flag and twelve lamp BeginPlay callbacks. Native `FUN_140ed98a0` queues both
commands 1 and 3 for field streams, but only command 1 for BattleMapNameList
streams. Construction and showing must not be conflated: preserve the battle
stream's constructor draw and residency, but execute environment initialization
only for newly created field streams during town loading. Keep battle actor
data available for battle activation.

Those 14 premature calls also changed the tavern lottery from three calls to
one, producing the observed net excess of 12. The regression compares every
call's state, category and NPC identity and checks both first-visit and revisit
world states; both now end at 19,287,273. This is a lifecycle fix, not a fixed
Conning-specific subtraction.

Follow-up native audit: queue consumer `FUN_140ed4030` dispatches command1
to `FUN_140ed22a0` (requests stream loading via vtable+0x270) and command3
to `FUN_140ed8640`. The latter requires a loaded level at+0x128 and invokes
`FUN_142c78250(...,1)`, setting visibility bit3 at+0xba. Field/battle transition
`FUN_140eda7e0` separately queues show3 for the selected battle backdrop and
hide4 for the field/other backdrops. These branches distinguish residency from
visibility, not story progression. A preloaded backdrop can therefore consume
constructor RNG without yet executing its environment BeginPlay RNG.

The dumped lamp ReceiveBeginPlay enters its graph at235 ->151 and calls
RandomFloatInRange(0,3), then SetNewTime. The flag graph calls
RandomFloatInRange(0.9,1.1), then SetPlayRate. Neither graph contains a story
flag branch. Story can affect which map/actors are selected elsewhere, but does
not gate these two RNG callbacks themselves. Do not generalize this to every
environment Blueprint. `tools/audit-stream-load-show.cjs` checks this native/
Blueprint chain. The captured town still executes its own two flag and31 lamp
calls; only the hidden battle backdrop's14 calls are excluded at town entry.
