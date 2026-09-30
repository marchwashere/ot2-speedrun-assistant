# Cape Cold night switch and Mug

`cape_cold_mug::predict` continues daytime Cape Cold with party[7,6] through
the user's first Mug of NPC_Twn_Snw_1_2_A_TALK_1100. Original-gear Lv1 Osvald;
English/2x. It imports the standalone core's fixed Mug routine.

For confirmed seed0 +6154 (0xA3471B82):

| Boundary | Advances | CRT |
|---|---:|---|
| Night switch and NPC placements |6163|0x3E984E25|
| Mug preview opened |6164|0xA38E8854|
| Opening-line lottery / before battle initialization |6165|0x8AAD55C7|
| R1 command menu |6192|0x3C165BB0|
| R2 command menu |6218|0x0F2A97C2|
| R3 command menu |6289|0x2A7DE193|
| Battle victory and drop |6295|0x0D1058F9|
| Loot/success dialogue dismissed, field control |6299|0xDC86178D|

Replay2 validates all38 calls from daytime arrival6154 through first menu6192.
Mug NPC initialization consumes BP + timing, without the ordinary enemy sprite
frame draw previously assumed. Study Foe and19 turn-start voice checks match.
Later boundaries remain source-derived: Icewind53 + Fireball226 leaves1 HP;
R3 staff deals11. Compact callsite fixture and source hash are retained in
`data/source/cape-cold-mug-first-menu-live.json`.

The shared loader evaluates the night change, including one black-market flag
lottery and eight new nighttime NPC frames. Preview consumes its normal timer
draw; Rob.UpdateFlow separately calls DrawEventLabel for the50/50 opening
line. Both FC_GAK_ROB_Before events and FC_GAK_ROB_Success have EncountReset=false
and contain only fixed talk/flag branches/end. The battle sublevel is already
resident. Ordinary battle field restoration evaluates its environment callbacks
and counter B/A, without an invented additional reset for the loot dialog.

Reproduce using `cargo run --offline --example cape_cold_mug_trace -- <route-export.json>`.
The GUI continues the selected route with `post_battle_field.cape_cold_mug`.
Detailed log shows damage, command-menu checkpoints, the final field CRT and
the chronological night/menu/battle/return trace. Manipulation only shows the
R3 staff finisher when needed (or a prediction error). Export JSON includes
the full result. No endpoint is generated if Cape Cold is not reached.
The captured source audit and fixture are
under data/source/capecold-mug-*.json; distinguish live capture from model output.

Initialization is now resource-derived, not a Mug exception: the initial
BAT_IDLE_ENEMY slot is absent, so RandomInteger(0) consumes no CRT draw.
See the core docs/enemy-initialization.md for the complete branch and scope.

## Retail completion correction

The completion replay confirms menus6192/6218/6289 and field control6299
(0xDC86178D). Prior6297 endpoint omitted kill voice at6293 and the break
slow timer at6294. Osvald normal attack voice lasts5689 ticks; damage occurs
at9600, so the kill reaction is eligible. JustBreak remains latched even when
the same hit kills and death clears the persistent break state. Drop is at6296,
battle exit6297; field reset draws6297/6298. Loot dismissal adds no extra reset.
The compact completion fixture retains all134 calls from battle initialization
to field return, plus the raw capture hash. This supersedes earlier source-only
completion values above.
