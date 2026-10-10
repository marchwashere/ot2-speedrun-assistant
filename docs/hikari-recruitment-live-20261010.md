# Hikari recruitment: map-local placement story filters

Retail replay starts at CRT state `0x3903DEB2`, relative advance 1141 from
identified state `0x5ACB5737` (Pirro first on R3, Throné 154 HP, 29 observations
`NSNNSNNSNSSSNSSSNNNSNSSSNSNNN`). The capture contains 199 consecutive calls,
ending at Canalbrine Bridge's first command menu. Debug registers were restored.

The battle ends at +1235 / `0xA8534BD8`, as predicted. Recruitment refreshes
five NPCs, not four. The omitted placement is Ryu townsman
`NPC_Twn_Dst_1_1_A_TALK_0300_D000`, whose MainScenarioType is -1.
Its identity is derived from placement data and the native gate; the capture
establishes five spawn-frame calls but does not record individual spawn labels.

Native `ec4900` calls `f4c660`. That calls `ece1c0` for the current map and
`f49fa0(map, false)`, which selects currently running stories whose related
map list contains that map. `f2b480` resolves the first flag-enabled
SwitchWMapLocation entry, falling back to RelationLevel if its list is empty.
A negative MainScenarioType means this **map-local** story list must be empty.
A positive value must occur in that list. The old implementation queried the
global story set and incorrectly let Throné's active Oresrush chapter suppress
an unrelated Ryu NPC.

The shared placement gate now uses imported MainStory relation lists and flag
switches. No compensating draw offset is used. The replay regression verifies:

- Hikari battle end: +1235.
- Recruitment complete: +1244, including five newly spawned NPCs.
- Canalbrine Bridge load: +1274.
- Water encounter: Blue Betta ×3, normal opening.
- First water command menu: +1340, state 1561477059, matching capture end.

Each post-battle checkpoint is checked against the actual recorded CRT stream.
This does not claim every internal battle animation/menu timing boundary has
been independently validated; the battle endpoint and subsequent field/menu
checkpoints above are the verified boundaries.
