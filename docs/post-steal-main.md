# Backstreets to New Delsta main after both thefts

**Endpoint confirmed by the user on 2026-09-28:** **5472 -> 5530**, CRT
**0xB15BB212**, for loading
`Twn_Cty_1_1_A` from `Twn_Cty_1_1_B` without intervening actions/time changes.
Ends when the zone finishes loading, before the next story cutscene.

The executable example `post_steal_main_trace` carries `world_after` from the
second parlor visit, applies 1400's actual flag writes (8013 on, 34049 off) and
Save tutorial flags 1015/1315, then uses the audited theft boundaries: clothes
+2, boy +3. The party remains Throné alone, and the scripted Day/Dusk phase
selects daytime NPCs. Both NPCs retain their actors after their items are stolen.

`world::Loader::load_map` evaluates the destination candidates and gates:

| Callback | CRT draws |
| --- | ---: |
| Outgoing encounter counter B/A | 2 |
| Six destination streamed levels | 6 |
| Eligible NPC initial frames | 48 |
| Incoming encounter counter B/A | 2 |
| Flag lotteries / tavern selection | 0 |
| Total | 58 |

The shared Btl_SinExpose remains resident. The map's BarEventLabel is None;
there is no active level-change flag lottery, and unchanged time does not
invoke Black Market selection. Counter after arrival is 612, though the town
has no encounter zone. Flag 8013 enables the next story trigger, not an NPC.

Run `cargo run --offline --example post_steal_main_trace` for every draw and
placement decision. No fixed map offset is used. The final endpoint is confirmed;
individual callback attribution remains source-derived, without a new live trace.
