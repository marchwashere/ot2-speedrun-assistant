# Forward route policy (2026-10-06)

The active GUI no longer calls the cross-formation optimizer described below.
Pirro R8 selects the first safe Throné-breakable Highroad opening; BP is allowed.
Freeze R8. Highroad executes a break and selects boosts satisfying safe Harbor
walking and a safe Throné-breakable Flats opening. Freeze Highroad. Flats executes
a break and solves its local village/cleric/Snows approach outcomes. Freeze Flats
before searching the Snows continuation. Later cost cannot reopen these choices.
A naturally safe Southern rare remains usable, but no upstream retry is made to
beat the later Eastern/Western rare plan. Oresrush's separate policy is unchanged.

Break candidates use real attack hit rolls and next-menu BP. A 1-BP break may
leave no BP for R2 manipulation, so R1 boost/cancel inputs remain searchable.
Successful field breaks contribute 25 latent to their attacker, capped at 80
for this Throné route, with no extra CRT call. Projection cache keys include that
gain, and the rare item clear receives the carried player resources.

# Bounded search and exact replay

Snows trials accept a remaining weighted-input ceiling. A rejected Snows branch
still checks the Southern rare alternative within the remaining admissible cost.
Only the predicted Osvald timing is needed while comparing formations. The
selected detailed replay fills in the player's timing override alternatives.

Hikari/coast searches propagate the incumbent ceiling through each nested menu.
The first safe Northern opening remains the Western fallback, even if that
fallback exceeds the bound: pruning it must not replace it with another non-Cait
opening. Later Northern Cait candidates remain eligible, with Northern preferred
on equal total cost. A nearest-menu candidate whose first Western escape exceeds
the bound is rejected, rather than changing the nearest-menu policy.

Formation search starts with a 32-input frontier (or its existing baseline if
smaller), doubles as needed, and exhausts the winning frontier with the normal
incumbent pruning. Negative deduplication is cleared when the frontier increases.
Oresrush similarly starts at 32, doubling without an arbitrary Required cap.
It caches Mother-route entry and recruitment per before-switch count. The final
winner is replayed independently with full diagnostics and RNG/cost assertions.

Regression coverage compares bounded and unbounded decisions/checkpoints, rejects
one-below-winning budgets, compares cached Oresrush snapshots including trace,
and retains the independent uncached Snows oracle. Unsupported enemy fixtures
must report the same error in both algorithms, not be treated as valid routes.

Release measurements (seed 0, title advance 1203417, profiling query in
`profile_current_route` with Required outcomes): bounded/deferred version 44.005s;
with the formation frontier 27.909s. JSON comparison excluding search counters
found zero differences. This is the calculation benchmark, not a browser latency
guarantee. The earlier formation stage still used 27.782s. Saved Oresrush/coasts
fixture improved from 324ms to 82ms with identical full replay except search counts.
