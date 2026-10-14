# Nameless Isle approach (inactive candidate)

`jp_finish(entry, max_inputs)` implements the daytime water JP100 alternative.
It equips the entry gear, runs to the first water encounter and accepts Ebony
Eel x3 only. It does not search opening turn order. Reject T->Throne->Osvald;
the other five relative orders use the agreed Defend/Springy Boots branches.
Partitio gives Temenos one small Pom. T2 Temenos +3 BG then Osvald +1 Fireball.
Search complete BG outcomes and actual Fireball execution, requiring JP100,
all four alive, and victory. Reject disrupted order/resources/silence/failed kills.
The caller's acceptance filter replays drops and field restoration and requires
a safe mixed run/walk remainder. Report percentages of the complete island path,
reserve three running decrements at the end, carry all rewards/inventory/resources.
JP manipulation ends at victory; the returned Gigantes-entry waypoint only
describes safe movement, with no further RNG advance or boss simulation.
The ordinary `finish` still uses Escape and proceeds through Gigantes/Georges.
Automatic location selection and UI activation remain separate work.

`finish(entry, max_cycles)` now wraps this approach with the manual entry menu
and complete Gigantes routine. It remains disconnected from the automatic
JP-location selector and UI. The lower-level `escape_approach` remains available
for counter-only tests. Import `import-galdera-nameless.cjs` after town imports.

The menu requires the actual acquired gear and nuts; missing inventory fails
without partial mutation. Osvald (Warrior) equips Firestarter, Scorched Bone
Spear, Skull Helm, Elemental Augmentor and consumes two Magic L. Throné
(Inventor) equips Swift Shield/Quick Cloak and consumes Critical L/Fortifying L.
No Mage's Robe, Finisher's Claws, Lost Tribe Dagger, Blessing in Disguise or
Critical S is fabricated. Applying the menu again does not consume nuts twice.

At the boss, switch night, accept the source prompt, execute the agreed two-turn
routine and carry actual resources, rewards, drops and weakness history through
field return and victory scene/flag40003. The level70 fixture has797 elemental
attack and a conservative51506 damage floor against50000 HP, excluding criticals,
fire bonuses, T1 attacks and extra Barrage hits. The floor is recalculated from
the carried loadout; source RNG prediction still awaits retail comparison.

After the existing Georges rescue scene, switch to day, steal Mage's Robe
(guaranteed at Throné47), switch path action/close, then open Scrutinize.
Scrutinize uses Osvald's actual level and the native SearchRate table (72% at70).
Failed Scrutinize candidates trigger a search of Partitio's T2 finisher boost
cycles (0->1->0, then normal 0->3). Replay the actual battle and all subsequent
callbacks, choosing the fewest cycles; return None only when the budget is exhausted.
`Trip.finisher_boost_cycles` records the selected adjustment. Success carries the translation information item,
the before-dialogue lottery and standard success event; Georges has no special
Scrutinize event. Open the fixed Finisher's Claws chest without an RNG draw.
Final continuation state is `Trip.loot.end()`, not `Trip.victory`.

Seven focused continuation tests pass, including missing-inventory rollback,
idempotence, whole-route percentages, boss/cutscene propagation and both
Scrutinize outcomes. Binaries compile; retail RNG comparison remains pending.

Fireball damage check for the prospective Ebony Eel x3 JP target: each has2367HP,
231MDEF and no fire weakness. Lv70 entry loadout (797MATK, both Magic L, fire
potency60%, no robe/claws) deals2578..2680 per target with +1BP by day, or
3053..3174 with0BP and both night talents. No break or Advanced Magic required.
The JP routine now validates actual BG effects and Fireball execution in addition
to this independent damage check.

The caller supplies a daytime Nameless Isle entry with Evasive Maneuvers.
The active route and UI do not call this builder yet.

| Segment | Running updates | Walking updates |
|---|---:|---:|
| Initial land to canoe | 70 | 73 |
| Water crossing | 67 | 69 |
| Final land to Gigantes | 181 | 191 |
| Total | 318 | 333 |

Always run to the first encounter on water. Reject ambushed upstream candidates.
Search actual boost/cancel inputs before guaranteed Escape, replay field return,
and evaluate its actual reset counter. Minimize boost cycles first, then maximize
running with a single run-to-walk transition. Walking is rounded up separately
for remaining water and final land. Reserve three additional running updates
(21 counter units with daytime EM) at the endpoint, even if walking there.
The exit counter must remain strictly positive.

Percentages refer to the entire unpadded route, including the mandatory running
prefix. Distance is normalized using each segment's walking measurement; no
single global conversion factor is substituted for the three segment lengths.
Exact segment counts and unrounded percentages are returned for later UI work.
The endpoint counter includes the conservative padding budget.

The builder only simulates an Escape approach to the boss. The Ebony Eel marker
identifies the prospective JP formation, not a validated JP budget or JP100
victory. Harbor boarding and automatic JP location selection remain separate
unfinished work. JP victory is modeled by `jp_finish`; Gigantes and post-boss
loot are modeled by `finish`.
No UI changes are made.

Validation: all first-water positions 2..59 and reset counters1000..1798 are
checked for positive padded exits, correct segment costs and one switch; exact
full-running and all-walking boundary cases; actual battle/field-return replay
across16 synthetic entry seeds. Retail validation remains pending.
