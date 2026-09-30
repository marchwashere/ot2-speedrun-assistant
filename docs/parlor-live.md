# Throné parlor capture — 2026-09-27

The route is: enter the New Delsta parlor, skip the first cutscene, run to the
next trigger, select Yes, skip two more cutscenes, and return to New Delsta
backstreets outside the parlor entrance. The capture is
`data/source/live-parlor-20260927.json`; the original remains under the parent
workspace's `outputs/throne-parlor-live-20260927.json`.

The tracer attached to game PID 17276 / game thread 4752 at seed-zero position
5374 (`0x6B5BBFD6`). It recorded 50 consecutive CRT rand calls, no srand calls,
and detached with the original debug registers restored at position 5424
(`0x080E80B0`). Every pre-call state and the final state match the CRT recurrence.
The roughly 211 seconds before the first recorded call contained no CRT draws.

## Boundaries and attribution

Positions below are completed advances from seed zero; ranges are half-open.
Attribution combines captured caller/Blueprint identity with EventList and the
event scripts; debugger timestamps are not game-time measurements.

| Positions | Calls | Source |
|---|---:|---|
| 5374–5376 | 2 | Outgoing map encounter-counter B/A reset |
| 5376–5382 | 6 | Fresh parlor streamed levels: primary, Art, Coll, GD, Sound, Btl_Twn_Cty_B |
| 5382–5386 | 4 | Parlor NPC starting frames |
| 5386–5388 | 2 | Incoming parlor counter B/A reset |
| 5388–5390 | 2 | 0900 FinishPre counter reset |
| 5390–5391 | 1 | 1000 sets Day; time-change Black Market flag lottery |
| 5391–5393 | 2 | 1000 FinishPre counter reset |
| 5393–5395 | 2 | Serial 1100 loads backstreets; outgoing counter reset |
| 5395–5400 | 5 | Fresh backstreets streamed levels |
| 5400–5420 | 20 | Daytime backstreets NPC starting frames |
| 5420–5422 | 2 | Incoming backstreets counter reset |
| 5422–5424 | 2 | 1100 FinishPre counter reset |

Totals: 11 streamed-level draws, 24 NPC draws, seven counter B/A pairs and one
Black Market draw. These are capture results, not a fixed +50 runtime rule.

The four parlor placements are TALK_0300_D000, TALK_1700_N000,
TALK_1800_N000 and TALK_1900_N000 (all prefixed NPC_Twn_Cty_1_1_C_).
The backstreets capture includes the chapter-specific TALK_0500_D000_TOU and
Epilogue_PLAYER_IS_Throne placements. The latter remains eligible via flag 2206.

The lottery is positively identified from the live RDI row's ID, begin/end
flags, trigger and ApplyValue fields as GMK_BLACKMARKET_CONTROL. It is caused
by the Night-to-Day change in 1000 command 1550, not by entering the parlor.
The trigger runs 0910, whose explicit Yes branch calls 1000. Its own FinishPre
has EncountReset=false and consumes no CRT draws. The 1000 script executes
FinishPre before serial event 1100. EventList puts
1100 on map 61, backstreets, and its script plays RI_MS_TOU_10_1200. Thus the
last two scenes are distinct serial events, each with its own counter reset.

The rich sequences 0900/1000/1200 were inspected with audit-sequence.cjs;
their decoded event-key bodies contain no Random/Timer calls. Actual captured
CRT calls are entirely accounted for by the categories above.

## Reproduce

Run `node tools/verify-parlor-capture.cjs` from OT2SpeedrunAssistant. This verifies
CRT continuity, captured caller categories, Black Market row identity, and the
scripted time-change/serial-event conditions against the local cooked assets.

`src/parlor.rs` now continues the city prediction using its exact CRT boundary
and `world_after`. Both map changes use `Loader::load_map`; events 0900, 0910,
1000 and 1100 run through the imported-command interpreter. The choice is an
explicit route input, not an assumed default in the event interpreter. The
runtime does not read this capture or use a fixed draw-count offset.

This extension exposed a loader bug: retaining a battle-map package merely
because the destination requests the same name skips its constructor draw.
Native `FUN_140ed9bc0` unloads the outgoing field's BattleMapNameList before
`FUN_140ed98a0` loads the destination. Both areas use Btl_Twn_Cty_B, which must
therefore be recreated twice. Btl_SinExpose is held separately and stays resident.
The loader now follows that unload/load sequence.

`cargo test --offline parlor` compares every live CRT state, caller category and
NPC identity, and checks flag/level/quest variants and relative draw counts.
Manipulation shows CRT after returning to backstreets; Detailed log and exports
include the intermediate entry/first-scene checkpoints and every callback.
