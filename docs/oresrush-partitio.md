# Oresrush and Partitio route extension

The subsequent Ryu endpoint now includes the authorized Hikari recruitment;
see [Hikari recruitment](hikari-recruitment.md). Earlier notes about stopping
before Hikari describe the preceding extension, not the current route.

The active route now continues from Southern Cropdale Trail through:

1. Southern Oresrush Wilds (`Fld_Wld_1_3`, MapID86).
2. Oresrush (`Twn_Wld_1_1_C`, MapID100).
3. Throné Chapter2 Mother's route acceptance and skipped opening scene.
4. Partitio recruitment, skipping his Chapter1 and dismissing the tutorials.
5. Return to Southern Oresrush Wilds.

`src/oresrush.rs` carries the selected branch's actual RNG, world flags,
levels/resources, enemy knowledge, Evasive Maneuvers and level residency. It
uses the same `world::Loader` as the preceding route; no fixed map-load or
cutscene RNG offsets are added. Each stage is exported and appears in Detailed
log. Manipulation adds a movement instruction for the Wilds-to-town leg.

## Movement

Southern Cropdale uses the existing padded122 walking /112 running update
budget. The predicted mixed distance leaves the counter positive before the
Southern Oresrush transition.

The user measured75 running updates from Southern Oresrush's entry on the
Cropdale side to Oresrush. Its daytime encounter decrement is9; Evasive
Maneuvers and dash give `trunc(9 * 0.5 * 1.4) = 6`. The leg consumes450,
whereas its newly rolled first counter is500..899. Running the entire leg is
therefore safe, leaving at least50 points, for the supported daytime route.
No walking update count or speed-switch density is assumed for this leg.

After returning from Oresrush, the next leg is Southern Oresrush Wilds ->
Northern Ryu Sands (`Fld_Dst_1_1`, MapID19). Measured127 walking /119 running
updates plus3 leeway give **130 walking /122 running**. Daytime Evasive costs
**520 /732** respectively. Use the fresh return counter; never reuse the
initial Southern entry counter. A counter of758 permits100% running with26
remaining before the transition. This is not Northern Ryu's loaded counter.

The preceding Eastern Cropdale return encounter is the last available boost
menu. Its cost-ordered, uncapped Escape search now replays the complete route
through Mother acceptance, Partitio recruitment and the Southern return for
each successful Escape. It requires safe walking on both the Southern Cropdale
approach and this Ryu leg. Rejected successful candidates do not affect the
selected RNG/history/world. No earlier search settings are repurposed.

The displayed running distance share interpolates the separate measured
densities: `cost(f)=520*(1-f)+732*f`. Reserve1 counter point, round the share
down to0.1%, and round consumed cost up. This remains the existing movement
estimate, not frame-by-frame speed-switch overshoot simulation. The prediction
continues through Northern Ryu and Ryu as described below; movement itself
consumes no CRT calls.

## Northern Ryu Sands to Ryu

`src/ryu.rs` visits Northern Ryu through the shared Loader, then derives the
budget from that newly rolled counter. Measured143 walking /137 running
updates plus3 leeway give **146 walking /140 running**. Daytime Evasive costs
**584 /840**, so walking requires counter585 or higher, all running841 or higher.
Running share uses `cost(f)=584*(1-f)+840*f`, with the same strict reserve and
downward rounding. The preceding Eastern Cropdale Escape search now also
requires this walking target; it never advances CRT synthetically to roll it.

On success, retain the estimated remainder until the real Ryu transition,
then visit `Twn_Dst_1_1_A` with outgoing/incoming counter callbacks, actual
stream residency and NPC gates. `tools/audit-ryu-loading.cjs` checks both maps'
395 preplaced Blueprint actors,49 classes including inherited/component
classes, and embedded level scripts. There are no additional environmental
runtime random callbacks. Both tavern events are None. Current progression
has two ordinary Northern NPCs and eleven in Ryu, including the six eligible
Hikari scene actors; counts are evaluated from placements, not hardcoded.

The four-member party, all EXP/JP, original equipment, enemy history, active
Mother scenario and Evasive support stay intact. **Stop before Hikari's
recruitment**, which the user deferred. Northern/Ryu checkpoints and numeric
budgets appear in Detailed log; the movement card shows only run/walk guidance.

## Mother Chapter2 acceptance

The selected arrival menu (`MapInStorySelect`, command7110) confirms story
`MS_TOU_01`, scenario701, released by8019. Native `SetConfirmedMainStory`
stores confirmation; `PlayEventForce` invokes the opening. `PlayEvent` calls
`UpdateMainStoryProgressOnPlayEvent` before map loading (`f55a00/f55560`),
setting the active story/scenario (`f55370/f37b20`) and task view flag30510.

Opening `MS_TOU_2A_0100` sets daytime, plays its skipped fixed sequence, sets
8050 and executes FinishPre. The refresh now enables the ordinary story NPC
`NPC_TOU_2A_0400_0000` through story7/scenario701 and8050; its frame lottery is
evaluated by the shared placement loader. FinishPre also executes the actual
counter reset. Accepting this chapter does not award a progression point.

Although the playable-character table has `MainMissionPlayingFlag2005` for
Throné, the audited opening path did not prove an assignment to it. The model
uses the saved active-story state and does not invent that flag mutation.

## Partitio recruitment

First invitation `MS_SHO_00_0100` selects YES at58 and NO (skip Chapter1) at72.
The repeat invitation0200 belongs to a prior-decline state37505 and is not
used. Command7450 chooses a fixed greeting from the party leader's FirstJob;
Throné selects option6, then command39. There is no random speaker selection.

The event adds character4, retains him in the active party through3005,
executes its normal flags, FinishPre refresh/reset, and tutorials66/69. Native
skip completion (`f15190 -> f54e00 -> f4ec50 -> f537f0`) applies scenario400's
on/off event-flag ranges as well as its one progression point. The supported
loader now applies these imported ranges for Osvald and Agnea too. Their extra
flags affect no eligible placement or lottery on the previous route, preserving
its RNG sequence. The3005 guard is global reminiscence flag2900.

Final party is `[Throné, Osvald, Agnea, Partitio]`, progression4. Partitio starts
level1, zero EXP/JP, original lance/bow and other equipment. His imported
growth record stays in the battle core. Mother's scenario701 remains active
through the return load. Fixed skip-story item grants consume no CRT; the
existing world model does not yet maintain shop/recruitment item inventory.

## Verification

`tools/audit-oresrush-loading.cjs` and `data/source/oresrush-loading-audit.json`
record map/event/story/placement sources and the Blueprint actor audit. The
391 preplaced Blueprint instances across the new maps use56 classes/parents;
no additional runtime BeginPlay random callbacks were found. Construction
script random helpers are already baked into the cooked levels.

Regression coverage checks stage-by-stage CRT continuity, immutable input,
Mother active/confirmed state, skip flags, character growth/equipment,
progression4 and the fresh return counter. The new endpoint predictions still
need retail validation.

Ryu regression tests check strict walking/running boundaries, carried support,
input immutability and a successful unadjusted Escape whose later return counter
fails the walking target. The search must select a later successful plan and
reproduce its complete Escape trace from the untouched boost prefix.

## Retail Mother acceptance capture, 2026-10-09

The continuous acceptance segment in outputs/oresrush-mother-live-20261009.json
starts at seed0 advance19286550 and ends at19286557. It contains two counter
resets (B/A each), NPC_TOU_2A_0400_0000's frame draw, then the known FinishPre
counter reset. The prior model omitted the initial two resets. Replay the real
counter routine for both so counter state and RNG propagate together.

The native cross-check resolves the dispatchers:

- BarMenuStorySelect.OnClosedConfirmDialog @0327 calls PlayEventForce.
- Native wrapper141005090 calls140e920f0, setting EventManager+0x59d=1.
- EventLoadLevel native wrapper141002830 uses vtable+0xba0. EventManager's
  constructor installs vtable143ca8a48, whose slot resolves to140e8dd70.
  That function rejects an identical destination only when +0x59d is zero.
  A forced event therefore requests the current map too.
- EventManagerBP.EventLoadLevel -> LevelFunction.RequestMapChangeForEvent ->
  BPC_MJLevelManager.RequestLoadFieldLevel -> Proc_OnFinishLoadFieldLevel_Impl
  calls ResetEncountVolume (first reset).
- Proc_OnFieldLevelShown_Last_Impl -> ResetEncountOnChangeLevel calls
  ResetEncountVolume again (second reset). Both restore FirstEncount and call
  ResetEncountStep. The existing streams/NPCs need not be recreated.
- FinishPre performs the separate final reset after the NPC refresh.

The loader now carries the force-request distinction from chapter selection
into the shared event/map path. The temporary MS_TOU_01-specific reset patch
was removed. Ordinary same-map events still skip the transition. This is not
an unconditional reset for opening/closing a story prompt or any cutscene.
Run tools/audit-forced-event-load.cjs from the workspace root to check the
native condition and Blueprint call chain. The retail regression checks all
seven calls and confirms that declining the chapter consumes none of them.
