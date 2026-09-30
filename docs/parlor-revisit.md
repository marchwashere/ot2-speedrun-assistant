# Second parlor visit: source-derived prediction

Start from the first visit's completed 1100 state: backstreets, Day, flag 8010
on and 8011 off. Enter the parlor, promptly skip 1200, then complete the serial
1300 event's fixed dialogue and dismiss the Steal tutorial after teleporting
back to the backstreets. This stage is not yet live-validated.

`src/parlor_revisit.rs` carries the prior CRT and `world_after` into `world::Loader`.
There is no fixed draw offset and no reconstructed fresh world state.

For the existing seed-0 replay:

| Boundary | Advances since seed | Draws added |
| --- | ---: | ---: |
| Previous backstreets return | 5424 | — |
| Entered parlor | 5435 | 11 |
| Backstreets arrival after 1200 | 5467 | 32 |
| Steal popup dismissed, player control restored | 5467 | 0 |

Final CRT: **0xB8E6F54D**, +43 total. The evaluated stages are:

- Parlor load: outgoing counter B/A, six streamed levels, one daytime NPC,
  incoming counter B/A: 11.
- 1200 sets Dusk, flag 8011 and runs FinishPre (`EncountReset=true`): two counter
  draws. Dusk itself produces no draws for these existing placements.
- Serial 1300 loads the backstreets: outgoing counter B/A, five streamed levels,
  21 eligible NPCs, incoming counter B/A: 30.

Flag 8011 enables `NPC_TOU_10_0500_D000`, accounting for the additional NPC versus
the previous daytime backstreets return. It is selected by its placement gate,
not a hardcoded count. `Btl_Twn_Cty_B` is unloaded/reloaded on both changes;
the separately retained `Btl_SinExpose` is not recreated.

## Time-change evidence

- EventList 1200 command 1550 option 2 requests native phase 3 (Dusk).
- `FUN_140f6aa40` calls `FUN_140f6eb70` before writing the new phase.
- For transitional phase 3, `FUN_140f6eb70` first calls `FUN_140ed58b0`, which
  refreshes tavern/placement eligibility using the old phase. Existing NPCs
  retain their random frames; only newly spawned actors draw.
- Day 2 -> Dusk 3 decrements placement timers by one. It does not dispatch the
  half-day time-change lottery event 0x27, so there is no Black Market draw.
- `FUN_140f65d50` -> `FUN_140f68140` -> `FUN_140f68400` also handles map-load time
  initialization. A delayed automatic time transition depends on runtime timer
  state; this prediction assumes the prompt skipped route, not arbitrary waits.
- The `RI_MS_TOU_10_1300` cooked sequence has no direct random/timer calls among
  its 86 imported event keys. Its ordinary character/audio/visual commands are
  not instantiated in this headless simulation.

## Return dialogue and popup

The initial implementation ended on map arrival. The predictor now retains
`after_return` for that boundary and exposes `after_popup` as its final result.

- 1300 command 300 enables the existing player/party actors: dispatcher
  `FUN_140eb67a0` -> `FUN_140ff95a0` -> `FUN_140e9d810`; no actor is spawned.
- Command 40 uses fixed label `TX_MS_TOU_10_1300_0320`. Blueprint `StartTalk` ->
  `SetTalkData` uses table lookups for text and voice. Native `FUN_140f632a0` and
  `FUN_140f63600` select the configured language's table and fixed row, not a
  random voice. Only this audited command shape is accepted by the evaluator.
- Flag 8012 is enabled and 514 cleared before command 7100 opens tutorial 31,
  `eFIELD_COMMAND_DAYTIME_TOU` (Steal). Its table supplies list flag 1029 and
  opened flag 1329; both are retained in the resulting world state.
- `FUN_140e9f1d0` -> `FUN_1410f7e30` -> `FUN_140f67070` opens the tutorial.
  `FUN_140f37740` / `FUN_140f39b50` update its flags. Close follows
  `FUN_140f66900` -> `FUN_140f6c860`. These callbacks add no CRT draw or counter reset.
- `FinishEvent` invokes implicit `FinishPre`. The placement refresh retains
  existing NPCs; neither 8012 nor 514 changes any imported placement gate.
  `EncountReset=false` suppresses the B/A counter pair.

Consequently the return dialogue, popup and cleanup add **zero** draws for this
route. This is source analysis, not a new live measurement.

Run `cargo run --offline --example parlor_revisit_trace` to reproduce the full
draw/decision trace. Regression tests preserve the prior live city/parlor
captures and exercise carried flags, Dusk actor reuse, lottery retention,
timer aging, event boundaries and route-relative GUI counts.
