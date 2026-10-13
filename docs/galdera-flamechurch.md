# Al to Eastern Flamechurch Pass

After Al hand-in, run100% on land for56 updates to Western Clockbank Highroad.
Daytime Evasive Maneuvers gives6 counter points/update:336 total, below the
minimum post-quest counter500. Load Fld_Cty_1_4 normally.

The user confirmed the next193 running updates are entirely on land. Cost is
1158, exceeding first-entry maximum899, so one encounter must occur. First
trigger is84..150 updates; execute native first-menu Escape100 with carried
party/supports/stats/weakness history. Field restoration clears FirstEncount
and rolls the next counter at minimum1000. Remaining43..109 updates cost at
most654, so no second encounter occurs. Pay the actual remaining movement and
then load Eastern Flamechurch Pass Fld_Mnt_1_2. Do not simulate a fight there.

Western Clockbank land volume EVM_FLD_CTY_1_4_LND_DAY references New Delsta
ENC_FLD_CTY_1_3_LND_DAY_* tables. All enemies/initialization already exist in the
core. Eastern Flamechurch map6 and its native load callbacks are imported by
import-town-loading. Reapply the Bag/Oresrush/Winterbloom/Wellgrove-supplies
scripts after regenerating town data.

Reject Surprise. Freeze Al and all earlier route decisions; if needed, replay
complete night/day pairs at Western Clockbank entry until a Normal/Preemptive
opening is found. Return to day each time and preserve the encounter counter.
Show only required pairs, actual formation/first actor and Escape100 in the
Galdera Clockbank card; retain100% running representation. Fixed movement,
map-entry and field-return checkpoints appear in Detailed log.

Regression exercises direct and adjusted seeds, replays time callbacks, checks
movement consumes no RNG, one encounter, successful native Escape and positive
exit counter. Full transcript/live API checks verify earlier Al endpoint stays
unchanged and party/items/EXP survive the extension. Retail validation pending.
