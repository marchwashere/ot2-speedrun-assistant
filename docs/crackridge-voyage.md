# Clerics to Crackridge Harbor

After the selected black-market route, buy soulstones, dismiss the shop, fast
travel to previously tagged New Delsta Harbor, switch immediately from night
to day, then buy passage to Crackridge Harbor and skip the sailing sequence.
Ordinary purchases and ship selection/payment do not perform CRT lotteries.
Money and inventory are not modeled; sufficient funds are assumed.

`Shop/Database/LinerShipRoute`, SHIP_ROUTE_001, maps the eastern harbor
Fld_Cty_1_1 to western harbor Fld_Wld_1_2 for 2500 leaves. The shop reverses
the row for the departure side, then its UpdateState_LinerShip invokes
PlayEventForce using SC_LinerShip_Cty_to_Wld. This is an event transition,
not a direct fast travel to Crackridge.

The EventList row has MapID189 (Fld_Ocn_1_1), EncountReset=true. Event commands
1500/1501 play the sailing rich sequence; command500 loads MapID85
(Fld_Wld_1_2); command230 positions the player; command9990 finishes with the
destination counter reset. Both map loads use outgoing/incoming counter rolls,
flag gates, cooked stream constructors, environment callbacks, NPC placements
and tavern gates through the existing shared loader. Counter draws retain the
first-encounter ratio. No total advance constant is substituted.

`tools/audit-crackridge-voyage.cjs` records the route row, shop callback,
sequence event keys, new map packages and their actor classes/parents.
Random stream calls use their own streams; construction-only random calls
on cooked actors do not replay during these loads. The sequence's ship actor
and balloon presentation have no battle initialization or voice lottery.
The prediction assumes the sequence is skipped and no incidental encounter,
path action or extra time switch occurs. A retail trace is still needed to
validate the complete endpoint.

`crackridge::after_market` carries the actual chosen Mug/clerics RNG and world
state into this series. Detailed log records shopping completion, Harbor night
arrival, day switch, boat departure and Crackridge field control, including
each CRT draw. Manipulation receives no status-only widget.

The route now continues through the first daytime land encounter in
Crackridge Harbor: Anchorage, ENT_FLD_WLD_1_2_LND. The carried first-encounter
counter is consumed by movement, then the selector resets B/A and chooses
the fixed first formation, surprise/preemptive roll, two traveler timing
draws and rare replacement. Main progress is 2 after Throné Chapter 1 and skipping Osvald's story.
The NO callback invokes SkipPrologueReminiscence (FUN_140f15190), which
completes the character's prologue rows through FUN_140f54e00 / FUN_140f4ec50
and adds their ClearDifficultyPoint. Osvald's rows600/601 contribute1/0.
EndPrologueReminiscence is a separate restoration callback; its early return
does not undo skip completion. Crackridge remains in the lower tier at2;
Danger is5. WorldState carries the points into later selections.

`crackridge::plan` first obtains the existing minimum-cost clerics plan. It
tests the established Harbor day switch, then a day switch in Flats after
shopping but before teleporting. Each option replays the shared loader's
map-specific time/NPC callbacks. If either avoids surprise, keep the Mug
plan unchanged. Otherwise jointly search Mug R2 boosts and Cape Cold time
switches for clerics AND a safe first Harbor encounter. Required has no cost
ceiling. A numeric optional budget allows two additional weighted input
units per repeat above the clerics plan; time switches cost7 each. The
separate clerics boost cap remains in force. All earlier checkpoints stay
fixed. The selected switch location appears as a player instruction.

The standalone battle core's `FieldEscapeMenu::new_party` initializes
original-gear Throné at the propagated route level and Lv1 Osvald. It uses
each actor's actual speed and voice rows, both turn queues, random opening
speaker and Study Foe. Study Foe draws once per distinct new enemy type;
copies receive the same weakness icon without another draw. A promoted
first traveler acts before enemies on normal openings. Clone this menu to
test minimal 0→1→0 boost cycles until the first Escape succeeds; a preemptive
opening bypasses the Escape lottery. The shared field loader then restores
the zone and rolls a normal post-encounter counter. No failed attempt is
played and no enemy action is simulated. Instructions identify the actual
first actor. CRT menus/check/return remain in Detailed log.

New dual-party initialization and this voyage/encounter remain source-derived,
pending a retail trace. This profile assumes no earlier regional weakness
discoveries and no intervening CRT-consuming actions, encounters or resets.
It does not predict a second encounter without a measured remaining route.
Reproducible Study Foe audit: battle-core tools/audit-field-party.cjs.
