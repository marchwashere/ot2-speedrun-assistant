# Harbor tag and return to Flats

`src/harbor.rs` continues a reachable daytime Flats entry through an on-foot
visit to New Delsta Harbor (`Fld_Cty_1_1`, LevelTable 44), immediately followed
by a return to Flats (`Fld_Cty_1_2`, LevelTable 45). Both transitions use the
shared world loader with the carried progression, NPC state and CRT stream.
The predictor is omitted when even the measured full walk cannot reach harbor
without an encounter. It assumes no intervening encounter, ship trip, path
action, menu or time change.

For seed 0 / title 4942 / shared level retained and the selected five-cycle
Highroad Escape plan, the source-derived prediction is:

| Boundary | Advances | CRT state | Encounter counter |
| --- | ---: | --- | ---: |
| Initial Flats entry | 6026 | `0xF0F1E302` | 717 |
| Harbor loaded and tagged | 6039 | `0x60917BF9` | See exported world state |
| Returned to Flats | 6053 | `0x04A0B097` | 515 |

This endpoint awaits live verification. The evaluated harbor load consumes 13
calls: outgoing counter B/A, five streamed levels, two gondola BeginPlay draws,
two eligible NPC animation frames, and incoming counter B/A. The return consumes
14: outgoing B/A, six levels, four eligible NPC frames, and incoming B/A. These
counts are outputs of the loader, not runtime offsets. For example, flag 6717
enables nine additional daytime harbor NPC placements. Shared `Btl_SinExpose`
remains resident, but the common `Btl_Fld_Cty_Land_A` package is recreated on
each map transition.

`BPC_MJLevelManager.Proc_StartChangeLevel` invokes `SaveVisitedMap_Impl` for
ordinary map transitions. It sets the visited/map-icon/mask bits without a CRT
lottery. The harbor's WorldMap row permits fast travel with no temporary ban.
The first-town fast-travel tutorial requires `IsNowLevelTown`; harbor has
LevelKind 1 (field), so it is ineligible. The new `Loader::visit_map` records
ordinary visits without changing scripted event-load visit gates or skipping
map initialization on a repeat visit. Its visit set records observed route
transitions, not an inferred complete prior save history.

Safe approach movement consumes no CRT calls. Its remaining counter is
overwritten by the outgoing map reset before dependent load callbacks. Returned
Flats receives a fresh first-encounter counter. The two imported harbor gondolas
are taken in Level actor-array order; cooked construction-script stream RNG is
not counted as CRT runtime initialization.

Reproduce the source audit with `node tools/audit-harbor.cjs`; its package hashes,
Blueprint inheritance/functions, LevelTable and WorldMap rows are recorded in
`data/source/harbor-audit.json`. `tools/import-town-loading.cjs` imports the harbor
through the existing data pipeline. To replay a selected route export, run
`cargo run --offline --example harbor_trace -- <route-export.json>`.

Manipulation shows only the return advance count, hex state and counter in the
Harbor → Flats card. Detailed log and JSON export retain both map-load traces,
boundaries and world states as `post_battle_field.harbor_return`.
