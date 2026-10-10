Western Winterbloom Snows entrance -> Scholar Guild -> Winterbloom town tag
uses 345 running counter updates, with no movement-error allowance.

The carried world's Evasive Maneuvers flag determines the first approach rate.
A first Western rare is killed with Shadow Soulstone (M); its real formation
rewards update both characters' EXP/JP, levels and stats, and the support skill
is equipped immediately after victory. Otherwise both encounters are escaped.
Escape searches and surprise selection use the actual EXP-derived party levels.
No planned opening, including the movement-error override, may be Surprise.

The first Escape is searched for success and a safe next opening. The second
Escape is searched for success and a reset covering the remaining measured run.
The extra-encounter override advances movement to the actual counter trigger,
even when that exceeds the perfect route estimate, then uses its immutable
Escape and town continuation. Selecting it never mutates prior route RNG.
When a second encounter is inevitable on perfect movement, avoiding it is not
offered. Third encounters are rejected by the second Escape's movement check.

GuildData GUILD_005 targets NPC_Fld_Snw_2_2_GUILD. Introduction and license
events SYS_SCH_GUILD_0010/0030 contain fixed dialogue, license flags406/411 and
no encounter reset. GuildMenuWidget.AfterLicenseAcquire opens tutorial59;
its dismissal has no counter callback. These events have MapID=None and stay
in place. License flag changes use the shared event placement refresh. Since
the interaction has no RNG calls for the current route's existing NPCs, its
position relative to an encounter does not change battle RNG.

Winterbloom map170 is imported with all placement candidates and streamed
levels. Shared loading evaluates real flags, NPC frames and outgoing/incoming
counter callbacks. RefreshBarCharacter draws its appearance lottery BEFORE
checking benched characters; an empty bench still consumes that lottery, but
no character-selection draw. Nonempty benches remain explicitly unsupported.

BPC_LevelManager_Others.ShowStorySelectMenu requests MapInStorySelect. Its
fixed dialogue and cmd7110 open the arrival chapter selection. Closing without
starting Throné chapter2 changes neither story progression nor CRT state;
EncountReset=false. The town endpoint includes this closure.

Tests cover support timing, level propagation, both immutable encounter-count
branches, successful escapes, no-surprise, guild/popup no-reset behavior and
safe remaining movement. Newly added Western/town boundaries are source-derived
and still require retail trace comparison.

The active fork continues from each immutable Winterbloom timing branch through
fast travel to the already-tagged New Delsta Harbor and the reused skipped boat
event SC_LinerShip_Cty_to_Wld, ending at Crackridge Harbor. After dismissing
Winterbloom town’s story prompt, switch to night. Arrive at New Delsta Harbor
at night, then immediately switch to day before the boat. Both fixed switches
use shared manual time/bar/NPC/lottery callbacks without resetting the encounter
counter. The town checkpoint includes the night switch; the Harbor checkpoint
includes the day switch. Both encounter-count branches and their safety searches
replay these events. These timer resets do not model elapsed travel time.

Every candidate Winterbloom endpoint is projected through these loads and the
first Crackridge encounter. Surprise candidates are rejected in Western Escape
search (or upstream rare/escape search when there is no second Western fight).
Both the predicted route and the second-encounter override must be safe.
Crackridge menu boosts search ascending actual input cost for a successful first
Escape and a safe remaining journey. Real party EXP-derived levels, Evasive
Maneuvers and enemy weakness/Study Foe history are carried throughout.

Boat to Eastern Cropdale Trail: measured 172 running /182 walking updates.
Run to the first encounter, subtract its trigger updates from 172, then convert
the remaining walk density with ceil(remaining_run *182/172). Mixed movement
interpolates those densities, reserves one counter point, rounds running share
down to 0.1%, and rounds consumption up. This is an estimate of mixed movement,
not an exact simulation of per-frame overshoot. Arrival loads Fld_Fst_1_3 with
shared Loader. Endpoints are source-derived and await retail trace validation.

Crackridge's Escape search now also projects Eastern Cropdale Trail loading
for every candidate and requires a safe walk into Cropdale. Measurements are
144 walking /134 running updates; the user-approved three-update allowance
makes these147 /137. Daytime Evasive Maneuvers gives decrements4 walking and6
running: costs588 /822, strictly requiring counters589 /823. The running share
is floor(1000 * (counter-1-588)/(822-588))/10 percent, clamped at100% once a full
run is safe. Cost interpolates route distance from the independently measured
walk/run totals, reserves a point, and rounds consumption up. This remains a
mixed-movement estimate, not a per-frame overshoot simulation. No additional
RNG action is inserted between Trail entry and Cropdale. The route then loads Cropdale through shared Loader (Twn_Fst_1_1_A,
MapID145), carrying residency, NPC/progression state and counter callbacks.
Agnea's first invitation executes MS_ODO_00_0200:125 YES and :134 NO, skipping
Chapter 1. Fixed dialogue/event-only actors do not consume CRT; FinishPre
refreshes changed placements and resets the counter, followed by tutorial73.
Preserve prior character EXP/JP/equipment/history, append Agnea ID8 at level1
and zero EXP/JP with imported original equipment and dancer growth.
Checkpoints and full traces appear in Detailed log and JSON. Source-derived;
awaiting retail validation. Seed0/title4997/shared retained/Before Osvald/
Western second encounter: Trail6768 -> Cropdale6802 -> Agnea6804. Retail Cropdale entry6802 was
reported on 2026-10-02. The former6798 omitted four BP_EnvObjMD_Flag_A
BeginPlay calls (EnvObjMD_Flag_A_5, _6, _8, _9). Each selects an initial
animation speed with RandomFloatInRange and SetPlayRate. Import instances in
level actor-array order and execute the callback through shared Loader;
never add a fixed Cropdale correction. Agnea6804 awaits retail validation.

After recruitment, load Eastern Cropdale Trail again with the three-member
party. Crackridge Escape candidates also project this future encounter and
reject Surprise before selecting their plan. The return-leg first command
menu uses actual character speeds, three-member speaker/voice lotteries,
Agnea's imported ordinary voices and Osvald's persistent Study Foe history.
Choose the cheapest boost/cancel path that makes the first Escape succeed,
then load Southern Cropdale Trail (Fld_Fst_1_4, MapID132) with shared Loader.
The user guarantees one encounter and no second on this leg; no unmeasured
movement length or leftover Eastern counter is inferred. Outgoing and incoming
map-load resets roll the real Southern entry counter. The route card names
enemies and the command-menu actor, followed by complete boost inputs/Escape;
map/RNG checkpoints remain in Detailed log. Source-derived pending retail trace.

Progression correction (2026-10-02): the previous return prediction used
progress1 and tier010, producing one Birdian II. The native NO callback for
skipping a recruited prologue calls SkipPrologueReminiscence (f15190), then
f54e00 completes the character's CHARA_PROLOGUE story rows via f4ec50.
ClearDifficultyPoint is added to save +0x648 (GetMainProgressPoint). This
happens separately from EndPrologueReminiscence, whose early return was
mistakenly treated as evidence that skipping added no progress. Import the
points and carry completed scenarios: Throné700 contributes1, Osvald600/601
contribute1/0, Agnea800 contributes1. Total3 selects Eastern tier020/danger8,
with no extra RNG calls from point bookkeeping. All route selectors now use
the persistent value, not a fixed1 or party-size substitute. Native call
bodies are preserved in outputs/main-progress-native.txt.

GUI verification for seed0/title4997/shared retained/Before Osvald/Western
second encounter: Cropdale6802, Agnea6804, Eastern return6817; now correctly
Woodland Birdian I x2 + Woodland Birdian II, non-surprise. Throné first menu
6877; no extra boosts, Escape; field6880 /0xAE2E29E0; Southern Cropdale
entry6895 /0x1FCCDF71. Existing Crackridge adjustment stays one boost/cancel.
Enemy formation confirmed by user; new battle/escape checkpoints await retail
comparison. The former one-enemy menu6860/field6864/Southern6879 values were
invalid and are superseded.

## Southern Cropdale map identity correction (2026-10-09)

Retail trace southern-cropdale-load-live-20261009 captured12 consecutive draws
from19286486 to19286498. SpawnNPC identifies NPC_SYS_SAVE_Fld_Fst_1_4_0000
and SYS_INFO_BOARD_FST_1_4_0010. The route was incorrectly mapped to
Fld_Fst_1_1 (MapID129), which added a GD streamed level and two flag draws.
Use Fld_Fst_1_4 (MapID132), six streamed levels and no imported scenery draws;
use ENT_FLD_FST_1_4_LND for movement budgeting. Counter parameters remain
minimum1000, A/B ranges400 and first ratio0.5. Regression compares all12 pre-draw
states, positions, NPC labels and final state, not an endpoint offset.
