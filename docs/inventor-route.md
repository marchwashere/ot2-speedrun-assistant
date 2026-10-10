# Canalbrine to Arkar

After benched Castti recruitment, visit Twn_Cty_1_1_A, accept Agnea Chapter 2
via MapInStorySelect (MS_ODO_01), skip MS_ODO_20_0100, then leave for Fld_Cty_1_3.
The shared forced story-load path carries scenario801, view flag35510, flags6550
and34167, daytime command1550, NPC refresh and FinishPre counter reset.
8031 only enables existing EventFlagObjects; command70 writes its persistent flag.
Source: data/source/agnea-new-delsta-story.json. Retail verification pending.
Select one daytime land encounter using current main progression, first-encounter
state, Evasive Maneuvers, actual active levels and enemy history. Bench members
are excluded. Reject surprise at the preceding Conning control before accepting
the route; the successful Pomegranate steal remains mandatory. This extends
the local search, without revisiting earlier fights. Escape boosts use the
actual first player menu and the shared carried-party adapter. Flee before
enemies act; unsuccessful attempts are search candidates, never route actions.

For a full party the surprise probability is
`5 + 4 * clamp(enemy_average - party_average, -1, 6)` percent (integer averages).
The minimum is1%. Earlier protections do not make its unmanipulated chance zero.

Arkar first interaction runs SYS_INV_GUILD_0010: Proof of the Inventor,
initial abilities, tutorial59, guild registration, FinishPre reset and sidequest
start. Then SYS_INV_GUILD_0D10 crafts Elemental Bomb Bottle, followed by
SYS_INV_GUILD_0C10 for Critical Scope. EventManagerBP.UpdateInventorFrow handles
Yes, consumes recipe materials and sets EventIndex from opt[2]; the learning
callback unlocks the ability. Both events have EncountReset=true and execute
their FinishPre callbacks. Only two inventions are crafted, so the all-inventions
completion branch is not taken. Flags2411/2410 record the two crafts; license11
and unlocked ability sets are retained in world state and cache keys.

Source records: data/source/inventor-audit.json contains EventList rows, complete
event scripts, recipe table, guild/substory rows, and dialog/learning Blueprint
functions. Core tools/import-highroad-return.cjs imports all relevant progression
formations and enemy initialization/weaknesses without replacing existing spell
or voice data. No enemy AI runs in this first-menu Escape route.

Assumptions: daytime, one Highroad encounter before reaching Arkar, required
recipe materials and sufficient travel funds already available. General field
inventory and sidequest journal tracking are not implemented; invention flags,
license/ability ownership and carried character progression are tracked. New
endpoints await retail verification.
