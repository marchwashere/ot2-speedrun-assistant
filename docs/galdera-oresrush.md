Current revision (2026-10-15): after the night Peddler hire, switch to day and
Steal Sturdy Pickaxe (ProperSteal22; guaranteed at Throné33). Switch path action
and close. Steal consumes an opening timer and a probability draw. Teleport
to Sai during day, then immediately switch to night before Skull Helm Entreat.
Both switches run through the native placement-refresh loader. This supersedes
the older continuous-night Pickaxe Entreat notes below.

# Warrior License and Oresrush supplies

After the Pickaxe, fast travel back to Sai, enter its East District, discard any
chapter-start prompt and remain at night. Agnea Entreats Elemental Augmentor
`ITM_EQP_ACS_250` from Townsperson `NPC_Twn_Dst_2_1_B_TALK_1800`, requiring
level25. BegItemExec checks level and grants the item without a probability
lottery. The user changes path action before closing, skipping the success
dialogue. This session adds only the common opening action timer (one draw).
The accessory is recorded but not equipped. Immediately switch to day and
Steal Reinforcing Jam from Townsperson `NPC_Twn_Dst_2_1_B_TALK_0400` outside
the orphanage. ProperSteal35 guarantees success at level46. Switch path action
before closing: one opening timer and one success check, no closing lottery.
Teleport to Winterbloom in daytime, then switch to night before Pickaxe delivery.
Remain at night for the Soldier Entreat; switch day after returning to Sai.
Loader evaluates the actual time-switch callbacks and preserves the hired Peddler.

After entering Sai, get the first Warrior License without equipping a new job or
learning skills. `SYS_WAR_GUILD_0010` and `0030` have `MapID=None`,
`EncountReset=false` and fixed dialogue; 0030 clears406/sets411. License JobID2
is distinct from GuildData ID1. The source-derived current route consumes zero
CRT at this stage, verified in the route replay.

Fast travel to Oresrush (`Twn_Wld_1_1_C`), discard chapter selection, switch night,
hire the Peddler `NPC_SS_TW11_0100_0100`, stay at night and Entreat Sturdy Pickaxe
from the indoor Retired Miner `NPC_SS_TSn21_0200_0200`. Both NPCs have all-day
placements gated by9000 and skipped Partitio prologue; Hire itself is night-only.
The Peddler costs1600 leaves, grants Smooth Talker (`SPACT_MERCHANT_014`) and uses
supporter `SUP_SS_TW11_0100_0100`. Hire stores the follower's identity/talent and
suppresses the original placement on later NPC refreshes. Character manager's
RefreshFollowCharacter uses pooled party follower actors; no placed-NPC random
frame is generated for this conversion. A full leaves balance is not modeled.

Pickaxe has ProperBeg30. Agnea level30+ qualifies, and the carried Agnea is70.
Miner CommandEnable7 enables Beg (bit4), and the all-day placement is present
at night. BegItemExec grants the item without a probability check. Store the
acquired quest item in `WorldState.route_items`.
Both inventory and hired NPC identity belong in exact search-cache keys.

Hire draws its common field-command action timer, then the success dialogue
lottery in BP_FCv2_FieldCommandFlowBase.DrawEventLabel. Two equal-weight events
are selected with the native15-bit float and <= cumulative comparison. The
selected FC_SHO_HIRE_Success_1/2 event has fixed command42 and no counter reset.
Pickaxe Entreat draws only its opening action timer. Change selected path
action before closing to suppress DrawEventLabel and the closing dialogue.
Peddler HireSuccesTalkID and Miner BegSuccessTalkID are both None; there is no
extra NPC-specific reaction draw.

Thus Hire adds two CRT draws and this one-item Entreat adds one; map/time
callbacks are evaluated separately using the carried state. These new action
endpoints await retail validation. Fixed steps appear only in Detailed log.

Data generation: after regenerating town data and Travelers Bag scripts, run
`tools/import-galdera-oresrush.cjs` to add Warrior and success-talk event scripts
and regenerate their source audit. The importer decodes the actual Blueprint
callbacks and saves the selected source functions with package hashes.
