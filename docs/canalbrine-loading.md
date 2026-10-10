# Ryu return and Canalbrine Bridge

This continuation starts at field control after Throné's Scholar assignment,
Lightning Bolt/Elemental Barrage purchases and Evasive Maneuvers equipment.
It follows the ordinary land transitions:

- `Twn_Dst_1_1_A_1` /1130: Ryu -> Northern Ryu Sands (Map19), arrival trigger03.
- `Fld_Dst_1_1_2` /1001: Northern Ryu Sands -> Canalbrine Bridge
  (`Fld_Sea_1_2`, Map107), arrival trigger04.

Both transition rows have zero gates and no conditional destination override.
The separate1003 transition goes by canoe and is not used. The bridge has no
forced time change, tavern callback, arrival EventList row or event-trigger
placement that requires an automatic tutorial/cutscene.

The user specifies no encounters on these legs. Walking itself consumes no
CRT. The route does not infer movement distance, safe running percentage or
counter consumption before the exits. The real map-load resets replace that
old counter. Tests vary the starting counter and show identical load RNG and
final state; level residency still changes constructor draws when appropriate.

`src/canalbrine.rs` invokes the existing `Loader.visit_map` twice. Its two
entries retain separate arrival/after CRT values, complete trace/decisions,
and world state. It carries party `[7,1,8,4]`, bench `[6]`, all EXP/JP, current
resources, gear, learned skills, support equipment, enemy knowledge, progression5
and Mother's active701. Neither map load awards EXP/JP or changes Scholar.

Northern Ryu is unloaded/recreated according to the normal shared loader;
being visited before does not remove its field-owned constructor/NPC draws.
The source tutorial20509 gate excludes the old side-story NPC/trigger.
The bridge includes seven streamed levels (base/Art/Coll/GD/Sound and two
battle levels),36 placement records, and the usual1000/400/400/0.5 encounter
counter parameters. The current daytime progression enables two Northern
NPCs and seven Bridge NPCs. Counts are derived from placement gates, not
stored as production RNG offsets.

Both checkpoints, fresh entry counters and every draw are shown in Detailed
log/export. Manipulation stays focused on actions the user must change; there
is no new diagnostic-only card. The two loads run after the selected Ryu
arrival's Hikari and Scholar continuation, not on rejected movement trials.

For the current seed0/4997-title-advance configuration, selecting before-Osvald
and the second Western Winterbloom encounter, Scholar setup ends at7104.
Northern return consumes12 source draws (six constructors, two NPC frames,
four counter draws):7116 advances / `0xC1A01BCC`, entry counter772.
Bridge load consumes18 (seven constructors, seven NPC frames, four counter
draws):7134 advances / `0x60F1BEB6`, entry counter836. These are diagnostic
results from replaying the full state, not stored offsets. The API comparison
preserves every upstream state, action, trace and selected search plan.

`tools/audit-canalbrine-bridge-loading.cjs` records497 placed Blueprint actors,
51 direct classes/63 including parents,145 source hashes and two empty embedded
functions. No additional runtime environmental CRT callback was found. Two
mesh-generation helpers use a private FRandomStream in construction; they do
not consume CRT. All1259 original catalog entries were preserved, and the
import reproduces byte-identical files. Evidence is in
`data/source/canalbrine-bridge-loading-audit.json`. The new checkpoints are
source predictions and await retail validation.
