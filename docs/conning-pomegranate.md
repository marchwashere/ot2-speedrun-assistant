# Conning Harbor Pomegranate steal

Townsperson placement `NPC_Twn_Sea_2_1_B_TALK_0100_D000` is daytime in
`Twn_Sea_2_1_B`. NPCPurchaseData points to shop
`NPC_Twn_Sea_2_1_B_TALK_0100_NPCBUY`; row `_01` holds `ITM_CSM_0080`,
Energizing Pomegranate (M), ProperSteal 8, no special success event.

`FieldCommandWidgetSteal.StealItemExec` uses character level minus ProperSteal.
GameParam STEAL_PROBABILITY_* tables give these thresholds:

| Level difference | Success % |
| --- | --- |
| ≤ -11 | 3 |
| -10 through -4 | 8 |
| -3 through -1 | 15 |
| 0 through 1 | 55 |
| 2 through 3 | 65 |
| 4 through 10 | 80 |
| ≥ 11 | 100 |

The check always draws inclusive 1–100, including at 100% success. The generic
Steal session has menu action timer, success check, and closing dialogue draws,
matching the existing generic item-steal adapter. No information bonus is
obtained by this route. This new endpoint has not yet been checked in retail.

After selecting the prior route, freeze Northern and Western encounter opening.
Enumerate increasing cost frontiers: two inputs per boost/cancel cycle and
seven per time switch beyond the first two in each zone. Replay Western battle,
town loading, switches, Harbor loading, switches, and steal. Require daytime,
the Townsperson present, successful steal, and the original Western victory or
first-try Escape. Among equal-cost candidates prefer fewer switches. Actual
carried Throné level is read after the Western result awards any rare-pack EXP.

The result records the acquired item and quantity, level, chance, roll, and
post-steal world/CRT endpoint. Manipulation shows variable switch counts and
updated Western boosts; Detailed log carries intermediate checkpoints.

Regression checks cover threshold boundaries, unchanged earlier fight/opening,
successful steal and daytime endpoint, three session draws, and rejection of
every cheaper candidate for the fixture.

## Onward voyage

After the successful steal, fast travel to the previously tagged New Delsta
Harbor. `SC_LinerShip_Cty_to_Isd` has EventList MapID189 (ocean), command500
target69 (`Fld_Isd_1_3`, Beasting Bay: Anchorage), and EncountReset=true.
`SC_LinerShip_Isd_to_Sea` similarly loads ocean189 then command500 target123
(`Twn_Sea_1_1_A`, Canalbrine), with EncountReset=true. Both have presentation,
position, FinishPre9990, tutorial52 and finish10000 commands, matching the
existing skipped liner-ship adapter. Use shared source-imported map loading;
never replace these with fixed draw counts. No extra time switches are inserted.
Boat payment assumes sufficient funds, as with the earlier voyage.
Detailed log records all three arrival CRT/counter checkpoints. Endpoint
predictions await retail verification.

### Canalbrine gondola correction

Retail endpoint was 19287465 versus the original 19287459. Auditing every
streamed actor class found six gondolas in the visible base map
`Twn_Sea_1_1_A`: three BP_EnvObjMD_Gondola_A_NoPaddle and three
BP_EnvObjMD_Gondola_A_Paddle. Each ReceiveBeginPlay dispatches entry214,
calling RandomFloatInRange(0,20) then SetNewTime on the bobbing timeline.
Timeline update and finished callbacks do not draw again. These were absent
from the importer's supported environment classes. The correction imports
both classes wherever present, in the cooked level actor-array order. Existing
visible/created-level gating applies; no RNG offset is attached to Canalbrine.
The per-actor regression requires exactly six one-draw callbacks in Canalbrine.
`tools/audit-canalbrine-voyage.cjs` records actual voyage commands, rich sequences,
all streamed actor classes and their Blueprint parent/function bodies.
The rebuilt API replay for seed0/title19284723 now ends at19287465
(`0x82FC3E8B`), matching the reported retail endpoint. Four gondolas in
`Btl_Twn_Sea_A` remain excluded because the battle backdrop is not shown.

## Castti recruitment

After landing, interact with NPC_ALCHEMIST_0000_0200 while carrying a Healing
Grape. PlacementData's delivery branch with flags9000..37515 selects
MS_KUS_00_0100. Command5500 at32 is the Yes/No delivery dialog; native
dispatch140ffe710/1410011b0 calls EventManagerBP.StartSubDeliveryDialog and
UpdateSubDeliveryDialog. The chosen opt sets EventIndex, and Yes delivers the
item through SubDeliveryItem. No random selection appears in those callbacks.
Accept invitation126 Yes and chapter prompt135 No. Command3000 adds Castti to
the bench because four characters are active; dismiss the replacement menu
without changing them. Preserve the active party's ledger and initialize Castti
from her imported level1 starter, then apply the shared skipped-prologue flags
and progress points. Command9990 executes placement refresh and counter reset.
Her recruitment checkpoint follows Canalbrine arrival in Detailed log. A grape
is assumed available; general field inventory is not yet modeled. This endpoint
awaits retail verification.
