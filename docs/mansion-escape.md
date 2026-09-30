The Manipulation continuation now predicts the first mansion encounter and
searches boosts at Throné's first menu for BOTH successful first Escape and a
counter sufficient to run to the Pirro trigger. It imports the standalone battle
core; no combat or CRT voice implementation is copied into this repository.

The user measured **161 counter-decrement updates** on 2026-09-28, running with
unchanged equipment. This is treated as the full mansion-entry-to-Pirro-trigger
path, including the lookout approach. The approach is encounter-free and its
cost is included once in the total, not subtracted twice. The distance is
hardcoded at 161 updates, with no settings or API override. Settings controls
this event's boost repeat limit (default 4, or Unlimited). Existing first-fight
budgets remain independent.

The active night zone is ENT_DNG_CTY_1_1_LND / EVM_DNG_CTY_1_1_LND_NGT,
progress 0, danger 3. Night decrement 9 * dash 1.4 truncates to **12**. First
encounter is fixed to ENG_DNG_CTY_1_1_LND_DAY_110, one Diamante Guard I, no group
lottery. The current map-entry counter survives lookout Ambush. Movement draws
no CRT. The field selector consumes trigger reset B/A, situation and party timing;
the combat core continues from before enemy initialization without redrawing the
player timing roll. Rare families are disabled in this volume.

For initial counter C: encounter occurs on ceil(C/12); remaining running updates
are 161-ceil(C/12). Successful Escape returns through `On Shown Field After
Battle Impl` -> `ResetEncountStep`, two CRT draws B then A, with FirstEncount now
false. This is the full normal counter range 1000..1798. Require
`counter > remaining_updates * 12`; equality triggers an encounter and is unsafe.
Discard any first-trigger overshoot when the counter is reset. Stop before the
Pirro cutscene; do not execute 1800 or pretend that entering its trigger is a
completed boss-area map load.

The route stays level 2: opening yields 4..6 EXP, fight two 7..9, and level 3
needs 16 total. Events 0700 and 1100 execute command 2310 (StartFullRecovery),
so use full level-two HP/SP while retaining latent gauge. The original equipment
is unchanged. BanFlag 652 is off after 1600, enabling the night talent.

Search clones the actual first-menu snapshot and replays every boost input,
Escape check, and field counter reset. It minimizes button presses, then CRT
draws, ending at zero selected BP before Escape. Available BP is read from the
menu, including surprise-round regeneration. Finite caps count full upward
climbs; Unlimited covers CRT offsets until the shortest plan is proven or the
cycle is exhausted. Impossible remaining distances are rejected before search.
Failed Escape candidates never reset the field counter. JSON exports preserve
the fixed route distance and repeat cap, the exact input path, states and callback trace.

This stage is source-derived and awaits a live command-menu/Escape comparison.
The existing confirmed mansion-entry endpoint remains unchanged. Extra movement,
menu opens, equipment changes or other field actions are not assumed.
