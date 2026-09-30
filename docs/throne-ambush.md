# First sewer guard: Ambush CRT trace

Expected **+4 main-CRT draws** from confirming Ambush after the checkpoint
tutorial to regaining field control after skipping `MS_TOU_10_0500`. This is a
source-derived result, not a live measurement. It assumes the fresh Throné
opening route, the current map remains loaded, and flags 8003 on / 8004 off.

**Opening the path-action menu first consumes one additional CRT draw**, so the
expected total from before menu opening is **+5**. The +4 boundary starts after
the menu is already open. The earlier version incorrectly included opening in
the zero-draw row below; the menu's timer is a separate draw.

| Step | CRT draws | Reason |
| --- | ---: | --- |
| Open path-action preview, before confirmation | 1 | Native action timer setup; outside the +4 boundary |
| Confirm Ambush; check required level | 0 | Level comparison, no success lottery |
| Select the guard's story event and fixed dialogue | 0 | Bypasses normal random dialogue selection |
| Knock out the existing guard | 0 expected | Changes existing actor and saves swoon state |
| `MS_TOU_10_0410` FinishPre | 2 | Encounter counter B, then A |
| Skip `MS_TOU_10_0500`, restore party; FinishPre | 2 | Encounter counter B, then A again |

The second reset supplies the counter used when exploration resumes. The first
reset still consumes RNG despite encounters being disabled. These four draws
are **additional to** the earlier +4 for the checkpoint cutscene/tutorial.

## Menu opening: one draw even with zero timer variance

`FCv2_FieldCommandManager.UpdatePreSequence` state 2 calls
`PlayFieldCommandActions` @472. The base flow's `PlayFieldCommandAction` starts
the normal day/night pose and returns true when the time-zone manager is valid,
including Throné's Ambush pose. After iterating the active flows, the manager
returns `IsPlayed=true` and the maximum action wait time.

`UpdatePreSequence` then calls @541:

```text
K2_SetTimerDelegate(OnFinishFieldCommandAction, MaxWaitTime, false, 0, 0)
```

Native thunk `FUN_1430f96c0` calls `FUN_142c52d90`. After validating the delegate
and world, the latter executes:

```text
r = rand()
initial_delay += (r & 0x7fff) * variance * 6.103702e-05f - variance
```

The `rand()` occurs **before** multiplication by the zero variance. This is one
main-CRT call (return RVA `0x2C52E57`), not Unreal's independent global RNG.
The timer is created once after the loop, not once per party member, and it does
not repeat. Menu confirmation is not required: cancelling retains that advance;
reopening through the normal path creates another timer and consumes another
draw. This concerns actively opening the preview menu, not the passive button
prompt displayed beside an NPC.

This timer is separate from the deterministic `SetKSGCTimer` postprocessing
discussed below. The first guard's story override bypasses the later dialogue
lottery, but it does not bypass this menu-opening timer.

## Ambush dispatch and the skipped lottery

The preview calls `BP_FCv2_FieldCommandFlow_Assassinate.IsGreaterEqualRequiredLevel`:
the current field-command level must meet `NPCBattleData.AssassinateNeedLevel`.
This guard requires level 1, has `CommandEnable=4`, and `CanSwoon=true`.
There is no `GetRandomForFC` success roll in this flow.

`UpdateFlow` state 1 checks authored placement events before running the ordinary
Ambush sequence. The guard placement `NPC_Dng_Cty_3_1_0100_N000` has:

- Event type `eFC_ASSASSINATE_SUCCESS` (the flow queries type 20).
- Event parameter `None`, start flag 8003, end flag 8004.
- Event label `MS_TOU_10_0410`.

The lookup requires matching type, flags, time and parameter. Thus the checks for
`FC_FADE_SE_ON` do not select this row; the type-20 / `None` branch does. It calls
`SetNextEventLabel`, binds `BeginEventCallback`, registers `SwoonNpcInFade` on
notice 38, and moves directly to state 8. It never executes state 2.

**Ordinary Ambush is different:** state 2 makes a weighted map of
`BeforeEventLabel1/2` and calls the base flow's `DrawEventLabel`. That helper
always calls `RandomFloatInRange(0, TotalWeight)` once, then finds the winning
label. Native `FUN_1430d6fa0` implements that call with one main CRT `rand()`:

```text
u = (rand() & 0x7fff) * 3.051851e-05f
draw = min + u * (max - min)
```

The generic success-after-dialogue label is used directly, without another
`DrawEventLabel`. However, generic Ambush's complete total still depends on its
selected event rows, story overrides and subsequent placement changes. Do not
apply this guard's +4 offset to every NPC.

The historical `tools/speedrun_audit` dump omitted this Ambush `UpdateFlow`
because its old decoder did not handle `SetMap`. The new audit decodes the whole
function with the newer decoder and verifies its `EndOfScript`, preserving the
body in `data/source/throne-ambush.json`.

## Fixed voice and knockout

`MS_TOU_10_0410` command 42 explicitly requests
`eFC_ASSASINATE_PC_TALK_01_A`; it does not choose between A and B. `SetTalkData`
looks up that label's voice. The native `GetTalkVoice` chain is
`FUN_14105c080 -> FUN_140f63560 -> FUN_140f63600`, which chooses the language's
table and looks up the supplied row. It is not the battle boost-voice lottery.

At fade-out, `SwoonNpcInFade -> SwoonNpc_Internal -> LibFieldCommand.Swoon NPC`
uses the guard's authored faint action, position and direction, calls
`ChangeSwoonState`, and adds it to `NPCManager.AddSwoonNPC`. The latter writes
the save state; this guard's `SwooningFlags` are all zero. No replacement field
NPC is spawned, so do not charge another random starting-frame draw. State 8
also has cleanup that removes the delegate and reapplies the swoon state; this
is not a second random result.

Field-command postprocessing contains `SetKSGCTimer(2)`. Its native path
`FUN_141053880 -> FUN_140f53710` uses a deterministic timer-type lookup and
requests GC timing through `FUN_142faac30`. It does not itself invoke CRT rand;
do not add the unrelated title-transition timer draw here.

## The two resets and party restoration

Event 0410 plays the fixed line, fades, plays its fixed sound label, explicitly
runs command 9990 (`FinishPre`), then command 1000 starts 0500. Its EventList row
has `EncountReset=true`, so the first reset is before the following cutscene.

Event 0500 runs its rich sequence, then after the skip boundary sets 8004,
clears 25304, restores guests 15/16 and refreshes placement. Its two temporary
field NPCs have end flag 8004 and are removed; no map NPC has start flag 8004.
The guests join the party rather than going through the field-NPC random-frame
spawn path. Its 42 authored sequence keys contain spawn/action, balloon, emotion,
fade and fixed sound calls, with no direct RNG command.

0500 then runs its own command 9990. `EncountReset=true` causes another B/A
reset; the regular `ENT_DNG_CTY_3_1_LND_1` zone is restored. Native FinishPre's
once-only guard prevents command 10000 from duplicating each event's reset.

Both resets read the current map parameters: B and A each use
`RandomInteger(400)`, then `trunc((1000 + B + A) * 0.5)` on this first-encounter
route. They advance, rather than reseed, CRT. Counter range remains 500–899.

## Reproduction and validation boundary

Run `node tools/audit-throne-ambush.cjs` from the assistant repository. It asserts
the placement, level gate, fixed voice row, event order, reset flags and relevant
party/placement changes, and writes hashed source evidence plus decoded flow.
The audit does not alter the predictor or attach to the running game.

The assistant's `route::throne_ambush` now implements this fixed route from
`ThroneCheckpoint.after_popup`, retaining the menu, post-0410 and post-0500 RNG
states and both counters. The opening GUI and JSON include it only when the
first checkpoint is reachable without an encounter. The run screen displays
the final CRT and counter; Detailed log displays all three checkpoints and
the +1/+0/+2/+2 trace. It assumes exactly one menu opening, then Ambush and skip.
The earlier post-battle field frame and battle-core result remain unchanged.

For live validation, record CRT before menu opening, at the open menu (+1
expected), during the following cutscene after 0410's FinishPre (+3 from before
opening), and after skipping it and regaining control (+5 from before opening,
or +4 from confirmation). This static audit is not exhaustive proof that all
transitive engine callbacks are RNG-free; keep the total marked expected until
those boundaries are captured.
