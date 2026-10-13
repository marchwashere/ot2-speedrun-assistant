# Sai return to Eastern Wellgrove Trail

After Winterbloom's nighttime Soldier Entreat, fast travel to Sai and discard
any chapter start. Immediately switch to day. Cross Southern Sai Sands without
an encounter, enter Western Sai Sands and take one land encounter before
entering Eastern Wellgrove Trail. The Southern crossing is user-confirmed;
no unmeasured walking budget or battle is fabricated. The route ends on Eastern
Wellgrove entry, without a fight there.

Shared Loader evaluates every map load and time callback. Western Sai's land
volume EVM_FLD_DST_2_2_LND_DAY actually references Southern Sai encounter tables
ENC_FLD_DST_2_3_LND_DAY_*; all enemies are already supported by the core. Map
Fld_Dst_2_2 and Eastern Wellgrove Fld_Fst_2_2 are imported by town-loading.

Carry active travelers, actual levels/jobs/supports, weakness history and
inventory into the battle. Accept Normal/Preemptive and execute first-menu
Escape using the native level/danger calculation; require100% and real success.
Restore the field/history/counter before loading Eastern Wellgrove.

Earlier route controls stay frozen. If the immediate-day departure is ambushed,
try complete additional night/day pairs in Sai, replaying callbacks and both
field loads each time. The first safe candidate wins. Instructions show the
extra switches only when needed, followed by the actual first actor and Escape.
Fixed travel checkpoints appear in Detailed log.

Regression exercises direct departure and a seed needing extra Sai time cycles,
replays those controls and checks no ambush, guaranteed escape and retained
state. Full transcript replay and served UI checks cover the new endpoint.
Retail validation remains pending.
