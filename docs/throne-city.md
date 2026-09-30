# Second fight to New Delsta radar tutorial

The runtime now executes the event commands and world callbacks for the route
below. NPC counts and total advances are outputs, not fixed profile inputs.
See the [town-loading audit](town-loading-audit.md) for coverage and validation.

**Live-validated on 2026-09-27:** second-fight drops finish at **5264**
(`0x41C7DB10`); the city sequence adds **110**, reaching the user's **5374**
(`0x6B5BBFD6`). The capture contains 263 consecutive CRT calls starting after
the first battle popup at 5111. The battle's 144 calls match the corrected
standalone engine, including all six player menus and one leader hit on Pirro.
The city calls match the table below by caller and RNG state. Adjacent counter
pairs are attributed to individual events from source order.

The previous 5380 prediction started with an incorrect second-battle exit of
5270. Fixes in the battle core remove random start voice selection for the
scripted BattleStartEvent, correct normal/charged enemy skill slots, and include
Darkest Night's break/kill reaction rolls. They also change conditional shuffle
draws. No blanket offset or city-count adjustment is used. See
[the capture](../data/source/live-second-city-20260927.json) and
[the battle analysis](../../OT2BattleEngineRust/docs/throne-second.md).

The supported transition consumes **110 main-CRT draws** after the second fight's
enemy drops. Skip both scenes
promptly, remain on the fresh Throné Chapter 1 route, and stop after dismissing
the radar tutorial without taking other actions.

| Stage | CRT draws |
| --- | ---: |
| Return from battle: encounter counter B/A | 2 |
| Sewer scene `MS_TOU_10_0700`, FinishPre B/A | 2 |
| Request map 60: outgoing counter B/A | 2 |
| Create New Delsta main and its battle scenery | 6 |
| Main-map NPC starting frames | 59 |
| Main-map shown: incoming counter B/A | 2 |
| `0800` sets Night again | 0 |
| Request map 61: outgoing counter B/A | 2 |
| Create backstreets and its battle scenery | 5 |
| Backstreets NPC starting frames | 26 |
| Backstreets shown: incoming counter B/A | 2 |
| `MS_TOU_10_0800`, FinishPre B/A | 2 |
| Radar tutorial 7, open/dismiss | 0 |
| **Total** | **110** |

The two scenes are `0700` (map 57, sewer) and `0800` (map 60, New Delsta main).
`0700` explicitly finishes before serially starting `0800`. At the end of
`0800`, command 500 loads map 61 (`Twn_Cty_1_1_B`), command 9990 performs
FinishPre, and command 7100 opens tutorial 7 (`eRADAR_MAP`). Thus the route
loads **two city maps**, including the main map used by the second scene.

`LevelFunction.RequestMapChangeForEvent` and `RequestMapChangeWithBGM` call
`BPC_MJLevelManager.RequestLoadFieldLevel`. On acceptance, that function
calls `Proc_OnFinishLoadFieldLevel` immediately, despite its name: its
implementation resets the outgoing map's encounter counter. Later,
`Proc_PrevFadeIn_Exit` dispatches `Proc_OnFieldLevelShown_Last` for field
transitions; that refreshes placements and resets the incoming map counter.
Both calls matter, even in a city with no random encounter zone.

The native loader (`FUN_140ed31b0`, `FUN_140ed8170`, `FUN_140ec60c0`) creates
the primary field map and existing Art/Coll/GD/Sound sublevels, plus its battle
map. The main map has GD; the backstreets does not. Each fresh streamed level
calls `FUN_1411c0100` once. The shared `Btl_SinExpose` remains resident and
adds no draw. The title-start timer is not automatically added to map loads.

NPC counts apply flag/time/story conditions and exclusion labels from
PlacementList/PlacementData, following `FUN_140ec4900`, `FUN_140ec5180`, and
`FUN_140ed5040`. Missing eligible NPCs reach
`BPC_LevelManager_Others.SpawnNPC.RandomIntegerInRange(0,15)` once. Existing
NPCs refresh without another starting-frame draw.

The **26th backstreets NPC** is `Epilogue_PLAYER_IS_Throne`, at (-10000,-10000).
Its actual gate is flag 2206, set in `MS_TOU_10_0100`; its name and off-map
position are not extra spawn conditions. Ignoring it would predict 109 instead
of 110. Its resource is `NPC_Epilogue_PC_Throne`, action `TouE003_Fml`.

Night is already active. `0800` command 1550 requests Night (native value 4);
`FUN_140f6aa40` -> `FUN_140f6eb70` returns when old and new time match, avoiding
another time-change/Black Market lottery. Both city rows have
`BarEventLabel=None`, so `RefreshBarCharacter` exits before its lottery.
Scenery's explicit RandomStreams and Unreal/CRI RNG are excluded. Baked
construction-script random values do not imply new CRT draws on every load.
The popup is part of `0800`, not another event; closing it adds no reset.

The route initializer supplies new city loads, ordinary unmodified NPC save state,
main scenario 700 in progress, no late-story/darkness overrides, and no
intervening player actions. The suspended `0600` event resumes into `0700`,
as `0200` resumes into `0300`; its reset flag is not an additional independent
cleanup pair. The live trace validates this route's call sequence, not arbitrary
save files or story states. The standalone battle core still ends after drops.

Regenerate the rules and evidence with `node tools/audit-throne-city.cjs`.
The script checks the cooked scripts, placement eligibility, map inventory and
Blueprint call paths, preserving hashes and decoded functions in
`data/source/throne-city.json`. Runtime inputs are `data/town-loading.json`:
all candidate placements and conditions, available streamed packages, event
commands and all 33 flag-lottery rows. `src/world.rs` executes them against
`WorldState`; `src/city.rs` runs 0700 and its serial 0800 continuation.
`data/throne-city.json` has been removed. Detailed log lists each individual
draw with its level/NPC/callback; exports also include skipped-branch decisions
and the resulting persistent world state for the next transition.

The initial flags follow the actual branches in 0100..0600. They do not include
flags from skipped sides of conditionals. The initializer does not replay the
already-accounted CRT draws before the second victory. Commands 230 and 2310
are player positioning and party healing, respectively, not NPC refreshes.
