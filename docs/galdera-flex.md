# Gravell to Roque JP ×100

Gravell arrival stays daytime for the Villager's small Energizing Pomegranate
and Strengthening Serum. One Steal menu, two successful checks, then path-action
switch/close: three CRT calls. Guaranteed levels18/32. Temenos buys Bewildering
Grace (30 JP) and equips A Step Ahead through the existing Inventor license;
Partitio buys Collect (500 JP) and equips Boost-Start. Prior jobs are restored.
Menu setup consumes no CRT. Existing blacksmith/sales/Jam sequence follows.

The search evaluates [boat], [book,boat], [Firestarter,boat], and both orders of
the two errands before boat. The user considers these required errands free
when reordered; scoring charges only extra switches (7 input-equivalents each)
and boost inputs. Unfinished errands are listed after the manipulation endpoint.
Fine battle search uses a96-input window and compares complete BG/Guards pairs
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
All four have A Step Ahead; Partitio has Boost-Start. Require Partitio before
Temenos in the opening. T1: both Defend; Throné small Pom to Temenos; Osvald
unboosted staff on the rightmost enemy. T2: Temenos +3 BG, then Partitio +3 Guards.
The latest Defend is first; both reach3 BP. Execute all BG children, reject
disrupted order/death/silence/insufficient BP, then actually execute Guards.
Acceptance requires victory, all four alive, JP modifier10000%. Rewards are
116 base JP ×100 plus Domination =12760 each. No Life in the Shadows equipped.
The small Pom is removed from inventory and real drops/history/resources carried.

Source import: tools/import-galdera-flex.cjs after the existing town/event
importers. New maps are included by import-town-loading.cjs. Core imports:
import-highroad-return.cjs, import-items.cjs, import-gravell-battle.cjs.
Existing map/event semantics are preserved; only new maps/placements/scripts
are added. Blueprint ship initialization is retained in the source audit.

Tests cover the turn/BP premise across128 seeds, all64 core tests, both errand
orders, boat events and time switches, full identified-route replay and three
alternate synthetic RNG states. New endpoints still need a retail trace;
simulator reproducibility alone is not evidence of live CRT alignment.
