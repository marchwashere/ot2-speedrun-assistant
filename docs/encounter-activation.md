# Active encounter tables

Source: EncountTrigger.SetEncountVolume selects the first true replacement flag in authored array order. ExecuteUbergraph calls it from collision activation; BP_Canoedock.ChangeToWalk and ChangeToCanoe call it explicitly for their destination terrain. CheckTriggerActive requires all nonzero start flags and no nonzero end flags. A flag is not a subscription that automatically reevaluates the selected table.

WorldState.active_encounter stores map, base trigger label and selected label. Call enter_encounter_zone only at an actual activation boundary. For ambiguous physical bindings use enter_encounter_binding. Ordinary battle returns, flag changes and time switches preserve this selection; map changes invalidate it. Do not invent movement/overlap events from a boss victory. The low-level encounter catalog still accepts an explicitly selected label for audits. Route code uses the world selection; missing activation on a replacement-bearing base raises an error instead of guessing from current flags.

Nameless Isle records land entry, canoe boarding, and landing before Gigantes. Its post-boss state therefore carries the original land table until a later explicit activation. Quicksand similarly records land activation before its boss. Both can now support an added post-boss encounter without selecting a replacement merely because the boss flag changed.

The importer records all 56 replacement-bearing physical/terrain bindings across 22 maps from the extracted level placements, including empty base tables and authored rule precedence. All 14 dungeon maps are covered:

- Abandoned Waterway (Dng_Cty_1_3)
- Old Clock Tower (Dng_Cty_2_3)
- Quicksand Gaol (Dng_Dst_2_2)
- Starfall Spring (Dng_Fst_1_2)
- Sinking Ruins (Dng_Isd_2_1)
- Verdant Wood (Dng_Isd_3_2_B)
- Stormy Cape (Dng_Isd_3_2_C)
- Forsaken Graveyard (Dng_Mnt_2_1)
- Stage of the Moon and Sun (Dng_Mnt_3_1_B)
- Curious Nest (Dng_Ocn_1_2)
- Nameless Isle (Dng_Ocn_1_3)
- Infernal Castle (Dng_Snw_3_4_B)
- Unfinished Tunnel (Dng_Wld_1_2)
- Ivory Ravine (Dng_Wld_3_2)

Regenerate with node tools/import-encounter-activation.cjs. The evidence includes both trigger and canoe source functions. Tests cover every binding, Nameless water/land changes, retaining the active table across flags/time/battle return, explicit reactivation, missing-state rejection, and search-cache separation. A test is a model regression, not a retail capture for every dungeon; the live Quicksand checkpoint is retained separately.
