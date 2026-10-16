# Gravell restoration and supplies

After Scorched Bone Spear, Firestarter and the Crackridge book, fast travel to
Gravell and discard chapter starts. Skip both the Old Man and the Villager.
Sharp Nut (L) and Diffusing Serum are stolen earlier from the Shopkeeper inside
Wellgrove's inn; do not buy another Serum here. Strengthening Serum is stolen
with Champion Seeker earlier. No Lychee (L) is collected on this visit.
Teach/equip the planned Temenos and Partitio skills (no RNG draws).
Switch night and Ambush the Debt Collector (Throné30 required; carried70).
Talk to Porta, restore both rusty weapons, switch day, sell Lionheart's Axe
and Conqueror's Sword separately, then Steal Reinforcing Jam from Shopkeeper
NPC_Twn_Wld_3_1_A_TALK_0500. Change selected path action before closing Steal.
Jam ProperSteal35 plus native over-level11 means guaranteed at46.

`tools/import-galdera-gravell.cjs` reads/hashes local cooked sources and imports
the actual Armsmaster first-visit event. Run after other event importers.
No map or event total is hard-coded. Native map/time callbacks carry through.

Generic Ambush opens the path-action timer and draws one weighted before-talk
event (0.5/0.5). Success dialogue is fixed. The collector has no special Ambush
event, unlike the Chapter1 guard. Swoon is recorded until script command2500
cancels it; flag2400 then removes the original blocking placement at FinishPre.

Armsmaster commands8601/2/3/4 execute the source first-visit flow. The game uses
sorted backpack order, so Sword converts before Axe in this dialogue. Conversion
removes rusty items, grants restored items, sets flags2402/2405, grants license8
(2417), unlocks both weapon skills globally and Peak Performance for recorded
characters. No supports or jobs are equipped. Two weapons do not complete the
six-weapon sidequest. Fixed dialogue, next-tick notifications and conversions
have no CRT draw; FinishPre still evaluates actual changed NPC placements.
Unsupported revisit/buyback states fail explicitly.

Sales call ShopMenuWidget.CompleteTrade -> ExecMerchantSpecialAction_WithoutNpc
(type1), thunk1410d9110 ->140e78f00 ->140e78a30. Smooth Talker SPACT_MERCHANT_014
uses sell-high range25..25: **one rand per sale even for the constant range**.
Each weapon's100000 base sale gives125000, total250000. Proceeds are stage
metadata, not a complete route wallet (earlier costs/income are not all tracked).
The result-message formatter140e7f760 performs no lottery.

Jam adds one opening timer and one Steal success roll even at100%; path-action
switch/close suppresses the normal closing dialogue lottery. The item remains
unused in inventory. All fixed stages appear in Detailed log/export.

Source-derived endpoints; retail trace validation remains pending.
