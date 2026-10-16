# Map-loading RNG audit — 2026-10-16

Audited 62 modeled maps, 323 unique streamed levels and 484 Blueprint classes including ancestors. Actor inventories come from cooked Level actor arrays, independently of the importer allowlist. All decoded successfully. Reproduce with `node tools/audit-map-rng.cjs`; detailed disassembly is written to `target/map-rng-audit.json`.

## Findings

- Southern Sai: six Flag B BeginPlay calls, Western Sai: three, Eastern Ku: one. Fixed in the preceding correction.
- Shipwreck: one BP_OnWaveObject_A actor, BeginPlay entry342 selects RandomFloatInRange(0,20) and SetNewTime. Added one gondola-bobbing draw. Timeline updates do not repeat that draw.
- Dng_Cty_3_1 hanging jail lamp: BeginPlay194 jumps to Play157, bypassing RandomFloatInRange at entry10. The Finished callback reaches entry10. Do not add a load draw; a later timeline completion is a separate timing-dependent concern.
- Light-beam and procedural placement randomness uses private RandomStream functions; it is not the tracked global CRT sequence. Flipbook randomization is construction-script-only on serialized placed actors, not BeginPlay.

## Lifecycle and limits

The loader initializes field actors on load, reinitializes field actors on battle return, and refreshes NPCs without environment BeginPlay on manual time switches. Hidden battle streams are inventoried but are not initialized on field entry. Counts below are environment calls only; NPC placement, streamed-level constructors and encounter-counter calls remain separate. This is a static Blueprint/actor-coverage audit, not a retail trace of every map or a complete audit of every native component, dynamic spawn, event, or timed callback.

## Every modeled map

| Map | Streams | Field initialization draws | Battle-stream environment draws |
|---|---:|---:|---:|
| Fld_Mnt_1_2 | 6 | 0 | 0 |
| Twn_Mnt_1_1_A | 6 | 0 | 0 |
| Fld_Dst_1_1 | 6 | 0 | 0 |
| Fld_Dst_2_2 | 6 | 3 | 0 |
| Fld_Dst_2_3 | 5 | 6 | 0 |
| Fld_Dst_2_4 | 5 | 0 | 0 |
| Dng_Dst_2_2 | 5 | 0 | 0 |
| Twn_Dst_1_1_A | 5 | 0 | 0 |
| Twn_Dst_2_1_A | 6 | 0 | 0 |
| Twn_Dst_2_1_B | 6 | 0 | 0 |
| Fld_Cty_1_1 | 5 | 2 | 0 |
| Fld_Cty_1_2 | 6 | 0 | 0 |
| Fld_Cty_1_3 | 7 | 2 | 0 |
| Fld_Cty_1_4 | 6 | 0 | 0 |
| Fld_Cty_2_1 | 6 | 0 | 0 |
| Dng_Cty_1_1 | 7 | 0 | 0 |
| Dng_Cty_2_1 | 5 | 0 | 0 |
| Dng_Cty_3_1 | 5 | 0 | 0 |
| Twn_Cty_1_1_A | 6 | 0 | 0 |
| Twn_Cty_1_1_B | 5 | 0 | 0 |
| Twn_Cty_1_1_C | 6 | 0 | 0 |
| Twn_Cty_1_1_D | 6 | 0 | 0 |
| Twn_Cty_1_2_A | 5 | 0 | 0 |
| Twn_Cty_1_2_B | 5 | 0 | 0 |
| Twn_Cty_2_1_A | 6 | 0 | 0 |
| Twn_Cty_2_1_B | 6 | 0 | 0 |
| Fld_Isd_1_3 | 7 | 0 | 0 |
| Fld_Wld_1_2 | 5 | 0 | 0 |
| Fld_Wld_1_3 | 6 | 0 | 0 |
| Fld_Wld_2_1 | 5 | 0 | 0 |
| Fld_Wld_2_2 | 6 | 0 | 0 |
| Fld_Wld_3_2 | 6 | 0 | 0 |
| Twn_Wld_1_1_C | 6 | 0 | 0 |
| Twn_Wld_2_1_A | 6 | 2 | 0 |
| Twn_Wld_3_1_A | 6 | 8 | 0 |
| Fld_Sea_1_2 | 7 | 0 | 0 |
| Fld_Sea_2_2 | 6 | 0 | 0 |
| Fld_Sea_2_3 | 6 | 0 | 0 |
| Twn_Sea_1_1_A | 6 | 55 | 20 |
| Twn_Sea_2_1_A | 6 | 33 | 14 |
| Twn_Sea_2_1_B | 6 | 0 | 14 |
| Fld_Fst_1_1 | 7 | 2 | 0 |
| Fld_Fst_1_3 | 7 | 0 | 0 |
| Fld_Fst_1_4 | 6 | 0 | 0 |
| Fld_Fst_2_2 | 5 | 0 | 0 |
| Twn_Fst_1_1_A | 6 | 4 | 0 |
| Twn_Fst_2_1_A | 6 | 0 | 0 |
| Fld_Snw_1_2 | 7 | 0 | 0 |
| Fld_Snw_1_3 | 6 | 0 | 0 |
| Fld_Snw_2_2 | 6 | 0 | 0 |
| Twn_Snw_1_2_A | 5 | 0 | 0 |
| Twn_Snw_2_1_A | 6 | 0 | 0 |
| Fld_Ocn_1_1 | 5 | 0 | 0 |
| Dng_Wld_2_Job | 4 | 0 | 0 |
| Fld_Dst_3_2 | 5 | 1 | 0 |
| Fld_Isd_2_1 | 6 | 0 | 0 |
| Twn_Isd_2_1_A | 6 | 4 | 0 |
| Twn_Isd_2_1_C | 6 | 0 | 0 |
| Fld_Sea_3_1 | 7 | 3 | 0 |
| Twn_Sea_3_1_A | 6 | 0 | 0 |
| Dng_Ocn_1_3 | 6 | 0 | 0 |
| Dng_Ocn_1_1 | 6 | 1 | 0 |
