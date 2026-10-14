# Shipwreck candidate

`galdera_shipwreck::run(entry, jp100, max_inputs)` assumes the boat is already
owned. Teleport Crackridge Harbor, board to the sea, enter Shipwreck with no sea
encounters. Map-load callbacks are carried through each transition; boarding
and travel still need comparison with a retail trace.

Stay/switch day for a single land Escape. Reject ambush and search minimum
0->1->0 boost cycles if needed for Escape success. Return exact inputs, actor,
actual escape percentage and final RNG. Temenos is not assumed high-level.

For JP100, switch night on Shipwreck entry. Apply the agreed gear menu here if
Nameless has not already done it; never consume nuts twice or create missing
gear. Same five opening branches as Nameless, with Partitio giving Temenos a
small Pom. T2 Temenos+3 BG then Osvald+3 Fireball. Osvald Boost-Start is required.
Validate actual BG effects, surviving recipients, resources and victory. All six
land formations are supported; no water encounters. Carry JP/EXP, resources,
knowledge, drops and Pom consumption. Manipulation ends at the JP victory.

Open Treasure_Dng_Ocn_1_1_06 for ITM_TRE_WPM_03 (Rusty Dagger). Fixed chest,
no RNG draw. User specified exactly one encounter: remaining chest distance is
unmeasured, so no counter decrement or guaranteed encounterless remainder is
invented. Automatic location selection and UI activation remain pending.

Validated: full JP searches for all six formations, both route branches through
the dagger, plus unchanged Nameless tests. Core suite69 tests passes.
