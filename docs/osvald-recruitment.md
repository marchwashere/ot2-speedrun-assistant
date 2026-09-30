# Osvald recruitment

Continue the returned-Flats stage through daytime Southern Cape Cold Snows
(`Fld_Snw_1_2`, map 152). Walk **70 counter updates**, as measured by the user,
to Osvald, help him, accept recruitment, decline playing his introductory
story, skip the remaining scene and dismiss its tutorials. The user confirmed
this branch on 2026-09-29. No extra time switch, path-action menu or encounter
is included.

`src/osvald.rs` retains the incoming world state and calls the shared loader.
For the fresh route, loading evaluates seven new streamed levels, three eligible
NPC starting frames and the outgoing/incoming B/A counter pairs: 14 CRT calls.
That total is an evaluated result, not an offset. The ordinary Al NPC is
disabled by carried story flags. A changed placement gate changes the draws.
Map actor/parent audit found only baked construction randomness and separate
random streams, with no additional CRT BeginPlay draw.

The 2026-09-29 correction includes `Fld_Snw_1_2_sound`. The importer previously
looked for `_Sound` with a case-sensitive JavaScript Map and silently skipped
this lowercase asset, despite Windows/UE resolving it. Package lookup now folds
case and preserves the actual cooked filename. Read-only live memory shows all
seven levels and the predicted three NPCs. Native `ec60c0 -> 11c0100` supplies
the omitted level-color draw before incoming counter B/A. No audio simulation
or arbitrary endpoint offset is added. The same import fix also finds the mixed
case collision package `Twn_cty_1_2_B_Coll` in an imported off-route map.

Walking uses the encounter catalog's daytime decrement (9), requiring counter
**631 or higher** to survive 70 updates with a positive remainder. There is no
running measurement for this segment, so no invented running percentage.
The Snows counter is a new first-encounter counter, independent of the remaining
counter on the preceding Flats walk. Unsafe approaches expose the entry state
and failed budget, but no hypothetical recruitment endpoint.

## Recruitment source path

`MS_GAK_00_0100` is an ADDMEMBER event on the same map with EncountReset=true.
Command 5010 at index15 selects Yes, then 3010 at61 selects Yes. Command3000
joins character6. The 3120 prologue prompt at70 selects No, continuing at76
with tutorial66. Command3005 finds Osvald already in the main party; the
EndPrologueReminiscence native call returns before work because flag2900 is off.
Command7410 increments persistent event variable1; 7420 compares it to8.
The fresh route takes the unequal branch through110 to131.

131..143 set recruitment/skip-prologue flags. FinishPre at144 refreshes changed
NPC placements (removing Osvald's field placements, spawning none on this route)
and rolls counter B/A. Tutorial71 follows at146. Fixed dialogue, join confirmation
and both tutorial popups have no separate CRT lottery or counter reset.
Recruitment therefore adds **two CRT draws** for this path; no level is reloaded.
The resulting party is Throné and Osvald, with progression and tutorial flags
carried forward. Recruitment does not play his prologue or simulate graphics.

Reproduce evidence with `node tools/audit-osvald-recruit.cjs` and rebuild the
imported catalog with `node tools/import-town-loading.cjs`. Evidence includes
the exact event script, map actors/parents, recruitment Blueprints and native
party operations in `data/source/osvald-*.json`.

## Search and presentation

The control point is the preceding **Flats encounter**. The independent
`flats_osvald_max_boost_repeats` setting defaults to Required outcome. Required
Osvald walking also requires Escape, the remaining village run and the prior
Flats-to-Snows walk. All required targets gate both direct and break candidates
before optional budgets. A numeric limit is an optional allowance after the
selected preceding plan; failure keeps that plan, and never invents a reachable
Osvald endpoint. Successful earlier targets remain preserved.

Manipulation displays safe walking and post-recruitment advances/hex. Detailed
log includes entry, pre-recruitment, post-tutorial boundaries and every draw.
Exports preserve the setting, selected boosts, world states and decisions.

At seed0/title4942 with the user's required upstream settings and Flats Boost
only, the corrected combined plan uses **one** `0→1→0` cycle: returned Flats6116,
Snows6130 (counter727), Osvald6132 / `0x0DF53D34`. Walking70 leaves97 before
recruitment resets it. The omitted level had shifted the counter rolls, giving
the incorrect one-cycle counter627 and forcing six cycles. No distance or
padding change is needed. For the previously executed six-cycle route,
returned Flats6121 now reaches Snows6135, matching the user's reported endpoint.

Validation: source audit, exact branch/flag/counter checks, search minimality,
required-prerequisite and optional-cap tests, break-strategy gating, full route
and export replay. The six-cycle loading endpoint and resident level/NPC
inventory match retail; no per-call load capture was taken. **Recruitment still
awaits live validation.** See `data/source/reported-snows-entry-20260929.json`.
