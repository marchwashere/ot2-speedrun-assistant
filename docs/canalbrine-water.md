# Canalbrine Bridge water encounter

The route continues from Hikari's recruitment fight, through his replacement
of Osvald, Throné's Scholar setup, and the existing daytime Ryu -> Northern Ryu
Sands -> Canalbrine Bridge loads. The user guarantees no encounter on those
land legs. At the Bridge, the player boards a canoe and takes the first water
encounter. The prediction requires an opening without surprise and a successful
first Escape.

## Two controls, one input-cost search

`src/hikari_manipulation.rs` searches these controls together:

1. Hikari recruitment fight's **second command menu**, before the prescribed
   unboosted default-weapon action.
2. Canalbrine Bridge water encounter's **first command menu**, before Escape.

Both menus start with one available BP, and both actions finish with zero BP
selected. Their legal RNG adjustment is therefore a `0 -> 1 -> 0` cycle: two
button inputs, one CRT draw. The search enumerates increasing total cycle count
across the two menus. Within an equal total cost it tries fewer Hikari cycles
first. It has no optional repeat budget or early fallback that accepts surprise
or failed Escape.

Each distinct Hikari adjustment is projected through the complete battle,
victory, recruitment, Scholar setup, ordinary map loads, boarding and encounter
selection. A Surprise candidate is rejected. For Normal or Preemptive openings,
the cached first-menu state is searched for successful Escape. The first accepted
pair has the fewest combined boost inputs **within these two controls**, starting
from the already selected Ryu arrival. It does not claim to optimize all earlier
route choices. Rejected candidates do not mutate the selected upstream route.

The selected Hikari adjustment and search totals are exported as
`ryu.hikari_search`. `ryu.canalbrine.water` contains boarding, encounter selection,
battle-entry RNG, the Escape adjustment/result, and the real field-return
checkpoint. The Hikari battle prediction retains its command-menu trace.

## Carried battle state

The recruitment fight uses the actual earlier party and resources. After Hikari
replaces Osvald, the active roster is Throné, Hikari, Agnea and Partitio
(`[7,1,8,4]`); Osvald remains on the bench (`[6]`). All five progression records,
EXP/JP, current resources, equipment, learned skills, assigned jobs and equipped
supports are preserved. Hikari keeps his actual newly recruited level and gear;
the water fight does not substitute a four-person average or a preset level.

Throné's Scholar assignment and Evasive Maneuvers remain active. The water Escape
adapter uses the active characters' current battle stats, levels and ordinary
voices, and carries enemy-weakness knowledge. Osvald is benched, so Study Foe
does not run. Initial battle BP follows the shared source initialization rather
than carrying spent BP out of the recruitment fight. Escape awards no EXP/JP.
The successful return performs the normal source encounter-counter reset.

## Water selection and initialization

The active zone is `ENT_FLD_SEA_1_2_WTR`. At progression 5 it selects
`EVM_FLD_SEA_1_2_WTR_DAY` -> `ENC_FLD_SEA_1_3_WTR_DAY_030`, danger level 11.
Day and night share the same three progression tiers (0/3/5 -> levels 5/8/11).
Every tier has `FirstEncount=0`, so the first encounter still draws a weighted
formation. At progression 5 the possibilities are:

| Weight | Formation |
| --- | --- |
| 30% | Blue Betta x3 |
| 25% | Sailfish x2 |
| 25% | Hermit Conch + Rock Tortoise |
| 20% | Sailfish x2 + Flying Fish x3 |

The volume disables Cait and enables the low Harbor Octopuff at a 4% gate before
the shared size-priority replacement step. All formations allow Escape and have
no forced battle-start event or talk actor. Encounter selection consumes the
ordinary group, situation, per-player timing and applicable rare-replacement
draws. The normal enemy initialization, turn-order and voice logic follows.

`UseMapIndex=1` selects `Btl_Fld_Sea_River_A` without a random map choice. Its
embedded startup scripts add no CRT calls. Each ordinary water enemy and Harbor
Octopuff has a nonempty initial idle flipbook; its actual frame count is supplied
to the existing enemy-initialization rules. There is no extra water draw offset.

## Boarding and unmeasured movement

`BP_Canoedock.ChangeToCanoe` sets movement mode 5 and the dock's water encounter
volume. The native move setters update movement, speed and saved-mode fields;
they do not draw CRT or reset the counter. Canoe visibility and pitching use
fixed transforms and a timeline started at zero. This differs from the separate
decorative canoe actor that can draw a random bobbing time during map loading.

The dock suppresses two encounter-step updates while boarding; those are not
RNG advances. First use can show canoe tutorial 6, tracked by list flag 1005 and
opened flag 1305. Its dismissal is CRT-neutral and does not reset the counter.

The water distance has not been measured. The prediction carries the real
Bridge arrival counter into encounter selection but does not invent movement
decrements, the precise encounter position, a remaining running percentage or
a guarantee against a second encounter. With no intervening RNG-consuming
action, movement to the first encounter changes when it occurs, not its CRT
selection state. The post-Escape counter is the actual newly rolled counter.

## Evidence and validation

Run `node tools/audit-canalbrine-water-encounter.cjs` to reproduce
`data/source/canalbrine-water-encounter-audit.json`. It records 18 source hashes,
all three tiers, 12 formations and six enemy initialization profiles, Blueprint
paths and native boarding/canoe evidence. The related complete map audit is
`tools/audit-canalbrine-bridge-loading.cjs`.

The joint-search regression independently replays cheaper allocations, verifies
the selected battle/Escape trace, checks the real field-return reset, and proves
the original upstream world and progression ledger remain unchanged. These are
source-derived predictions. The new water encounter and return checkpoints
still require comparison with retail gameplay.

Rebuilt/current-config check (seed0, title4997, shared level retained; before
Osvald and Western second-encounter path): the entire earlier prediction is
unchanged through Ryu arrival7000. Hikari's second menu is Agnea at7087 and
needs no extra boost. Bridge entry stays7134. The water pack is Sailfish x2,
Normal; Agnea's first menu is7192. Two `0 -> 1 -> 0` cycles give a successful
40% Escape check, with field return7197 (`0x3C9AD3EF`) and counter1421. Required
checks pass for both emitted Snows timing alternatives. Complete API response
is saved in `outputs/canalbrine-water-live-4997.json`; release request took
3.13 seconds. The initial-seed advance counts include the4997 title advances
and12 opening draws; stored route RNG positions remain relative to battle entry.
