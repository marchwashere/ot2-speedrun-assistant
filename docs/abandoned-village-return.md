# Abandoned Village return → Southern Cape Cold Snows

Continue the selected Flats encounter through Escape and the remaining run to
the village. `village_return.rs` then visits `Twn_Cty_1_2_A` and immediately
returns to `Fld_Cty_1_2` using the shared `world::Loader`. Both loads retain the
real progression, NPC gates, streamed-level residency, flag lotteries and B/A
counter draws. The fresh return counter is **first encounter**, 500..899;
Escape's previous reset was the normal 1000..1798 counter. Never carry the
residual village-approach counter into this walk or add a fixed RNG offset.

The village was already included in the imported town catalog. The reproducible
`tools/audit-abandoned-village.cjs` scans its packages and blueprint parents;
`data/source/abandoned-village-audit.json` preserves evidence/hashes. No new
BeginPlay CRT callback was found. Light beams use a separate random stream;
the grass flipbook's random calls are cooked construction-only. The village's
bar event is None, so no bench/tavern roll. A first town visit after the finished
chapter qualifies for tutorial 62; dismissing it changes tutorial flags without
another CRT or encounter-counter draw. No path action, time change or story
selection is included. This new segment is source-derived, awaiting live trace.

Measured route costs are 70 walking updates or 66 running. Add the user's
two-update safety allowance to both: **72 walking / 68 running**. Flats daytime
costs are 9/12, so walking costs 648 and running costs 816. A strictly positive
counter is required: minimum 649/817. The route ends at the Southern Cape Cold
Snows entrance **before loading that zone**.

For running distance fraction f, estimate cost as `648*(1-f)+816*f`. Reserve
one point, round running share down to 0.1%, and counter consumption up. This
uses both speed-specific measurements to account for observed overshoot, but
does not simulate the phase of individual walking/running switches. The
displayed running distance percentage is therefore an estimate.

Settings puts `flats_snow_max_boost_repeats` in **New Delsta Flats encounter**:
default Required outcome (`None`, query `required`; legacy `unlimited` accepted).
Required candidates must Escape, run safely to village, replay both map loads,
and obtain counter >=649. This predicate applies equally to boost-only and
immediate-break strategies. A finite optional cap is additional repeats beyond
the actual Escape/village-run plan; failure preserves that earlier plan and
reports the unsafe walk. Do not change earlier Highroad/Pirro searches to solve
this target. Exports include the complete `flats_escape.village_return` and
`snow_search` records, with source trace/checkpoints in Detailed log.

Unadjusted fresh-counter odds over uniform 32-bit CRT states (consecutive B/A
draw correlation retained), reproduced by `cargo run --offline --release
--example village_snow_odds`:

| Route budget | Required counter | No encounter |
| --- | ---: | ---: |
| Measured 70-update walk | 631 | 78.4661458805% |
| Safety-padded 72-update walk | 649 | 72.1550324466% |
| Safety-padded 68-update run | 817 | 8.5601584520% |

These are baseline reset odds, not probabilities conditioned on the complete
route or selected boost search. A selected valid plan deterministically meets
the modeled walking budget.
