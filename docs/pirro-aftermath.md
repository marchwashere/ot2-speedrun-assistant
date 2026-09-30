# Pirro victory to Eastern New Delsta Highroad

Validated against the 2026-09-29 retail replay below. Assumes the
fresh solo Throné route, prompt scene skipping, and no intervening actions.

For seed 0, title advances 4942, shared-level recreation off, and the current
boost plan, Pirro's after-drops boundary is 5900 / `0x533B5D0C`.
The ending predicts **5962 / `0x6E0026C2`**, after control returns on
`Fld_Cty_1_3` (Eastern New Delsta Highroad), with encounter counter 862.
This includes the corrected R10 Pierce Through hit, which reaches the
reported command menu at5892; the previous forced-miss prediction was wrong.

| Stage | CRT draws | Seed position afterward |
| --- | ---: | ---: |
| Return from boss and finish mansion scene 1900 | 4 | 5904 |
| Load backstreets for scene 2000 | 36 | 5940 |
| Ending commands, load highroad, FinishPre | 22 | 5962 |

The 62 calls comprise 12 newly created streamed levels, 34 eligible NPC
initial-frame draws (27 backstreets, 7 highroad), seven counter B/A pairs,
and two canoe initialization draws.
These are outputs of the stateful loader, not a fixed runtime offset.

The earlier 5959 prediction omitted Pirro's `DeadEffectSlow` timer (+1 inside
the core) and two `BP_EnvObjMD_Canoe_A.ReceiveBeginPlay` calls (+2 in the loader).
Each canoe calls RandomFloatInRange(0,20) to set its bobbing timeline position.
`import-town-loading.cjs` reads the actual Level actor array, preserving its
order and owning streamed level. This ending executes them on level creation.
The later Highroad Escape capture also verifies both callbacks on field
restoration after battle; level residency does not suppress them. Both paths
share the environment initializer. No animation is executed. Static meshes
without this Blueprint do not draw.
See `data/source/canoe-initialization.json` and `live-pirro-aftermath-5900.json`.

The last normal task in `MS_TOU_00` completes the chapter as 2000 starts,
before its destination load. The loader removes Throné's active chapter and
prologue gates. Event 2000 sets Dawn, not Day. The native Night-to-Dawn path
refreshes old-phase placements and ages timers by one; it does not trigger
the Black Market lottery. The retained shared level is not recreated.

`tools/audit-pirro-aftermath.cjs` records scripts, sequence keys, native
task-successor/completion and time callbacks, plus asset hashes. The battle
return uses the same continuation mechanism as the captured second-fight
return. The complete replay now matches all 305 battle calls and 62 ending
calls by state and caller category. The two canoe identities also match in
source actor-array order. Other seeds/progression states remain source-derived.

Reproduce from an exported route JSON:

```text
cargo run --offline --example pirro_aftermath -- <route-export.json>
```

`src/pirro_aftermath.rs` accepts the actual after-drops state and carried
`world_at_entry`. Its `after_battle` adapter continues completed victories only.
The GUI/API exports the result as `post_battle_field.pirro_aftermath`, after
the final boost plan has been selected or replayed. All counts retain the
route's origin; the viewer adds the initial-seed offset once.

Manipulation shows the endpoint below Pirro's boost and latent instructions,
with advances primary and hex secondary. Detailed log includes the endpoint
and complete loader trace; JSON also carries loader decisions and world state.
Defeat/incomplete routines have no ending prediction. Unsupported ending
states show an error without discarding the battle result.
