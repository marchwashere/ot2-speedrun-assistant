# Throné: first sewer checkpoint and Ambush tutorial

Source-derived expectation: **four main-CRT draws**, from arriving at the first
checkpoint to dismissing its tutorial, assuming the fresh opening route, no
intervening encounter, the same loaded map, and the cutscene skipped. This has
not yet been validated by a live capture of this boundary. Opening results now
show arrival and the expected post-tutorial CRT separately, only if walking or
the displayed mixed movement can reach the trigger without an encounter. The
field-encounter screen still ends at arrival, before these event callbacks.

| Cause | CRT draws |
| --- | ---: |
| Spawn Pirro's field NPC; random starting animation frame | 1 |
| Spawn Scaracci's field NPC; random starting animation frame | 1 |
| Reset encounter counter: B, then A | 2 |
| Open/dismiss the Ambush tutorial itself | 0 expected |

No animation or tutorial rendering belongs in the headless model. Preserve only
these CRT draws and gameplay state changes. This is not another title transition:
do not add streamed-level color draws or the optional shared-level draw here.

## Script and placement evidence

`EV_TRIGGER_MS_TOU_10_0400` is active between story flags 8002 and 8003 and runs
`MS_TOU_10_0400`. Its EventList row has map 57 (`Dng_Cty_3_1`), skip enabled,
`EncountReset=true`, and `EncountTrigger=ENT_EMPTY_1`.

After the rich sequence, the script sets flag 8003, sets flag 25304, removes
guests 15/16, updates placement, executes command 9990 (`FinishPre`), then command
7100 with target 32 (tutorial), and finally command 10000 (finish event).
The popup is part of **0400**, not a separate event.

The map's PlacementList contains these newly eligible entries:

- `NPC_Dng_Cty_3_1_0200_N000`: start 8003/end 8004; resource
  `NPC_TOU_10_0100`, action `TouPirlo_Mle` (Pirro).
- `NPC_Dng_Cty_3_1_0300_N000`: same flags; resource
  `NPC_TOU_10_0200`, action `TouSchillaci_Mle` (Scaracci).

The guard `NPC_Dng_Cty_3_1_0100_N000` already exists from map initialization;
his event changes eligibility at 8003, but this does not create a new NPC.
`FUN_140ed5040` distinguishes refreshing an existing object from spawning a
missing eligible one. `BPC_LevelManager_Others.SpawnNPC` @1677 makes one
`RandomIntegerInRange(0,15)` call per new NPC, even for the explicitly selected
single-play motion. Do not count rich-cutscene character spawns as field NPCs.

The decoded rich sequence has no direct RNG commands. Its keys are character
spawn/action, balloon, and fade commands; an authored key count is not a draw count.

## Reset timing and popup

Command 9990 dispatches `FUN_140ff7430` (FinishPre). Native implementation
`FUN_140e8e740` checks and sets the once-only `ExecFinishPre` flag at +0x519.
Pending placement refresh runs through `FUN_140ed58b0` / `FUN_140ed5040` before
the Blueprint continuation. `EventManagerBP.FinishPre` @3342 checks EncountReset,
then @3470 calls `ResetEncountStep`. The expected order is therefore two NPC
draws followed by counter B and counter A, before tutorial opening.

`EncountManagerBP.ResetEncountStep` reads **map** parameters even when the active
encounter trigger is empty. `CalcEncountStepCount` draws B then A, each using
`RandomInteger(400)`, and sets `trunc((1000 + B + A) * 0.5)` while FirstEncount is
still true. The range remains 500–899. Use the existing binary32 scaling helper,
not `rand() % 400`. The counter reset never reseeds CRT.

Tutorial target 32 resolves through **TutorialFlagPart**, not TutorialListTable:
`eFIELD_COMMAND_NIGHT_TOU`, page `TUR_FC_N_TOU`, the Ambush tutorial. The inspected
open/close path contains no separate encounter reset: command 7100 calls
`FUN_1410f7e30`; its implementation goes through `FUN_140f67460` to
`FUN_140f67070`. Closing invokes `OnTutorialFinish` -> `FUN_140f66900` ->
`FUN_140f6c860`, returning control to the event. `EventManagerBP.FinishEvent`
@80 does not rerun FinishPre when ExecFinishPre is already set. Counting a
second two-draw reset for dismissing this popup would double-count it.

The four-draw total is a source-derived expectation, not a captured measurement
or an exhaustive proof that every transitive engine callback is RNG-free.
A live validation should record CRT immediately before the trigger, at tutorial
opening, and after dismissal, with NPC-spawn and counter-reset call stacks.

## Following route state

Flag 25304 disables the regular encounter volume and 0400 selects `ENT_EMPTY_1`;
Throné is alone during the Ambush tutorial section. The pre-checkpoint encounter
configuration must not continue to be used here.

The guard's successful Ambush triggers **0410**, which has its own
`EncountReset=true`, then branches to **0500**. Event 0500 clears 25304, restores
guests 15/16 and the regular encounter trigger, and also has EncountReset enabled.
Those later resets are separate from the four draws above. The subsequent
[Ambush source audit](throne-ambush.md) expects another four draws from confirming
this guard's Ambush to regaining control after skipping 0500. Its special story
branch bypasses the ordinary Ambush dialogue lottery; live validation is pending.

## Reproduction

Run `node tools/audit-throne-checkpoint.cjs` from this repository. The output
`data/source/throne-checkpoint.json` stores the relevant cooked rows/scripts,
source hashes, newly active NPCs, sequence call inventory, native/Blueprint
references, and the chapter event manifest. The manifest's reset flags alone
do not establish complete RNG counts for the remaining chapter.
