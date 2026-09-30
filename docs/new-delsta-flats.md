# New Delsta Flats arrival and harbor approach

## Live Escape verification

The 2026-09-29 live capture verifies every CRT call from menu **6002** to field
control at **6012**: five boost voice draws, one Escape check, two gondola
BeginPlay draws, then counter B/A. The missing calls were the two resident
field gondolas initializing their bobbing time again when the field returned.
They are not new streamed-level constructor calls or another counter reset.
`Loader::restore_field_after_battle` evaluates the imported field actors before
resetting the counter. See `data/source/live-highroad-escape-6002.json` and its
reproducible importer, `tools/import-highroad-escape-live.cjs`.

The map loader now imports `Fld_Cty_1_2` (LevelTable ID 45). It evaluates its
streamed packages and unfiltered placements using the carried world state;
there is no fixed map-load offset. Run `examples/flats_trace.rs` with a GUI
route export to continue its selected successful Highroad Escape. This assumes
daytime, no intervening CRT actions, and no second Highroad encounter.

The GUI/API now carries `highroad.field_return` and `post_battle_field.flats`
using `highroad::return_to_field` and `flats::after_highroad`. Manipulation
shows the returned-field CRT beneath the Escape inputs, then the Flats entry
CRT in the next card. Detailed log and export preserve both traces/world states.

For seed 0 / title 4942 / shared level retained, with the selected five Highroad
0→1→0 cycles: Escape check ends at 6008, gondolas end at 6010, and field-return
counter B/A ends at **6012 / 0x2E4773FC**, counter **1448**. The Flats load ends
at **6026 / 0xF0F1E302**, counter **717**. The load
evaluates two outgoing counter draws, six new levels, four eligible NPC frame
draws, and two incoming counter draws. The Flats endpoint awaits live verification.

The user's separate harbor route measurements are **61 walking updates** and
**56 running updates**. Daytime Flats costs 9 per walking update and truncates
9 × 1.4 to 12 per running update. Costs are 549 walking and 672 running; retain
a positive counter at the exit, requiring 550 or 673 respectively. For the
predicted 717, walking leaves 168; all 56 running updates are safe, leaving 45.
`flats::harbor_reach` owns both measured budgets and exposes them in GUI/JSON;
`flats_trace` uses those same budgets for its probability calculation.

`EncountManagerBP.UpdatePlayerStep` compares displacement against
`PC_ONCE_STEP_LENGTH`, calls PlayerStepCounter once, then assigns the current
position to PrePlayerPos. It discards the excess rather than carrying it into
the next step. Faster movement therefore can traverse the same path with fewer
decrements. These full-route observations do not establish exact mixed movement
counts, and their ratio must not be applied to other routes. The opening's
running/mixed display is now explicitly an estimate from its walking count;
the mansion retains its independent measured 161 running updates.

The Highroad search now replays Escape, field restoration and the Flats load
for candidates, requiring at least550 for the whole walk. Its independent
additional-repeat setting is grouped under Highroad, the last controllable
event. A capped failure keeps the successful Escape and clearly reports that
the Flats walking target was not met.

For mixed movement let f be the fraction of **route distance** run. We estimate
counter cost from the separate measured full-route densities:
`61*9*(1-f) + 56*12*f = 549 + 123*f`. The displayed percentage is
`min(100, floor(1000*(counter-550)/123)/10)` when counter>=550; otherwise no
mixed plan is offered. Round cost up and reserve at least one point. This
interpolation accounts for the different update counts and per-update costs,
but does not reproduce sampling phase at the speed switch. Intermediate
percentages are explicitly labeled estimates, not exact guarantees. Counter673
or above permits100% running by the measured full-run budget. Counter600 gives
an estimated40.6% run /59.4% walk, costing599 and leaving1.

The first counter is floor((1000 + A + B) / 2), A/B each selected in 0..399.
`flats_trace` counts the actual correlated consecutive CRT draws over the full
state space (bit-31 symmetry reduces this to 2^31 cases, counted by intervals).
Safe walking: 2079690960 / 2147483648 = 96.8431569636%.
Safe running: 1341753310 / 2147483648 = 62.4802573584%.
These are uniform-state odds, not uncertainty about a fixed supplied state.
The interval-counting implementation was previously cross-checked with an
independent Euclidean floor-sum calculation at thresholds 505 and 673, with
boundary checks that required counters 500/900 yield 100%/0%. The revised
walking count above uses the new measured threshold 550.

The route now continues through an immediate harbor tag and return; see
[harbor-return.md](harbor-return.md). Both maps reload through the shared loader,
and returned Flats receives a fresh first-encounter counter.
