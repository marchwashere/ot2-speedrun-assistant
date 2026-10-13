# Winterbloom supplies

Continue from the daytime Sai East District Reinforcing Jam Steal. Fast travel to
Winterbloom, discard chapter starts, then switch to night before Pickaxe delivery.
Carry the hired Smooth Talker Peddler
and Sturdy Pickaxe from Oresrush.

The quest NPC selects Pickaxe delivery SC_SS_TSnow21_0200_02A0. Confirm Yes:
SubDeliveryItem consumes the Pickaxe. The completion script sets20393 and34155,
refreshes NPC placements at command9990, resets the encounter counter using
native B/A draws, and grants Rusty Sword, Revitalizing Jam and7000 leaves.
The temporary scene actor uses KSFieldCharacter and does not generate a
randomized field-NPC frame. The party-count branch includes following NPCs:
four active travelers plus the hired Peddler is five, below the eight-person
alternative. No preliminary quest acceptance is needed with the Pickaxe.

Entreat the nighttime Soldier NPC_Twn_Snw_2_1_A_TALK_1500 for Fortifying Nut(L),
ITM_CSM_0550. ProperBeg26 requires Agnea level26. Change selected path action
before closing to skip the success dialogue lottery. The session draws only
the opening path-action timer, and records the nut without using it.

Source import: tools/import-galdera-winterbloom.cjs writes runtime data and the
source/Blueprint audit, and adds the quest event to town-loading.json. Run it
last after town, Travelers Bag and Oresrush event imports. Regression verifies
Pickaxe consumption, rewards, NPC removal, counter reset and one-draw Entreat;
full route replay verifies prior endpoints remain unchanged. A full money
ledger is not modeled;7000 is recorded as reward metadata. Retail verification
of these new checkpoints remains pending.
