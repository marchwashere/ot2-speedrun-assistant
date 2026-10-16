# Sai return to Quicksand Gaol

## Current routine (supersedes the historical routine below)

After Temenos recruitment: Osvald learns Lightning Bolt as his fifth Scholar
skill (500 JP), immediately equips Elemental Augmentation (+50 elemental attack),
then learns Collect while Merchant (500 JP) and takes Warrior. Partitio learns
Collect. Equip Boost-Start on both immediately. Osvald retains Evasive Maneuvers
and A Step Ahead, filling all four support slots. The flat elemental bonus survives
job changes, levels and later equipment menus without being applied twice. Give
Temenos Dancer and A Step Ahead directly; retain A Step Ahead on all four.
Gravell subsequently teaches only Temenos Bewildering Grace. No menu RNG.

Night Sands routine: T1 Throne Critical Scope, Osvald Defend, Temenos Holy Light,
Partitio Defend if before Osvald, otherwise unboosted lance. T2 Osvald +3
Aggressive Slash, Partitio +3 Beastlings, others Defend if needed. Six sword hits
plus Holy Light break seven shields. Both merchants have3BP on T2. Implemented
by `gigantes::predict_sands`;128-seed tests cover six-hit Slash and unchanged
action prefixes when adjusting the finisher. The UI uses actual action reports.

## Historical three-turn implementation

After Temenos recruitment, use Warrior license2 for Osvald (replacing Merchant0), Dancer license5 for Temenos. Native job revisions, permitted weapons, default commands and support unlocks update through the shared assignment helper; existing learned/equipped supports survive. Osvald first learns Collect while still Merchant (500 JP), unlocking Boost-Start; he equips it later at Gravell, not for Lord of the Sands. No new support assignments here. Setup consumes zero RNG.

Teleport daytime Sai, discard chapter starts, load Southern Sai then Western Sai with no Southern encounter per user routing. Predict one Western land encounter, reject Surprise, first-menu Escape100, then load Quicksand Gaol Dng_Dst_2_2 map40. If ambushed, increase complete boost0→1→0 cycles before the +1BP Temenos recruitment attack, require a one-action kill, and replay rewards, field return, recruitment scenes, party swap, jobs and all loads. Introduction/first menu and earlier route remain frozen; there are no Sai time switches. Distances after Sai are unmeasured, so this models the specified encounter count without inventing movement/counter decrements. Continue with the agreed boss routine below.

Temenos is level1 and has no A Step Ahead. Bonus-turn members on Normal openings are indices0,1,3; Preemptive uses the native all-player first queue. Core imports Temenos voices/latent cap110 and accepts an explicit support-member list. Osvald retains native Evasive Maneuvers, Throné/Osvald/Partitio retain A Step Ahead. Escape evaluates all four actual levels.

Detailed log records setup, Sai/load/encounter/return/dungeon boundaries. Separate Quicksand card shows formation and first actor. Source importer adds map40; rerun additive route event importers after town imports. Retail validation pending.

## Agreed Lord of the Sands routine

Switch to night before the boss so Throne and Temenos apply their night talents.
The labels below are actions per character, not a fixed order within each turn.

- T1 (A Step Ahead): Throne +1 BP dagger attack (two hits); Osvald Defend;
  Partitio Defend. Temenos has no A Step Ahead and does not act.
- T2: Throne Critical Scope; Osvald +2 BP Aggressive Slash (five sword hits);
  Partitio Defend; Temenos unboosted staff attack.
- T3: Partitio +3 BP Hired Help: Beastlings.

Keep Temenos's staff attack as requested; do not substitute Defend silently.
Boss: 40,000 HP, seven shields, sword/dagger weaknesses, physical defense270.
Two dagger hits plus five sword hits break on T2. Osvald's previous Defend puts
him before the boss; Critical Scope only needs to precede Beastlings. Partitio's
previous Defend gives his T3 action priority. Temenos's three-turn defense debuff
remains active. Beastlings use their own stats, without Throne's attack buff:
minimum computed damage10,441 per hit before cap, actual9,999 x4 =39,996.
Earlier attacks cover the remaining4 HP. Cost: Osvald9 SP, Beastlings15,000 leaves.

Status: implemented. User confirmed no random encounters between entry and boss.
Manual night switch uses the real map callbacks. EV_Dng_Dst_2_2_0010 Yes0 then
command600 selects the first fixed group ENG_DNG_DST_2_2_L_110 (no random
formation/situation lottery). Core replays eight actions in actual turn order,
retains three-member ASA, applies both night talents, rejects any enemy action or
unexpected order, records rewards/drop lottery and actual party resources/history.
Field return uses native callbacks, then EV_Dng_Dst_2_2_0020 sets flag40010.
Temenos receives the actual EXP/JP with the other active characters. Beastlings'
15,000-leaf cost is reported explicitly; the route has no shared wallet ledger.
Detailed log includes each action before/after plus all world boundaries; a
separate boss card shows actual action order. No dungeon movement counts invented.

Regenerate core tools/import-lord-sands.cjs after import-bg-effects; regenerate
assistant tools/import-galdera-lord-sands.cjs after town/event imports. Core tests
cover128 RNG seeds and source trace continuity; the full route and browser fixture
tests cover integration. New boss checkpoints still await retail trace validation.

## Post-boss weapon pickups

Switch to day, open Treasure_Dng_Dst_2_2_01 (Rusty Axe, ITM_TRE_WPM_04),
take one encounter on ordinary land outside the arena, Escape, then open
Treasure_Dng_Dst_2_2_05 (Scorched Bone Spear, ITM_EQP_LNS_220). Retain both
items unequipped and persist opened chest IDs. Neither chest has random contents
or placement events; TreasureBoxBP uses a literal OpenTimer, fixed PickItem and
ordinary reward dialogue, with no CRT draws or encounter counter reset.

The placed land trigger replaces its table when defeat flag40010 is set:
ENT_DNG_DST_2_2_L_LND, not the pre-boss ENT_DNG_DST_2_2_LND. Its table includes
ENG_DNG_DST_2_2_L_120 (repeat Lord of the Sands, Escape enabled). Import this
enemy along with all ordinary/rare possibilities. Require Normal/Preemptive and
100% Escape with the carried party. If ambushed, search minimum complete 0→1→0
cycles before Partitio's final +3 Beastlings, replaying actual voices, damage,
rewards, return, day switch and chest. Earlier battle actions stay frozen.

User specified the encounter count/order, not distances. Do not infer a running
percentage or guarantee the physical crossing from an invented movement budget.
The post-Escape counter remains the reset counter until further distances are
provided. Source audit: tools/import-galdera-quicksand-loot.cjs. Retail pending.

## Live post-boss encounter correction (2026-10-16)

The land encounter after Rusty Axe retains `ENT_DNG_DST_2_2_LND` on this
uninterrupted visit. `EncountTrigger.SetEncountVolume` evaluates the replacement
flag when the player activates the collision trigger, not as a live flag binding.
The boss victory flag does not itself refresh the active encounter volume.
The former eager switch to `ENT_DNG_DST_2_2_L_LND` selected the wrong formation.
Source methods are already recorded in `data/source/galdera-quicksand-loot.json`.

The carried snapshot and live observations in `quicksand-post-boss-live.json`
reproduce menu7048 with Partitio/Temenos/Throne/Osvald on the bonus turn, then
Assassin Bug/Savage Scorpion/Throne/Partitio/Osvald/Desert Worm/Temenos. This
regression covers formation, RNG position and both queues, with flag40010 true.

Nameless Isle audit: flag40003 enables a replacement land label on its land trigger and canoe landings. The current route encounters water enemies before Gigantes and takes no post-boss encounter, so it does not select that replacement prematurely and requires no corresponding runtime change.
