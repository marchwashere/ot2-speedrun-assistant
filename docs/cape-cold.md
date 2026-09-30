# Osvald to Cape Cold

After recruiting Osvald, declining his introductory story and dismissing the
tutorials, continue on foot through daytime Southern Cape Cold Snows into
Cape Cold (`Twn_Snw_1_2_A`, map169). `src/cape_cold.rs` carries the final party
and story flags from `osvald::Prediction`; no reseeding or fixed draw offset.

The user measured52 walking or49 running counter updates and requested two
updates of leeway for each. Costs therefore are **54*9=486** walking and
**51*12=612** running. Strictly positive arrival thresholds are487 and613.
Even the minimum first-encounter reset500 permits the padded walking route.
No new boost-search setting is needed; preserve the previously chosen plan.

Mixed movement estimates distance share using cost(f)=486+126*f. Reserve one
counter point, round the running fraction down to0.1%, and cost up. Independent
speed measurements account for whole-route overshoot, but interpolation does
not simulate the exact speed-switch location or update phase. Label it as an
estimate. At counter552, run approximately51.5% and walk48.5% (estimated cost551).

Movement advances no CRT. Keep the walking reference remainder in carried
world state, then use the existing `Loader::visit_map` for the destination.
The outgoing reset replaces that remainder before any counter-dependent work,
so safe mixed movement has the same map-entry RNG. If walking is unsafe,
report the failed budget without inventing a map endpoint.

The local asset audit imports five case-insensitively resolved streamed
packages and all29 Cape Cold placement candidates. For this progression,
13 NPCs spawn. Together with outgoing/incoming B/A resets, the evaluated total
is22 CRT calls. The shared battle level stays resident. BarEventLabel=None;
the bartender is an ordinary NPC frame draw, not a reserve-party lottery.
Actor/parent Blueprint audit finds construction-only flipbook randomness and
an explicit random stream, but no extra CRT BeginPlay call.

For seed0/title4942 with current required targets and one Flats boost cycle:
Osvald6132 / counter552 -> Cape Cold6154 / `0xA3471B82`. This is source-derived
and **awaits live validation**. Source evidence is reproducible with
`node tools/audit-capecold.cjs`; runtime data uses
`node tools/import-town-loading.cjs`.

The GUI adds an Osvald → Cape Cold card containing the running share and the
map-entry advance count/hex state. Detailed log and exports include full trace,
movement budgets and carried state. The stage runs once after selecting the
Flats manipulation; it does not enlarge or rerun that search.
