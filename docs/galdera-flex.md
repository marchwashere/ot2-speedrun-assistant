Current order supersedes older notes: spear -> Firestarter -> Crackridge book -> Gravell restoration/supplies -> Roque -> Nameless -> Shipwreck. Firestarter/book are committed before the deferred JP search; Firestarter is no longer a JP-search Defend control site. JP location remains flexible.

Current route order: keep the Gravell restoration/supplies stop, then always
Firestarter -> Crackridge book -> boat/Roque -> Nameless Isle -> Shipwreck.
Only the JP100 location and RNG controls remain flexible. Prediction still
ends at the chosen JP100 victory; subsequent destinations keep this order.
This supersedes historical permutation-search descriptions below.

# Flexible route to JP ×100

## Active automatic location selection

Defend refinement now follows the existing accepted switch-only search. Five
controls allow0..7 Defends: Western Sai Firestarter encounter, Beasting Bay,
Western Tropu'hopu, daytime Roque Harbor, and Nameless water when fleeing for
Shipwreck. Each action uses the core safe-step-ahead continuation, permitting
the first regular turn only while the next command remains a player. Escape
must still be100% with the carried party. No enemy actions are accepted.
Defend costs3 inputs; entering the next round costs8 once, so seven cost29.
All resulting resets, run/walk requirements, formations, gear and battles are
replayed. Viewer lists actual Defend actors/turns and the later Escape menu.

Refinement compares every single-site count and uniform multi-site counts,
then performs a second coordinate pass around an improved winner. It compares
Roque prefixes and all six later errand orders, with the existing Tropu switches
and fine boosts available. A shared12000-candidate budget bounds refinement;
the accepted fallback is retained. This is a bounded improvement search, not
exhaustive enumeration of all8^5 Defend combinations. Small input-cost lower
bounds prune candidates only when they cannot beat the current accepted plan.

Crackridge book (2), Roque boat/loot (1), and Firestarter (3) may occur in any
of the six permutations. All three must finish before Nameless Isle (4).
Shipwreck (5) requires the complete Nameless Escape/Gigantes/Georges route.
If JP100 succeeds at Roque or Nameless, the manipulation stops at that victory;
later errands/bosses are not searched. Roque retains its five early prefixes.

Every later candidate adjusts at Tropu'hopu BEFORE the shipyard, using even
switch counts so Roque Harbor stays daytime. The search replays the entire
downstream route, including all remaining errands, native map/time callbacks,
escapes, inventory, equipment, Gigantes and successful Georges Scrutinize.
It then runs the full JP battle search. Thus there is no guessed fixed offset
that can accidentally skip an otherwise viable candidate: each departure's
actual battle state determines its 96-input fine-search window. The returned
downstream_draws measures final Tropu adjustment to battle initialization.
One regression measured495 draws for Nameless and963 for Shipwreck; these are
examples, not constants or distances to the BG lottery itself.

Start with the accepted Roque plan. Compare all later order/location/switch
candidates whose lower-bound input cost can beat it. Cost is seven per extra
time switch plus actual boost inputs, including Escape and Gigantes adjustment
cycles. Required errands remain free to reorder per user instruction. Stop
coarse exploration once its cost alone cannot beat the incumbent. Ties retain
the earlier accepted candidate. A seed0 regression selects Nameless with
order[1,3,2,4], cost16, proving the selector can actually choose a later site.

The selected result includes its full continuation and chosen location in the
viewer/detailed log. The terminal field_return always means JP100 victory.
Shipwreck's fixed Rusty Dagger pickup is retained in branch data but does not
extend manipulation. No measured Shipwreck remainder has been supplied.

Search limits:96 fine boost inputs and48 Escape/Gigantes adjustment cycles.
Only the first safe Escape/Gigantes continuation is compared per departure.
This selects the cheapest evaluated plan, not a global optimum over every
possible battle action or upstream adjustment. Exact simulated replay still
needs retail validation. The historical inactive descriptions below describe
the candidate builders before this selector was connected.

## Deferred Roque continuation (inactive)

`galdera_flex::deferred_roque` is a candidate builder for the future automatic
Nameless Isle / Shipwreck comparison. It is not called by the active route and
does not expose a manual UI selector. Input is the daytime Gravell departure
Entry (or an entry after completed useful errands), and an even Tropu switch count. Odd switches are rejected: the boat and
harbor must remain daytime. An ambushed harbor candidate returns None so a future
search can try another preceding adjustment; no silent on-field time switch.

After one normal/preemptive harbor encounter and guaranteed Escape, enter Roque
Island during daytime and discard chapter selection. Factory Worker TALK_0100:
Pom M and L only; no Wind Soulstone pickup. One opening plus one
draw per steal, switch path action and close. Night, Ambush Guard TALK_0200
(level48), execute native Ambush dialogue scripts, immediately switch back to
day, then open six fixed chests:
Lychee L, 39,800 leaves, three Rejuvenating Jams, Magic Nut L. Buy Swift Shield
and Quick Cloak from normal stock for42,000 total; Smooth Talker is a sale
talent and does not discount these purchases. Shop transactions and fixed
chests consume no CRT. Purchases are carried as inventory, not auto-equipped.
Remain in daytime when teleporting to Beasting Bay: Anchorage and sailing to
Nameless Isle. Replay the post-Ambush switch through the normal loader callbacks.

The returned final Entry retains RNG, NPC/flag/encounter history and supplies;
`jp100_complete=false`. Chest money and shopping costs are separate fields,
not a fabricated complete wallet balance (the existing world does not track
all earlier spending). No departure path or later JP battle is invented.
The existing night Roque JP100 search still ends immediately at its victory.
Retail validation of the new endpoints remains pending.

Gravell skips both Old Man and Villager. Sharp L and Diffusing Serum are already carried from the Wellgrove inn Shopkeeper steals; no Serum purchase here. Temenos learns Bewildering Grace; retain the earlier A Step Ahead and Boost-Start setup. Menu setup consumes no CRT, then switch night for the existing blacksmith/sales/Jam sequence.

The search evaluates [boat], [book,boat], [Firestarter,boat], and both orders of
the two errands before boat. The user considers these required errands free
when reordered; scoring charges only extra switches (12 input-equivalents each)
and boost inputs. Unfinished errands are listed after the manipulation endpoint.
Fine battle search uses a96-input window and compares complete BG/Fireball pairs
by total input count. Coarse switches grow until a valid plan exists, then
compare alternatives below the incumbent cost. This is the least-cost evaluated
plan, not a proof of a global optimum over arbitrary actions or larger windows.

Beasting: run to the first land encounter, Escape, use real menu boost paths to
select a reset counter that permits the remaining measured203-run/217-walk
crossing plus3 updates of padding. Maximize running percentage for the selected
counter, round walking up, require a positive exit counter. That same Escape
adjustment also rejects ambush in Western Tropu'hopu's one encounter. Initial
ambushes use logged departure time pairs. Unmeasured encounterless legs use the
user's specified encounter counts, with no invented travel distances.

Crackridge: daytime Mason Mighty Leaf Steal and switch/close, night Scholar
Ambush (level30), immediately day, fixed book chest. Firestarter: Sai, Southern
Sai without encounter, one Western Sai Escape, Reinforcing Jam chest, Eastern
Ku without encounter, Firestarter chest. Extra Sai pairs are displayed only if
needed to prevent an ambush.

Tropu'hopu switches execute the normal loader callbacks. In the retained fixture,
the first day-to-night switch consumes20 CRT calls and the pair49; these counts
are measured from the carried state, never hardcoded. Shipyard events
MS_SHO_EX1_0100/0110/0200 and purchase0400→0410→0500→0600→0700 execute their flags,
NPC refreshes, encounter resets and ocean load. Default hull/sail, finish and
confirm; no decoration changes and no closing path-action trick. Ship purchase
costs100000, covered by the preceding250000 weapon sales. No sea encounters.
Roque entry switches to night if not already night.

Roque requires two Winged Landwalkers, no rare replacement and no Surprise.
All four have A Step Ahead; Osvald has Boost-Start. Use the same five opening
orders as Shipwreck: reject Temenos -> Throne -> Osvald, ignoring Partitio's
position. Partitio uses unboosted Donate BP on Temenos; the shared Defend/
Springy Boots branches put Temenos first and Osvald second on T2.
T2: Temenos +3 BG, then Osvald +2 Fireball. Execute all BG effects and the
actual spell; reject disrupted order, death, silence, insufficient resources
or incomplete victory. Require JP modifier10000% and all four alive.
Rewards are116 base JP x100 plus Domination =12760 each. No Life in the Shadows.
No Pom or Hired Help money is consumed; carry actual drops/history/resources.
At Lv70, 774 MATK and +60% fire potency, nighttime buffs/debuffs give7243..7530
per unbroken Landwalker (6897 HP), before any beneficial BG effects.

Source import: tools/import-galdera-flex.cjs after the existing town/event
importers. New maps are included by import-town-loading.cjs. Core imports:
import-highroad-return.cjs, import-items.cjs, import-gravell-battle.cjs.
Existing map/event semantics are preserved; only new maps/placements/scripts
are added. Blueprint ship initialization is retained in the source audit.

Tests cover the turn/BP premise across128 seeds, all64 core tests, both errand
orders, boat events and time switches, full identified-route replay and three
alternate synthetic RNG states. New endpoints still need a retail trace;
simulator reproducibility alone is not evidence of live CRT alignment.

Supply revision: skip the Gravell Villager. Steal Strengthening Serum with Champion Seeker from the Man with Horses (guaranteed at Lv32), then switch path action and close. All three JP locations use unboosted Donate BP, consuming no Pom. The previously reserved Roque Pom M is spare: planned Tyrannodrake T2 Osvald Defend, T3 latent +3 Fire Storm, with Peak Performance, Deal More Damage and the agreed equipment. Tyrannodrake is not yet an executable route segment.

Reload cost revision (2026-10-15): Tropu’hopu reloads 1–3 cost zero input equivalents; later reloads cost 12 each. In Roque Island, post-Ambush reloads 1–4 cost zero, including the mandatory night-to-day switch; later reloads cost 12 each. The earlier day-to-night switch for Ambush does not count toward this allowance. Every switch still executes native loader callbacks and consumes real RNG. Search post-Ambush odd counts to preserve daytime departure, including both free choices (1 and 3); paid alternatives are explored within a 96-input reload-cost window and the incumbent budget. These are bounded evaluated candidates, not a global optimum.
