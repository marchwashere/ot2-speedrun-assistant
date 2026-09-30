# Mansion lookout: encounter-counter boundary

Source audit, 2026-09-28: the lookout's scripted Ambush does **not** reset the
encounter counter. Walking to him and walking onward share the existing counter.
This is a source result, not a live counter measurement.

- `NPC_Dng_Cty_1_1_0100_N000` selects `MS_TOU_10_1700` through its
  `eFC_ASSASSINATE_SUCCESS` event while flag 8016 is off.
- EventList: map 50, `EncountReset=false`, `EncountTrigger=None`.
- Script 1700 plays fixed Ambush lines/fades, sets 8016, clears 25305, calls
  9990 (FinishPre), and ends with 10000. No map transfer or serial event.
- EventManagerBP.FinishPre guards ResetEncountStep with PlayEventData.EncountReset.
  It therefore skips the counter roll for 1700, including the implicit final
  cleanup (the native FinishPre also guards repeated execution).
- A placement/encounter-volume refresh is not a counter reset:
  EncountTrigger.SetEncountVolume -> EncountManagerBP.SetEncountVolume ->
  GetCurrentEncountVolume / SetEncounter selects active encounter data without
  calling ResetEncountStep or CalcEncountStepCount. The placed mansion trigger
  ENT_DNG_CTY_1_1_LND and its volume have no flag gates or replacements.
- The separate next story trigger, `EV_TRIGGER_MS_TOU_10_1800`, is enabled by
  8016. Event 1800 **does** have EncountReset=true; its event-end cleanup resets
  the counter after that cutscene. It is not a serial command in Ambush 1700.

## CRT continuation now implemented

`mansion.lookout_ambush` runs after the separate mansion-entry boundary. It
predicts **5562 / 0xDDCC3032 -> 5564 / 0x5121903C** for the current route:

| Boundary | Advances | CRT |
|---|---:|---|
| Before opening Ambush | 5562 | 0xDDCC3032 |
| Menu opened (CRT action timer) | 5563 | 0x3425562D |
| After Ambush and placement refresh | 5564 | 0x5121903C |

The level-one requirement is deterministic, with no success roll. This lookout
has a type-20 / None story override, so the ordinary dialogue lottery is skipped.
Both 1700 dialogue lines are fixed. Its NPCBattleData has all-zero SwooningFlags.

Flag 8016 disables awake placement `NPC_Dng_Cty_1_1_0100_N000` and enables
`NPC_Dng_Cty_1_1_0100_N010` (same resource, authored sleeping action 36).
Native ed5040/ecda30 looks up existing actors by **placement label**, so the new
label spawns and consumes one NPC starting-frame draw. The world evaluator
derives that call from the imported placement gates, not a fixed +2 offset.

Manipulation displays after-Ambush CRT beneath mansion entry. Detailed log and
JSON contain the menu checkpoint, both draws and resulting flags/placements.
The 5564 endpoint remains source-derived, awaiting live validation.

The short walk to the lookout is assumed encounter-free as specified by the
user. Its unmeasured counter cost is not deducted by this CRT continuation.
`ambush_lookout` preserves whatever counter the caller supplies; consumers must
apply the approach's measured walking/running cost before using it to forecast
an encounter. No counter is displayed for this stage in Manipulation.

Evidence: cooked PlacementData, PlacementList, EventList, scripts 1700/1800,
and decoded EncountTrigger / EncountManagerBP functions in
`outputs/mansion-lookout-audit` at the parent workspace. FinishPre is in
`outputs/throne-city-audit/FinishPre.txt`, particularly the EncountReset guard.
