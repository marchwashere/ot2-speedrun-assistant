# Route regression repair

The duplicated test-only Eastern eager implementation omitted apply_escape_resources:
it neither initialized an absent ledger nor carried break latent/SP changes. Production
passage() already did both. The nested Snows uncached oracle calls this implementation,
so its 25-point latent discrepancy was inherited from the same omission. The eager
oracle now uses the shared resource transition and the same enabled spell candidate
set, while keeping independent candidate enumeration for search comparisons.

The early Highroad test treated escape_max_boost_repeats=None as excluding breaks.
That is an unlimited Escape budget, not a ban on breaking; the independent required
break setting remains authoritative. The test now accepts a valid optional break and separately enables the required-break setting to assert that it stays. The second Highroad replay test now replays the selected break strategy, rather than treating its post-break boost sequence as a first-menu Escape.
This is the first Highroad after Pirro, not the new Inventor Highroad.

A Winterbloom onward test reached Forest Octopuff Traveler (ENE_NML_WTR_570), which
was present in encounter selection but absent from field initialization/combat imports.
Its real EnemyDB speed/BP/weaknesses/initialization are now imported additively. No
unsupported enemy AI is invented; this is first-menu Escape support.

The Ryu walking regression fixed seed69 as a rejected candidate. Current shared map
and story replay gives it a safe counter, invalidating that test setup. The test now
finds an actually unsafe successful Escape under current source callbacks, verifies it
is rejected, and verifies the accepted replacement preserves both downstream walks.

Validation: all six originally reported failures and the extra Highroad replay test pass. Final library run130 passed, zero failed; the remaining legacy flats Cait-fork exhaustive search was stopped after its long-running notice. Release GUI rebuilt/relaunched; saved-route API still matches first menu19287621 and both observed orders.
