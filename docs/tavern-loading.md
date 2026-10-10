# Tavern CRT callbacks

`tools/import-taverns.cjs` extracts all 14 configured maps, eligibility masks,
slot capacity, names and game parameters into `data/taverns.json`.
The shared loader consumes these rules on map loading and explicit tavern refresh callbacks.
This does not add full route/map support for towns outside the existing map catalog.

RefreshBarCharacter rolls 0..99: 0..50 requests zero slots, 51..74 one,
75..99 two. Winterbloom caps this at one because BarPCPos2.x <= -10000.
Each slot selects from eligible benched characters (including a draw for a
singleton pool), removes the selected member, and calls SetupCharacter ->
CalcItemData, rolling 1..100. The latter is unconditional even for the ID-zero
placeholder used when no eligible members remain. Gift eligibility is <=5;
the trace stores the raw 0..99 draw and reports eligibility separately.
The final-chapter override and BarEventLabel=None bypass all these draws.

Evidence: ../outputs/throne-city-audit/RefreshBarCharacter.txt;
../tools/speedrun_audit/Character__BP__Type__KSBarCharacter.txt;
../docs/conning_creek_roster_rng.md traces native dispatch through
KSCharacterManagerBP.AddBarCharacter with no ID-zero spawn guard.

The previous loader incorrectly skipped placeholder item draws. Consequently
the prior empty-bench Winterbloom +14 per switch is a baseline, not a constant:
the corrected branch adds zero or one item draw. Existing saved route results
must be recalculated. This correction is source-derived, awaiting live replay.
Tavern conversations and receiving gifts are not modeled route actions here.

## Manual-switch correction (2026-10-09)

The earlier statement that manual time switching refreshes the tavern was wrong.
`FUN_140f5cb60` calls the placement refresh `FUN_140ed58b0`; that native path
does not dispatch `RefreshBarCharacter`. The Winterbloom retail capture
records the town-load tavern lottery at19286246, but none during the subsequent
manual switch. That switch records the Black Market lottery at19286249 followed
by12 NPC frame draws, ending19286262. The shared manual switch helper now
preserves the tavern state. Map-load and explicit recruitment callbacks remain.
The regression checks all46 calls from field departure through Night against
`data/source/live-winterbloom-night-20261009.json`.

Validation: retail regression, counter-preservation, guild/story, and Western rare
branch tests pass. The broader random-route test now reaches the existing
unsupported first-Escape enemy ENE_NML_WTR_570; that full-route coverage remains
incomplete and is not evidence against the captured town/switch sequence.

## Trigger audit (2026-10-09)

The integration error conflated RefreshSpawnCurrentLevel with its map-load
wrapper. BPC_LevelManager_Others.RefreshPlacementOnChangeLevel contains four
separate calls: RefreshSpawnCurrentLevel, VisibleAll, RefreshBarCharacter,
RefreshFlagObject. RefreshBarCharacter is not inside RefreshSpawnCurrentLevel.
Evidence: tools/speedrun_audit/Level__BP__BPC_LevelManager_Others.txt @0023
within RefreshPlacementOnChangeLevel; outputs/throne-city-audit/RefreshPlacementOnChangeLevel.txt.
Native manual f5cb60 calls ed58b0 -> ed5040, which iterates placements.
Event FinishPre e8e740 also calls ed58b0 when its placement-dirty flag is set.
Neither call justifies the map wrapper's separate tavern lottery.
Removed duplicated tavern calls from legacy manual-switch routes, transitional
set_time placement refresh and explicit/implicit event FinishPre cleanup.
Retain map loading and the separately modeled party replacement callback;
this audit does not establish arbitrary party-management UI behavior.

Lottery internals rechecked against Blueprint: roll0..99; <=50 none,
51..74 one,75..99 two; cap second slot if BarPCPos2.x<=-10000.
Filter bench by map eligibility mask; select without replacement (singleton
also draws), then SetupCharacter/CalcItemData gift roll even for an empty-bench
ID-zero placeholder. These additional branches are source-derived; the retail
Winterbloom capture validates only the zero-slot branch.
