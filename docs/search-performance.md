# Route search performance

Release profiling, 2026-10-02, seed0/title4997/shared retained, active Osvald
encounter route and the current required targets. OT2_PROFILE=1 enables
thread-local inclusive timers; normal operation does not collect timings.
Manual reproducible test:

    $env:OT2_PROFILE='1'
    $env:OT2_PROFILE_OUTPUT='outputs/profile-after.json'
    $env:RUST_MIN_STACK='33554432'
    cargo test --release --bin ot2-gui profile_current_route -- --ignored --nocapture

Measured request: 3.893s -> 2.904s (25.4% shorter). Formation optimizer remains
the dominant stage: 1,031 Flats continuations, 634 Southern route searches.
Inclusive timings overlap and must not be summed. These results describe this
configuration on this computer, not all seeds or a guaranteed response time.

Removed redundant work:

- Pure shortest-button-path results are reused for draw counts0..256, keyed
  by extra draws, available BP (clamped to3), and final BP. The bounded cache
  stores successful paths only; larger counts use the unchanged planner. This
  is a memory bound, not a Required-outcome cap. Planner runs536,244 ->835,
  inclusive time0.679s ->0.034s.
- Direct boost/cancel and zero-BP break/R2-cancel paths have even input costs
  (break costs10). Skip impossible odd layers before Eastern initialization
  and zero-BP candidate expansion. Eastern layer calls51,088 ->26,459;
  candidate calls108,211 ->84,142.
- Existing character ledger entries no longer eagerly construct a discarded
  starter character on every stat sync. Equipment/resources/JP remain carried.

Full serialized response snapshots before/after are byte-identical, SHA256
A872269D7CEF57E742BB22012F5569853AFCF8E24CAF57E9D962C7B063636292.
This includes both timing alternatives, Western override endpoints, chosen
plans, search counters, history, EXP/levels and every recorded CRT checkpoint.
88 library tests pass; cached button paths separately match uncached results
including256/257/300 draws. The long existing uncached-route oracle was not
rerun in this check; the current complete response provides an additional
before/after regression fixture.

Still expensive: real route lookahead, map loading (61,989 calls), battle
candidate replay and carrying both player overrides. No arbitrary retry cap,
formation whitelist, disabled outcome or altered candidate ordering was added.

## Second optimization pass

Implemented incremental direct boost/cancel prefixes and post-break R2 boost
prefixes. Cursors include voice history and the full CRT trace. Escape rolls
always run on clones, so failed rolls never contaminate the next candidate.
Baseline Escape results are retained, and earlier/nonsequential cost requests
reset the prefix safely. Eastern battle initialization and break caches now
survive cost layers. Ordered visitors stop generating later siblings after a
valid continuation is accepted.

Formation lookahead omits Loader trace/decision allocation, then replays the
selected continuation with its original cost bound for the complete log and
search metadata. This does not disable RNG callbacks or battle event recording.
When clerics are required throughout a search, an impossible walk or failed
night lottery rejects the candidate before day/recruitment/encounter work.
Optional goals retain their previous full projection behavior.

Exact typed state keys replace JSON serialization in formation deduplication.
The Flats projection key now includes enemy weakness/Study Foe history as well
as CRT state/draws and movement choice. The prior key omitted history; no
incorrect current-route result was demonstrated, but the regression explicitly
checks that identical CRT endpoints with different history remain distinct.
Keys exhaustively destructure world/character state, so adding a field requires
reviewing its cache identity.

A bounded full-map-result cache was tested and removed. It hit 21,403 of 54,931
loads, but cloning exact keys/results cost more than replaying the loader:
current-route request 2.373/2.416s with it versus 1.978/1.939s without it.
Neither version changed the complete result. Do not reintroduce it based only
on hit rate.

Rebuilt live-server complete response comparisons, seed0/shared retained:

| Title advances | Previous response | Updated response | Complete JSON |
| --- | --- | --- | --- |
| 4997 | 2.898s | 2.180s | byte-identical |
| 4916 | 7.139s | 4.235s | byte-identical |
| 5000 | 1.151s | 0.578s | byte-identical |

These are local single-request measurements (cold-start/order affects timings),
not a guarantee across all inputs. Snapshots are in outputs/perf-baseline-*.json
and outputs/perf-live-*.json. Both timing alternatives, selected actions, history,
levels, search metadata and every diagnostic checkpoint are compared together.
The manual GUI profile test accepts OT2_PROFILE_QUERY for alternate fixtures.

Validation for this pass: 97 assistant library tests and 20 core unit/field
tests pass. New tests compare fresh versus incremental boost replays including
voice/weakness history, failed Escape rolls and out-of-order costs; eager versus
streamed complete continuations; both Osvald timings; and 192 required-cleric
projections. The legacy unbounded whole-route oracle was not rerun; bounded
successful oracle fixtures and the three complete response snapshots were used.
