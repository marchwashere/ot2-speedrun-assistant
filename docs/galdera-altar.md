# Altar of the Trader detour

Galdera only: Inventor -> Crackridge Harbor encounter -> Southern Crackridge
encounter -> Altar of the Trader -> Southern Crackridge encounter -> Western
Crackridge encounter -> town -> Western return encounter -> Merchant/Chubby.
Any% Hikari retains its four-encounter continuation. Galdera now has five.

Run until the first Southern encounter. The measured whole approach to the altar
is 286 running or 301 walking updates. Subtract actual running updates already
spent reaching that encounter; convert the remaining distance to walking with
upward rounding and add three updates to both remaining movement endpoints.
Enumerate whole running updates and ceil the remaining walking share. Daytime
Evasive costs 7 running / 5 walking per update. Require a strictly positive exit
counter. Failed escape/reset candidates continue the real boost search, including
the existing safe Defend options. The UI percentages apply to the remaining
journey AFTER Escape. No extra time switches are added on this leg.

Map95 Dng_Wld_2_Job and SHRINE_SHO_0100 are imported by import-town-loading.cjs.
The event's command8910(target4,opt1) unlocks Partitio's second EX slot:
ABI_SET_MER_090, **Negotiate Schedule**, not Share SP (ABI_SET_MER_080).
Native e9c5c0 -> f34de0 changes the separate advanced-ability save array.
There is no JP cost, ordinary skill count, support unlock, or RNG lottery.
Store this in CharacterJobs.ex_abilities, separately from learned_abilities;
both serialization and exact state keys retain it. Flag2503 removes the trigger.
EncountReset=false; the actual altar load and Southern re-entry still run their
native loader callbacks/counter resets. See data/source/galdera-altar.json.

No measured return distance was supplied: the return follows the user's specified
single Southern encounter before Western, without inventing a movement budget.
Require a successful escape and non-ambushed next opening at each stage.

The Galdera baseline/refinement departure window increases from1900 to3000 draws,
with preferred trial gap1400 instead of850, to accommodate the added fight and
loads. These only order/filter search candidates; they do not advance RNG or
replace simulation. Adaptive rescue still has no hard departure-gap restriction.
The complete fight must retain Chubby, EXP100, victory, and all four survivors.

Validation: movement endpoint enumeration, EX2/JP/support/RNG invariants, separate
UI cards and percentage display, and full route replay. New retail checkpoints
remain unverified.
