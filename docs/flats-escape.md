# Harbor return to Abandoned Village

`src/flats_escape.rs` starts at the returned-Flats state from `harbor.rs`.
The user's full running distance is about 165 counter updates; the route uses
their requested fixed **170-update** allowance. It is not a Settings field.
Daytime running costs 12 points per update. First encounter update is
`ceil(entry_counter / 12)`. After Escape, require strictly more than
`(170 - first_encounter_update) * 12` points. The first trigger's overshoot is
discarded; do not subtract it again from the fresh post-Escape counter.

Highroad has an independent optional repeat allowance for avoiding surprise in
this Flats encounter, default four, plus a **Required outcome** toggle. Required
Flats non-surprise jointly searches Escape, safe Harbor walking and the returned
Flats opening before optional improvements. All candidates replay field
restoration, both Flats loads, Harbor and actual encounter selection. An optional
miss retains the required plan. Do not spend Pirro's budget or invoke downstream
Escape searches inside these trials. See [search policies](route-searches.md).

The Flats encounter has its own default-four break preference and **Boost only**
choice. Reuse the
battle crate's `FieldEscapeMenu` and actual `first_action_break` simulation;
Flats' daytime progress-one table is the already imported Highroad table.
Search strategies in this order:

1. Ascending R1 `0→1→0` repeats within the limit, accepting only successful
   Escape **and** a reset counter sufficient for the remaining run.
2. An immediate hit-verified single-enemy break followed by guaranteed R2
   Escape, also requiring a sufficient reset counter.
3. If neither meets both targets, continue the direct boost search without a
   repeat limit. Choosing Boost only goes directly to the boost search.

Candidates always clone the untouched command menu. Never execute a rejected
Escape check or break before the selected strategy. Field restoration uses the
same loader callbacks as Highroad, with Flats' own environment data and a normal
(`first_encounter=false`) counter reset. A surprise opening yields an explicit
unavailable result until the upstream target succeeds; surprise combat is not
approximated. Existing HP/equipment assumptions remain: solo route-level Throné,
unchanged equipment, no damage since the post-Pirro level-up. Supported immediate
break fallbacks take no enemy actions and award no EXP.

The original Escape/run endpoint remains the Abandoned Village **entrance before loading/tagging it**. The separate `village_return` stage now continues through the tag and return, and its walking target shares this encounter's boost search; see [Abandoned Village return](abandoned-village-return.md).
Its CRT state equals field return because the remaining safe movement draws
no CRT; its world counter has the remaining running cost deducted. No new map
load or village tag is invented. Manipulation shows only the selected inputs,
remaining run and field-return CRT. Details, encounter and all callback spans
are available in Detailed log/export (`post_battle_field.flats_escape`).

The seed0/title4942 retained-shared route now needs seven Highroad cycles
(two beyond the old five-cycle Escape/harbor plan); the old plan's returned
Flats state at6053 produces Surprise. The selected plan returns from harbor
at6055, counter609; first encounter occurs on update51, leaving119 updates.
One Flats boost cycle then Escape gives counter1503, leaving75 at the village
entrance. These new encounter/escape predictions are source-derived pending
live verification, not additions to the earlier live Highroad fixture.

## Baseline second-encounter estimate

Run `cargo run --offline --release --example flats_second_encounter_odds`.
The first counter is `floor((1000+A+B)/2)` (500..899), and the post-Escape
counter is `1000+C+D` (1000..1798). For each possible first trigger update n,
weight the probability that the next reset is at most `12*(distance-n)`.
The tool counts each pair using the actual CRT mapping over all 2^31 distinct
rand-output states, preserving consecutive B/A correlation within each pair.
The two reset starting states are treated as independently uniform; this is a
distance-only baseline, not route-conditioned odds or failed-Escape combat.

| Whole route | Second-encounter estimate |
| --- | ---: |
| 165 running updates | 25.9763% |
| 170 running updates | 37.0678% |

For a fixed counter515, the same random-reset model gives76.4524%, since the
first encounter leaves127 updates and requires a reset of at least1525. This
is not uncertainty about a supplied complete CRT state. The actual route and
boost search use serial CRT replay, not these probabilities or independent draws.
