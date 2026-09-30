# Mansion arrival after the Artist

Endpoint confirmed by the user on 2026-09-28: seed 0, after all three
Artist soulstones are stolen in one menu session and the menu is closed:
**5535 / 0x94B3AD61 -> 5562 / 0xDDCC3032 (+27)**.
Both scenes are skipped, with no intervening actions or automatic time change.
The endpoint is player control inside Diamante's Estate, before the next Ambush.

| Stage | CRT calls | Advances after stage |
|---|---:|---:|
| 1500 FinishPre: flag 8014 enables MS_TOU_10_16001800 NPC, then counter B/A | 3 | 5538 |
| Outgoing New Delsta counter B/A | 2 | 5540 |
| Mansion map streams: base, Art, Coll, GD, Sound, two battle maps | 7 | 5547 |
| Initial mansion NPCs: two saves, two NACT actors, prologue shop | 5 | 5552 |
| Incoming mansion counter B/A | 2 | 5554 |
| 1600 sets Night: Dusk normalizes to Day; Black Market lottery | 1 | 5555 |
| 1600 FinishPre: flag 8015 enables five mansion NPCs | 5 | 5560 |
| 1600 FinishPre counter B/A | 2 | 5562 |

These are evaluated results, not runtime offsets. The shared Btl_SinExpose
stays resident, and this map's BarEventLabel is None. There is no tutorial
command in these two scripts.

## Source and reproduction

`tools/import-town-loading.cjs` imports LevelTable ID 50, its complete placement
list and gates, both event scripts, and actual streamed packages. 1500 uses
RI_MS_TOU_10_1400 (47 event keys), then serially starts 1600 on Dng_Cty_1_1.
1600 uses RI_MS_TOU_10_1410 (44 keys). The sequence audit finds fixed dialogue,
actions, sound/fades and indoor visibility, without direct random/timer calls.
Reproduce with the sibling battle repository's `tools/audit-sequence.cjs` on
`Event/RichEvent/01_Main/70_TOU/ep1/RI_MS_TOU_10_1400` and `_1410`.

Native flag setters e93950/e942b0 and time command e97200 mark EventManager
+0x810. FinishPre e8e740 dispatches ed58b0 before its Blueprint counter reset.
The interpreter now evaluates this pending placement refresh at explicit
9990 commands. Omitting it loses the city dummy and five mansion actors.
Existing city/parlor live-call fixtures still pass with the refresh enabled.

```powershell
node tools/import-town-loading.cjs
cargo run --offline --example post_steal_main_trace > ../outputs/post-steal-main-prediction.json
cargo run --offline --example mansion_trace -- ../outputs/post-steal-main-prediction.json > ../outputs/mansion-prediction.json
```

The latter file includes every draw, gate decision and carried world state.
The prior 5530 main-city boundary and final 5562 mansion boundary are
user-confirmed. Attribution of individual calls remains source-derived.

`src/mansion.rs` now runs the entire continuation from the second parlor popup,
including all five stolen items, the main-city load and both mansion scenes.
Manipulation omits the mansion entry and parlor return cards. Detailed log
and JSON retain mansion entry, lookout Ambush, both parlor stages, the steals
and every CRT call; route propagation through them remains active.
