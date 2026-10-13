# Sai return to Quicksand Gaol

After Temenos recruitment, use Warrior license2 for Osvald (replacing Merchant0), Dancer license5 for Temenos. Native job revisions, permitted weapons, default commands and support unlocks update through the shared assignment helper; existing learned/equipped supports survive. No JP purchases or new support assignments. Setup consumes zero RNG.

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
