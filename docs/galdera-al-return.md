# Return Al's bag after Dancer License

Switch to day in Wellgrove immediately after Magic Nut L Entreat, then obtain
the Dancer License. Teleport to New Delsta main during day, discard chapter
prompts and enter New Delsta Highroad without another time switch.
Talk to Al and confirm Yes to deliver his recovered bag. No distance was
supplied for the direct entry-to-Al leg; no extra encounter or movement budget
is fabricated. The route ends after quest completion at daytime Highroad.

SC_FC13_Tutorial_0310 command29 grants ITM_TRE_SS_TD11_0100_0010 after the earlier
thief fight. Record this existing zero-draw grant in route_items. Delivery
SC_FC13_Tutorial_0400 requires20510 and the actual bag. Command5500 Yes invokes
SubDeliveryItem and consumes it;20511 marks completion and suppresses Al.
Native9990 refreshes placements and resets the encounter counter B then A.
The event references map46 but begins on that same map, so no extra load occurs.
Fixed dialogue and temporary scene actor setup do not add CRT draws.

Reward:1500 leaves, Slippery Nut ITM_CSM_0680 x1, Healing Grape ITM_CSM_0010 x3.
Record items and money metadata; a full wallet ledger is not modeled. Keep nuts
unused and carried characters/jobs/EXP unchanged. Chapter arrivals are discarded.

import-galdera-al-return.cjs reads the actual delivery placement, reward table,
bag grant and event metadata, plus Blueprint SubDeliveryItem/temporary actor
callbacks. Existing Bag events remain owned by import-travelers-bag.cjs.
Regression verifies the correct day-switch location, bag consumption, rewards,
Al removal, native counter draws and unchanged carried state. The full Galdera
transcript exercises the real earlier bag grant. Retail validation pending.
