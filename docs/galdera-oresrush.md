# Warrior License and Oresrush supplies

After entering Sai, get the first Warrior License without equipping a new job or
learning skills. `SYS_WAR_GUILD_0010` and `0030` have `MapID=None`,
`EncountReset=false` and fixed dialogue; 0030 clears406/sets411. License JobID2
is distinct from GuildData ID1. The source-derived current route consumes zero
CRT at this stage, verified in the route replay.

Fast travel to Oresrush (`Twn_Wld_1_1_C`), discard chapter selection, switch night,
hire the Peddler `NPC_SS_TW11_0100_0100`, then switch day and steal Sturdy Pickaxe
from the indoor Retired Miner `NPC_SS_TSn21_0200_0200`. Both NPCs have all-day
placements gated by9000 and skipped Partitio prologue; Hire itself is night-only.
The Peddler costs1600 leaves, grants Smooth Talker (`SPACT_MERCHANT_014`) and uses
supporter `SUP_SS_TW11_0100_0100`. Hire stores the follower's identity/talent and
suppresses the original placement on later NPC refreshes. Character manager's
RefreshFollowCharacter uses pooled party follower actors; no placed-NPC random
frame is generated for this conversion. A full leaves balance is not modeled.

Pickaxe has ProperSteal22. GameParam `STEAL_PROBABILITY_LEVEL_OVER.Init=11`
and `STEAL_PROBABILITY_OVER.Init=100` make it guaranteed at level33 or higher.
Carried Throné is level70. Still execute the real RandomIntegerInRange1..100
success check. The acquired quest item is stored in `WorldState.route_items`.
Both inventory and hired NPC identity belong in exact search-cache keys.

Hire draws its common field-command action timer, then the success dialogue
lottery in BP_FCv2_FieldCommandFlowBase.DrawEventLabel. Two equal-weight events
are selected with the native15-bit float and <= cumulative comparison. The
selected FC_SHO_HIRE_Success_1/2 event has fixed command42 and no counter reset.
Steal draws its action timer, item success roll, then its ordinary closing
dialogue lottery FC_TOU_STEAL_Success_1/2. No Conning Creek voice-suppression
exception is assumed here. Source Peddler HireSuccesTalkID and Miner
StealSuccessTalkID are both None; there is no extra NPC-specific reaction draw.

Thus Hire adds two CRT draws and normal one-item Steal adds three; map/time
callbacks are evaluated separately using the carried state. These new action
endpoints await retail validation. Fixed steps appear only in Detailed log.

Data generation: after regenerating town data and Travelers Bag scripts, run
`tools/import-galdera-oresrush.cjs` to add Warrior and success-talk event scripts
and regenerate their source audit. The importer decodes the actual Blueprint
callbacks and saves the selected source functions with package hashes.
