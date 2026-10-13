# Active project

- Shared core turn-order correction: later Defend overtakes earlier Defends and
  Springy Boots, after break recovery. Four ASA Defends reverse party order in
  the next turn. Boots excludes shuffle draws and adds 20000 to future timing;
  break immediately moves hidden rows to the end. Recalculate route outputs;
  see core docs/turn-order-controls.md. Both Galdera web fixtures keep their
  prefix through Gaol entry, but Lord Sands actions/exit RNG change.

- After Lord of the Sands: switch day, open Rusty Axe chest, one ordinary-land
  encounter outside the arena, guaranteed Escape, open Scorched Bone Spear chest.
  Defeat flag40010 replaces ENT_DNG_DST_2_2_LND with ENT_DNG_DST_2_2_L_LND,
  including the repeat-boss group. Use that post-boss table. Reject ambush via
  complete boost/cancel cycles before Partitio's final Beastlings; freeze all
  earlier actions and route controls. Fixed chests consume no RNG; record opened
  chest IDs and inventory. No measured movement budget exists for this leg.

- Lord of the Sands routine agreed: night before boss; T1 ASA Throne dagger x2,
  Osvald/Partitio Defend; T2 Throne Critical Scope, Osvald +2 Aggressive Slash,
  Partitio Defend, Temenos staff; T3 Partitio +3 Beastlings. Preserve these actions.
  Implemented through night switch, boss prompt, all eight actions, victory rewards,
  field return and defeat flag40010. User confirmed no dungeon encounters.
  Source-backed prediction awaits retail trace validation; see docs/galdera-quicksand.md.

- After Temenos recruitment, assign Osvald Warrior2 and Temenos Dancer5 using
  existing licenses/native job stats; retain existing supports, no Temenos Step Ahead.
  Teleport Sai daytime, discard chapters, Southern Sai no encounter, Western Sai
  one Normal/Preemptive encounter with guaranteed Escape, load Quicksand Gaol
  Dng_Dst_2_2 map40. Temenos recruitment boost cycles repair ambushes; no Sai time switches. Three-support
  opening uses actual [0,1,3] bonus-turn membership; Temenos joins regular turn.
  Distances unmeasured: do not invent counter budgets. See docs/galdera-quicksand.md.

- After Flamechurch entry: recruit Temenos via MS_SIN_00_0100 Yes59/battle82,
  first actor +1 BP regular attack with actual boost/cancel kill search. Normal
  attacks use80% AbilityRatio; Osvald/Partitio may require crit manipulation.
  Native field return, MS_SIN_00_0300 Yes46/skip58/replace Agnea68; Temenos starts
  level1, party [7,6,5,4], Agnea benched. Log every boundary and show separate card.
  Source imports and core profile documented in docs/galdera-temenos.md.

- Flamechurch continuation now uses the preceding Clockbank Escape boost/cancel cycles
  to ensure an encounterless Eastern Flamechurch Pass crossing. Measured133 run/137 walk +3 each (136/140);
  require counter561+, maximize safe running within first accepted counter,
  display run/walk percentages. Carry real Escape/reset/map-load RNG; freeze opening
  and earlier controls. Load Flamechurch map20 and discard chapters. See docs/galdera-flamechurch.md.

- After Al delivery: run100% for56 land updates on New Delsta Highroad, cost336
  with day/Evasive (6/update), then load Western Clockbank Fld_Cty_1_4. Run100%
  for193 land updates toward Eastern Flamechurch; exactly one encounter, native
  Escape100 at actual first actor, remaining traversal without a second fight.
  User confirmed entirely on land. Western volume aliases CTY_1_3 land tables,
  core coverage already complete. Freeze Al/earlier controls; only if ambushed,
  add complete night/day pairs at Clockbank entry using real callbacks/counter
  preservation. Show extra pairs, formation, actor and Run100% in a distinct
  Galdera Clockbank card. Native field reset pays remaining193-trigger updates;
  load Eastern Flamechurch Fld_Mnt_1_2, stop at daytime entry. Map6 is imported;
  rerun Bag/Oresrush/Winterbloom/Wellgrove-supplies event imports after town.
  See docs/galdera-flamechurch.md; retail validation pending.

- After Dancer License: teleport New Delsta main, discard chapters, immediately
  switch day in town, enter Highroad Fld_Cty_1_3 and deliver recovered bag to Al
  via SC_FC13_Tutorial_0400 command4 Yes. No unmeasured movement/battle invented
  on the direct entry-to-Al leg. SC_FC13_Tutorial_0310 command29 now records
  the actual earlier bag grant (ITM_TRE_SS_TD11_0100_0010), with zero RNG change.
  Delivery consumes bag, sets20511, removes Al via native9990 refresh, resets
  counter B/A and rewards Slippery Nut1, Healing Grapes3,1500 leaves metadata.
  Same-map event avoids another load; fixed dialogues/temp scene actors have
  no draw. All stages in Detailed log. Source audit import-galdera-al-return.cjs
  reuses existing Bag events and confirms SubDeliveryItem. Retail pending.

- After Wellgrove entry: immediately switch night, then one tavern Shopkeeper
  Entreat menu obtains Sharp Nut L and Diffusing Serum; change path action before
  close (one opening draw, no item probability checks). Both require Agnea26;
  NPC_Twn_Fst_2_1_A_TALK_0400. Then Entreat Empowering Lychee M from Girl's Mother,
  followed by path-action
  switch/close (1 CRT opening; ProperBeg20). Use actually placed quest NPC
  NPC_SS_TF21_0200_0300, not unused A_TALK_0600; same items, quest not accepted.
  Acquire first Dancer License at night (JobID5, SYS_DAN_GUILD_0010/0030): fixed
  dialogue, flags406false/411true, no reset or RNG; record license without job
  assignment or purchases. All steps in Detailed log; retain supplies unused.
  tools/import-galdera-wellgrove-supplies.cjs adds runtime/source audit and event
  scripts; rerun after town/bag/Oresrush/Winterbloom importers. Retail pending.

- Eastern Wellgrove continuation: run43 updates to Man with Horses, Steal
  Champion Seeker ITM_EQP_DGR_120 from NPC_SS_TD31_0300_0300 (ProperSteal38,
  level49+ gives100%, Throné70), switch path action before close: two draws,
  opening timer + success check, no dialogue. User measures125 running updates
  total to Wellgrove. Require exactly one Normal/Preemptive land encounter AFTER
  Steal, immediate Escape100, remaining traversal without a second encounter,
  Wellgrove main entry and discarded chapters. Run100% throughout; retain item.
  Freeze Sai/Western opening and earlier decisions, search the Western first
  Escape menu's complete boost0->1->0 cycles only as needed. Replay field return,
  Eastern load, actual counter43*7 reduction, Steal and encounter; reject entry
  counter<=301 or>875. Counter after Escape pays remaining measured updates.
  Detailed log includes Steal/movement/town; both encounters have separate cards.
  Source import tools/import-galdera-champion.cjs audits NPC/item/threshold; core
  import-highroad-return adds Eastern Wellgrove land enemies. Retail check pending.

- After Winterbloom Soldier Entreat: fast travel Sai, discard chapters,
  immediately switch day, cross Southern Sai (Fld_Dst_2_3) without encounter,
  load Western Sai (Fld_Dst_2_2), exactly one land encounter, first-menu Escape,
  then load Eastern Wellgrove Trail (Fld_Fst_2_2). No invented movement budget
  for the user-confirmed encounterless crossing. Native Western volume aliases
  Southern Sai land tables; core already supports every referenced enemy.
  Preserve carried party/history/levels/inventory. Avoid ambush via the nearest
  Sai time control: only if required, extra complete night/day pairs after the
  immediate day switch, then re-evaluate native loads. Show extra switches and
  actual actor/Escape100 in the Western Sai card; fixed loads in Detailed log.
  Source-loaded maps and regressions are included; retail validation pending.

- After Sai East District Entreat: fast travel Winterbloom at night, discard any
  chapter start, deliver Sturdy Pickaxe via SC_SS_TSnow21_0200_02A0 Yes. Consume
  Pickaxe; reward Rusty Sword, Revitalizing Jam, 7000 leaves. Native 9990 refresh
  removes quest NPC and resets encounter counter; temporary quest actor is a
  field character without randomized NPC frame. Then Agnea Entreat Soldier's
  Fortifying Nut L (ProperBeg26), switch selected path action before closing.
  One opening timer draw only; inventory retained without consuming the nut.
  Fixed steps appear in Detailed log. See docs/galdera-winterbloom.md and source
  audit; endpoints await retail validation. After other event importers run
  tools/import-galdera-winterbloom.cjs to retain the quest event.

- After Pickaxe: fast travel back to Sai main, enter Sai: East District
  Twn_Dst_2_1_B, decline any chapter prompt, remain at night, Agnea Entreat Elemental
  Augmentor ITM_EQP_ACS_250 from NPC_Twn_Dst_2_1_B_TALK_1800 (requires level25).
  Change selected path action before closing to skip dialogue. Entreat adds only
  the opening action timer1 CRT; BegItemExec has no probability lottery. Carry
  the item in route_items and retain hired Smooth Talker; end at night, unequipped.
  East District map and source Entreat Blueprint audit are imported. Fixed steps
  appear in Detailed log, native endpoint still awaits retail validation.

- After Sai: Warrior first license (JobID2, SYS_WAR_GUILD_0010/0030; source
  current route zero CRT), fast travel Oresrush C, decline chapter prompt,
  night -> Hire Peddler NPC_SS_TW11_0100_0100 (1600 leaves, Smooth Talker
  SPACT_MERCHANT_014) -> stay night -> Entreat Pickaxe from Retired Miner, change
  path action then close to suppress dialogue. NPC_SS_TSn21_0200_0200 has an
  all-day placement, Beg enabled (CommandEnable bit4), ProperBeg30. Agnea70
  qualifies. Hire timer + dialogue lottery2; Pickaxe Entreat opening timer1,
  no probability or closing draw. Paying/registering follower adds no CRT draw.
  Remain night through Sai main/East District Augmentor Entreat and Winterbloom
  Sword quest/Fort L Entreat; switch day only after landing back in Sai.
  Retain hired identity/talent and quest item in state/cache; suppress original
  hired NPC placement on refresh. Conning Creek keeps its existing Steal
  closing exception; Oresrush now uses the same Entreat closing trick.
  Fixed steps belong in Detailed log. Regenerate event data with import-galdera-
  oresrush.cjs after town and bag imports. See docs/galdera-oresrush.md; endpoints
  await retail validation.

- Corrected Sai leg: ONE Eastern Sai Sands encounter, Escape; load Southern Sai,
  ONE encounter, Escape; enter Sai main town. Both openings must be Normal or
  Preemptive, with native Escape100% at carried levels. Eastern escape/history
  and field-reset RNG feed Southern loading (never skip this battle). Reject
  all arrival chapter selections throughout Galdera via MapInStorySelect policy;
  discard consumes no CRT and starts no chapter. Any% retains its old choices.

- Extend the same Galdera coast search to require a Normal/Preemptive first
  encounter in Southern Sai Sands (Fld_Dst_2_3), after loading Eastern Sai.
  Initialize the actual carried party, A Step Ahead and weakness history;
  verify the native level/danger escape calculation is100%, execute immediate
  first-menu Escape and restore field/history/counter with shared callbacks.
  Show formation/actual first actor/Escape100%; no unnecessary boost adjustment.
  Do not invent Eastern -> Southern movement or a second-encounter guarantee.

- Galdera now continues after EXP100: freeze the selected Guards/BG and earlier
  route controls, search only the last Thunder L menu boost adjustment and the
  optional Conning Creek landing-night/coast-entry-day pair (user cost zero).
  Switch day before tagging Gravell; fight -> Gravell is user-confirmed
  encounterless. Fast travel Conning Creek, cross Western Coast to Eastern Sai.
  Exact measured endpoints 113 walk/107 run +3 each; enumerate integral run
  updates, round remaining walk updates up, retain positive counter, show run
  percentage. Real BP availability matters: a zero-BP finisher cannot cycle.
  Detailed log carries mandatory travel/day change and exact counter budgets.
  Shared Loader imports Eastern Sai Fld_Dst_2_4. See docs/galdera-coast.md;
  these new endpoints await retail validation.

- Cat fixed elemental damage must use EnemyDB.UseCatDamageType: integer divide
  post-resistance damage by100, minimum1. SSM deals11 to Cultured Cait8HP,
  so the coast pack does NOT grant Overkill EXP. Missing this awarded129 extra
  EXP and made Agnea Lv15 instead of14, reversing Highroad menu order.
  Source audit data/source/cat-fixed-damage.json (core).

- After Castti skip/bench: fast travel New Delsta main town, accept Agnea Chapter 2 through
  MapInStorySelect/MS_ODO_01 and skip MS_ODO_20_0100, then load Eastern New
  Delsta Highroad, exactly one expected encounter, no surprise hard requirement.
  Existing Western Coast boost/Conning town+Harbor switch search now preserves
  successful Pomegranate steal AND this safe opening; freeze earlier choices.
  Search nearest Highroad first menu boost/cancel for successful Escape. Then
  Arkar license SYS_INV_GUILD_0010, Elemental Bomb Bottle0D10, Critical Scope0C10.
  Assume crafting materials available; execute script flags/counter resets and
  track license11 plus globally unlocked Inventor abilities. Only variable
  encounter/Escape guidance in Manipulation; arrival/craft CRT in Detailed log.
  Highroad return uses all progression-tier enemies; after regenerating core
  field imports also run tools/import-highroad-return.cjs. Retail validation pending.

- After Canalbrine boat arrival recruit Castti via MS_KUS_00_0100: deliver one
  Healing Grape (command32 Yes), accept126 Yes, skip Chapter1 at135 No, retain
  Castti on bench (full party unchanged). Import her starting stats/EXP ledger
  and skipped-prologue flags/points. Use actual FinishPre refresh/reset. Assumes
  the grape is available and no earlier Castti interaction. Endpoint in Detailed
  log only; retail validation pending. See data/source/castti-recruit-audit.json.

- Canalbrine has six CRT-consuming gondola BeginPlay actors (three NoPaddle,
  three Paddle). Both classes call RandomFloatInRange(0,20) -> SetNewTime at
  entry214. Import their per-actor callbacks in level actor order, only for
  created visible field streams, as for canoe/flag/lamp callbacks. They were
  missing from the initial voyage prediction (19287459 vs retail19287465).
  Source audit: tools/audit-canalbrine-voyage.cjs. Do not add a fixed +6 offset.

- After Conning Harbor Pomegranate steal, fast travel to tagged New Delsta
  Harbor, skip SC_LinerShip_Cty_to_Isd to map69 Fld_Isd_1_3 (Beasting Bay:
  Anchorage), then skip SC_LinerShip_Isd_to_Sea to map123 Twn_Sea_1_1_A
  (Canalbrine). Both events load ocean189 first and reset at FinishPre.
  Carry party/EXP/time unchanged; no additional time switch or recruitment.
  Fixed travel checkpoints belong in Detailed log. Retail verification pending.

- Conning Harbor Townsperson Energizing Pomegranate (M) steal follows Harbor
  entry. Freeze earlier selected route; search Western Coast menu boosts plus
  town and Harbor time switches, two free switches PER zone, seven input
  equivalents for each further switch. Exit daytime. Preserve Western opening,
  rare victory/first-try Escape, real carried EXP/levels and shared loader state.
  ProperSteal is 8; use actual Throné level and FieldCommandWidgetSteal thresholds.
  See docs/conning-pomegranate.md. Source-derived steal awaits retail validation.

- Route now continues Conning Creek town -> Conning Creek: Harbor, map125
  Twn_Sea_2_1_B (GameTextEN confirms name; LevelTriggerTable town exit5160
  targets Harbor trigger02, WALK, no flag writes/conditional destination).
  Use shared Loader.visit_map and carry the town endpoint, party/progression and
  day unchanged. No Osvald chapter start. Harbor CRT/counter/trace belong in
  Detailed log, not a fixed-action Manipulation widget. Retail check pending.

- Town load must not run hidden battle-backdrop BeginPlay callbacks. Preserve
  all streamed-level constructor draws, but initialize environment actors only
  in created field_streams. Conning re-entry live fixture 2026-10-09 is the final
  segment after a manual CRT rewind: 19287194 -> 19287273 (79 calls). Earlier
  records belong to leaving town. See docs/town-loading-audit.md and the per-call
  regression; do not patch this with a town-specific draw subtraction.

- Attack instructions must identify the target position for single-target attacks in multi-enemy battles (rightmost/leftmost/middle, numbered from right for larger formations). Use the actual report action name, not the equipped weapon, so Osvald spell breaks never display as Staff. All-target spells need no target position.

- Snows searches may insert safe Defends after breaking. Accept a Defend only when its resulting player menu still has every living enemy broken; reject recovery/enemy-menu branches without running enemy AI. Replay guard RNG, turn order and BP recovery in Battle. User-specified Defend cost is 8 boost-input equivalents. Defend itself draws21 for Guard application plus1 guard voice; round/menu progression is additional. Field CRT imports must retain type17 guard rows for Throné/Osvald (copied from audited Pirro/Mug data by import-field-spells.cjs). Search successive safe Defends (bounded by the two-member break window), show actor/round/menu plus Defend in Manipulation, and keep final Escape guaranteed. This does not enable exposed Defend turns or arbitrary enemy combat.

- Snows break search now includes unboosted first-actor spells: Osvald Fireball/Icewind (innate), and Throné Darkest Night against a lone 1-shield dark weakness. Require every surviving enemy broken; reject full victory or any live unbroken target, never skip enemy AI. Preserve SP costs and per-target latent gains on field return. Spell break has the existing 10-input-equivalent action cost; compare R1/R2 boosts and direct Escape. UI names the actual skill. Callback data imports Fireball/Icewind from the Mug source, reusing real actor voices; multi-target reaction is aggregated. New spell-break RNG awaits retail verification.

- Crackridge Harbor joins Highroad and post-Harbor Flats as a mandatory Throné break encounter. The preceding Winterbloom control selects a non-surprised single enemy that her real turn-one Sword/Dagger attack can break (BP allowed). Execute the break, then guaranteed Escape at the actual next player menu; carry +25 latent, real levels/stats/weakness history and remaining BP. Never select direct Escape for these three active-route encounters. Freeze previous choices and preserve all onward movement/surprise goals.

- Latest Pirro → Snows policy: strict forward propagation. Pirro R8 requires no surprise plus an actual Throné turn-one breakable Highroad formation; Highroad requires the same for Flats after Harbor, preserving safe Harbor walking. BP is allowed for the break. Execute mandatory breaks in both encounters; search R1/R2 boosts using actual remaining BP. Freeze each selected upstream control before proceeding; do not run formation_search::optimize or revisit Flats to reduce Snows cost. Track +25 latent per successful break (cap80 for Throné); preserve carried resources through the rare item clear. No joint global optimum claim.

- Search optimization: propagate remaining incumbent cost into Snows and Hikari/Canalbrine/coast trials. A bound is NOT a Required-outcome retry limit. Formation and Oresrush searches use increasing cost frontiers, growing until a complete candidate exists; preserve candidate ordering and Northern tie preference. Quiet Flats trials compute only the predicted Osvald timing; detailed winning replay reconstructs overrides. Oresrush PrefixCache reuses town/Mother entry and Partitio recruitment per before-switch count, scoped to one immutable entry. Never share it across different world/RNG inputs. See docs/search-pruning.md.

- Latest time-switch policy supersedes Cropdale search below: NO Cropdale adjustments. Freeze earlier route/Eastern Cropdale Escape, compare Oresrush switches only after Mother Chapter 2 opening, before and after Partitio recruitment. Each side has its own two free switches; additional switches cost seven inputs. Total switches even (exit day); odd/odd allocations recruit by night and are replayed accordingly. Preserve walking-safe return-to-Ryu legs; strict nearest-menu downstream propagation and Northern Cait tie preference. Manipulation shows separate before/after Partitio lines in Oresrush chronological position.

- Tavern correction: shared Loader.refresh_bar uses data/taverns.json imported from cooked LevelTable/GameParam. Appearance 0..50/51..74/75..99 requests 0/1/2 slots, capped by map capacity (Winterbloom one). Each slot consumes a selection draw only if eligible bench remains, then ALWAYS an item-gift roll even for hidden ID-zero placeholders. Do not restore the old empty-bench one-draw shortcut. See docs/tavern-loading.md. Corrected source-derived totals await retail replay.

- Manipulation presentation: show only variable run decisions, not fixed every-run setup/routine reminders. Keep adjustments (including explicit zero-adjustment notices), menu identity, dynamic attack/break/SSM-vs-Escape choices, steal order and computed movement limits. Fixed Scholar assignment, party replacement, recruitment/story instructions, guaranteed travel and fixed night/day instructions belong in Detailed log. Cropdale before/after Agnea switch counts must be separate stacked lines via instructionBoxHtml.

- Cait location policy: actively search Canalbrine boosts for Northern Cultured Cait, comparing full downstream boost cost with the first-safe-Northern propagated Western fallback. Do not freeze Canalbrine at its first safe opening and accidentally force Western. Western only wins when strictly cheaper; prefer Northern on equal cost, including across Cropdale switch candidates. Preserve both no-surprise checks and successful Escapes.

- Latest search policy: Compare even Cropdale switch counts, first before Agnea and rest after, exiting by day. First two switches free; each later switch costs 7 boost inputs. Each candidate uses strict sequential propagation, not cross-menu allocations. Select lowest switch + Eastern Escape + downstream input cost; stop when switch cost alone cannot improve the best plan. Keep earlier route decisions fixed; use nearest boost menus only for the Conning continuation. Hikari protects Canalbrine opening, Canalbrine Escape protects Northern opening, Northern action protects Western opening and targets Cultured Cait if Northern did not supply it. Western boosts only secure Escape. All required outcomes remain mandatory; no joint minimum-input-cost claim. This supersedes the older joint-search notes below.

- Conning Creek extension: after the Canalbrine water Escape, visit
  Northern then Western Conning Creek Coast. Switch to night just before each
  first land encounter, switch back to day after each fight. Require no surprise
  on BOTH encounters, Cultured Cait in either (not both required), clear that
  pack with Shadow Soulstone M and Escape the other. Carry real EXP/JP, stats
  and party [7,1,8,4], bench [6]. End at Conning Creek town entry: DO NOT start
  Osvald Chapter 3 or replace anyone in the party. Optional Cropdale switches
  must put the FIRST day-to-night switch BEFORE Agnea recruitment; ALL remaining
  switches happen AFTER recruitment. Leave Cropdale during day (zero or an even
  number of switches). The first two switches cost zero; each further switch
  costs seven boost INPUTS. Replay recruitment at its actual time of day and
  source-driven NPC/bar/flag callbacks; never substitute a fixed RNG offset.
  enter_cropdale_with_switches records each boundary and departure() selects
  the actual final checkpoint. conning::extend_selected_routes runs at the GUI
  API boundary after earlier route selection, outside intermediate search probes,
  and feeds Manipulation/Settings/Detailed log. search_cropdale minimizes total
  input-equivalent cost across time switches, Eastern return Escape, Hikari
  menu2, Canalbrine Escape and both coast menus, freezing the earlier selected
  Crackridge approach. Cache exact world/RNG states and replay the winner with
  diagnostics. Northern/Western cards show night, formation, boosts/item/Escape
  and day. Source-derived endpoints still await retail validation. See
  docs/conning-coasts.md; do not claim global optimality before Cropdale.

- Canalbrine Bridge water encounter is now authorized after the encounterless
  Ryu -> Northern Ryu -> Bridge continuation. The last upstream control is
  the SECOND command menu of Hikari's recruitment fight, before its ordinary
  unboosted weapon action. src/hikari_manipulation.rs jointly searches that
  menu's boost/cancel cycles and the FIRST water encounter menu's cycles by
  their combined input cost. Require Normal/Preemptive and successful first
  Escape, with no repeat cap; ties prefer fewer Hikari cycles. Both menus have
  one available BP, so each 0->1->0 cycle is two inputs and one CRT draw. Replay
  the complete Hikari victory/recruitment, Scholar setup and shared map loads
  for each distinct Hikari candidate; preserve the selected earlier route.
  Carry the actual party[7,1,8,4], bench[6], all EXP/JP, resources, gear, jobs,
  equipped supports and enemy knowledge. Benched Osvald performs no Study Foe.
  Water zoneENT_FLD_SEA_1_2_WTR atProgress5 selects level11 table030, with no
  fixed first formation; no river-specific CRT offset. Boarding and tutorial6
  consume zero CRT and do not reset the counter (flags1005/1305 remain tracked).
  User has not measured water movement; do not invent distance, decrements or
  a second-encounter guarantee. ryu.hikari_search and canalbrine.water retain
  both adjustments, selection, first-menu/escape checkpoints and field return.
  See docs/canalbrine-water.md and tools/audit-canalbrine-water-encounter.cjs.
  This extension is source-audited and awaits retail validation.

- Continue after Throné's Scholar setup: daytime Ryu -> Northern Ryu Sands
  -> Canalbrine Bridge (Fld_Sea_1_2/Map107). User explicitly guarantees no
  encounter on these legs; do not add an encounter search or ask for movement
  distances. src/canalbrine.rs uses shared Loader.visit_map for both loads,
  preserving party[7,1,8,4], bench[6], all EXP/JP/gear/jobs/supports, Progress5
  and Mother701. Land transitions1130/1001 have no flags or overrides; do not
  use the alternate1003 canoe transition. Bridge has no automatic event or
  tutorial/tavern callback. Map constructor, eligible NPC and real outgoing/
  incoming counter draws are derived from data. Walking adds no CRT calls;
  pre-exit counters are unmeasured and are not invented. Both arrival counters
  are the real incoming reset. Show both CRT checkpoints in Detailed log and
  JSON; no diagnostic-only Manipulation widget. Append only after selecting
  valid Ryu movement and completing Hikari/Scholar, never during cost trials.
  See tools/audit-canalbrine-bridge-loading.cjs and docs/canalbrine-loading.md.

- After Hikari replaces Osvald in Ryu, assign Throné Scholar, learn Lightning
  Bolt (ABI_SET_SCH_030) then Elemental Barrage (ABI_SET_SCH_060), and equip
  Evasive Maneuvers (ABI_SCH_SUP_01). This costs30+100=130JP. Scholar's two
  default abilities count toward the four-skill support unlock. Native job,
  learning, support and menu-close callbacks consume zeroCRT and do not reset
  the counter. src/scholar.rs carries separate assignment/learning/equipment
  checkpoints and real JP/resources. Core jobs helpers apply source subjob
  revisions before flat gear and preserve job bonuses through levelups. Newly
  permitted staff is source zero-stat ITM_EQP_ROD_010; assignment does not heal.
  CharacterJobs and job_licenses are included in exact search keys/exports.
  Count secondary licenses on active AND benched members; Osvald's main job
  consumes none. Guild first TaskClear grants one license and notification100.
  Evasive depends on active equipped support, so Osvald replacement disables
  it until Throné equips it. Tutorial54 follows acquisition;56/57 share gates,
  as do58/59; carry earlier Osvald/guild menu flags. No invented ability IDs
  for Osvald's unspecified two skills. See docs/scholar-assignment.md and the
  native/cooked audit. Manipulation shows all three setup instructions.

- Hikari recruitment is now explicitly authorized, superseding the earlier
  cancellation below. After the selected safe Ryu arrival, src/hikari.rs runs
  MS_KEN_00_0100 YES47 until scripted battle600@66. Event lookup is fixed
  ENM_EVE_WAR_C00_010 -> ENG_EVE_WAR_C00_010 and consumes no field-selection
  CRT rolls. Core routine: first actor regular Ice Soulstone, next actor
  unboosted original default weapon. Carry all four participants' resources,
  enemy knowledge and EXP/JP; only participants receive battle rewards.
  Restore the real field B/A counter before MS_KEN_00_0300; skip NO138,
  join at EXP0/Lv1, select Osvald replacement at3005@148, then apply that
  event's distinct FinishPre B/A and tutorial67. Do not finish the old intro
  event during serial handoff. Full party becomes[7,1,8,4], bench[6]; preserve
  Osvald ledger/skills and disable Evasive while he is benched. Progress5,
  active Mother701 retained. Manipulation shows the swap; Detailed log keeps
  menus, final CRT/counter and all five ledgers. Required movement search is
  unchanged; fixed recruitment runs after each selected valid continuation,
  never on rejected movement candidates. See tools/audit-hikari-recruitment.cjs
  and docs/hikari-recruitment.md. New endpoint awaits retail validation.

- Continue from Southern Oresrush return through Northern Ryu Sands
  (Fld_Dst_1_1/Map19) and Ryu (Twn_Dst_1_1_A/Map28), before Hikari recruitment.
  Northern -> town is padded146 walking /140 running updates (143/137 +3),
  day + Evasive costs584/840. At the Eastern Cropdale return Escape menu,
  replay the COMPLETE continuation and require all three subsequent walking
  budgets: Southern Cropdale approach, Southern return -> Northern Ryu, and
  Northern Ryu -> Ryu. Preserve minimum-input cost order and uncapped required
  search. Ryu entry/counter/CRT belong in Detailed log; Manipulation shows
  location-specific running share. Carry the four-character EXP/JP/gear ledger.
  Source audit covers395 placed BP actors/49 classes and embedded level scripts;
  both maps have no additional environmental runtime CRT draw or tavern event.
  See tools/audit-ryu-loading.cjs. Hikari implementation was explicitly cancelled;
  do not recruit, swap Osvald or simulate the battle in this extension.

- Southern Oresrush Wilds return -> Northern Ryu Sands uses padded130 walking
  /122 running updates (measured127/119 +3). Derive costs from the ACTUAL
  southern_return world after Mother acceptance and Partitio recruitment;
  day + Evasive costs520/732. The last controllable event is Eastern Cropdale
  return Escape: replay all subsequent map/story/recruit callbacks for each
  successful Escape and reject candidates without a safe walking budget on
  either Southern Cropdale -> Southern Oresrush or Southern return -> Ryu.
  Keep cost order and uncapped required search; no synthetic CRT offsets.
  This supersedes the older note treating the first Oresrush approach as
  informational only. Manipulation shows the location-specific run/walk share;
  Detailed log holds counters/costs. Reserve1, round down to0.1%, interpolate
  independently measured densities. The continuation above now loads Northern
  Ryu and Ryu, stopping before Hikari recruitment.

- Continue Southern Cropdale's measured mixed-movement arrival through shared
  Loader to Southern Oresrush Wilds (Fld_Wld_1_3/MapID86), then Oresrush
  (Twn_Wld_1_1_C/MapID100). The Wilds -> town leg is 75 RUNNING updates only;
  no walking measurement is invented. Day + Evasive costs450 and first counter
  is500..899, so it is always safe on this route. Select Throné Mother Ch2
  MS_TOU_01/scenario701 through MapInStorySelect: save confirmation, active
  story7/701 and task-view30510 before playing MS_TOU_2A_0100. Its skipped
  opening sets8050, refreshes the newly eligible ordinary story NPC and rolls
  its real FinishPre counter. No assumed MainMissionPlayingFlag2005 assignment.
  Recruit Partitio ID4 using FIRST invitation MS_SHO_00_0100 YES58 / NO72,
  dismiss tutorials66/69, then visit Southern Oresrush again. Carry party
  [7,6,8,4], original Partitio gear, zero EXP/JP, progress4, Mother701 still
  active, enemy history and all world flags. Entry CRT/counters and callbacks
  belong in Detailed log; Manipulation has the town approach Run100% card.
  These new endpoints are source-derived and await retail validation.
- SkipPrologueReminiscence applies imported MainStory on/off event-flag ranges
  as well as progression points. Replay it for all supported recruited members;
  older Osvald/Agnea additional flags do not change the route's eligible NPCs
  or CRT. StartAddMemberEnd uses global reminiscence guard2900, not per-character
  flags. Partitio greeting7450 uses the top member's FirstJob (Throné opt6).
  Confirmed story IDs are persistent WorldState and exact WorldKey fields.

- Southern Cropdale Trail entry -> Southern Oresrush Wilds uses padded122
  walking /112 running updates (measured119/109 +3 leeway). Derive costs from the carried
  time and Evasive Maneuvers and use the actual Southern entry counter in each
  timing branch. oresrush_reach is movement budgeting, now also required by the
  Ryu continuation search above; it adds no map load. Manipulation displays a location card
  with running/walking distance share; Detailed log keeps numeric budgets.
  Reserve one counter point and round mixed share down to0.1%, following the
  existing interpolation; speed-switch overshoot is not individually simulated.

- Search acceleration preserves complete selected-plan output, not only final
  CRT. FieldSearch keeps direct and post-break boost prefixes with real voice
  history; each Escape check runs on a clone. Eastern search contexts survive
  cost layers, and ordered visitors stop once a continuation is accepted.
  Formation trials suppress only Loader diagnostics and replay the winner with
  its original cost bound for the complete log. Keep this quiet scope after
  Pirro: earlier mansion code currently mutates diagnostic trace entries.
  Required-cleric failures stop before unnecessary day/recruitment callbacks;
  optional targets retain full projection semantics. No retry caps added.
  Exact typed WorldKey replaces JSON dedup keys; Flats projection identity
  includes weakness/Study Foe history, not just CRT. New state fields must be
  included (exhaustive destructuring forces an audit). A full map-result cache
  was benchmarked and removed because key/result copying made searches slower.

- Search profiling: opt-in OT2_PROFILE thread-local inclusive timers and ignored
  release GUI test profile_current_route. Pure boost paths cache common draw
  counts0..256 with exact argument keys; larger counts still run uncapped.
  Zero-BP-break/cancel strategies have only even cost layers (break cost10);
  skip impossible odd Eastern layers before initialization. Preserve serialized
  outputs/candidate order, both overrides and Required outcomes. Current seed0
  title4997 complete response is byte-identical before/after; 3.893s ->2.904s.
  See docs/search-performance.md. Do not confuse cache bounds with retry caps.

- Carry saved main_progress_points and completed_scenarios separately from EXP,
  party size and flags. Completed/skipped MainStory rows contribute imported
  ClearDifficultyPoint once; command3120 NO invokes native SkipPrologueReminiscence
  via AddReminiscenceDialogFinishCallback. Throné completion gives1; Osvald skipped
  rows600/601 add1/0; Agnea skipped row800 adds1, total3. Every route encounter
  uses carried points for tier selection. Eastern Cropdale return crosses the
  progress3 boundary (tier020, danger8): retail Birdian II + Birdian I x2.
  No extra CRT calls belong to point bookkeeping. Do not substitute party size
  or a fixed tier override. EndPrologueReminiscence is NOT the skip-completion
  callback; prior claim that skips contribute no progress was wrong.

- After Agnea, return to Eastern Cropdale Trail (Fld_Fst_1_3), then load
  Southern Cropdale Trail (Fld_Fst_1_4, MapID132). User guarantees exactly one
  encounter on this return leg; do not invent movement counts or enforce a
  second-encounter counter target. Gate no-surprise at the preceding Crackridge
  Escape candidate, after replaying Cropdale/Agnea and the return map load.
  Use the real three-member party [7,6,8], original gear, carried EXP/history,
  Agnea's voice set, and Osvald Study Foe; minimum-input first-menu boost/cancel
  Escape. Southern map entry and its newly rolled counter use shared Loader.
  Show exact enemies/actor/boost/Escape by location; diagnostics in Detailed log.
  New return encounter and Southern endpoint await retail validation.


- Shared environment initialization includes BP_EnvObjMD_Flag_A BeginPlay:
  one CRT draw per actor for animation speed, in level actor-array order.
  Cropdale has four (_5, _6, _8, _9); retail entry6802 instead of old6798.
  Import actual instances; no fixed map offset. Preserve constructor residency
  separately from environment BeginPlay callbacks, as for canoes.


- Continue the active voyage through Cropdale (Twn_Fst_1_1_A, MapID145)
  using shared Loader, then MS_ODO_00_0200: invitation125 YES, chapter134 NO.
  Carry every earlier ledger/history entry; append Agnea (ID8), zero EXP/JP,
  original starting gear and character-specific growth. Recruitment finishes
  its placement refresh/counter reset and tutorials66/73. Cropdale and Agnea
  checkpoints/traces belong in Detailed log, not diagnostic-only Manipulation
  widgets. Source-derived endpoints await retail validation.


- The active Winterbloom fork continues through tagged New Delsta Harbor and
  SC_LinerShip_Cty_to_Wld to Crackridge Harbor, then Eastern Cropdale Trail
  (Fld_Fst_1_3). Protect Crackridge no-surprise at the preceding Western
  controllable fight for BOTH Winterbloom timing endpoints. Use carried party
  levels/history/Evasive, never the old voyage adapter's Osvald level1.
  Crackridge boosts secure first Escape and safe onward travel; measured total
  budgets are172 running /182 walking. No new day switch on an already-day route.
  Crackridge Escape candidates must also load Eastern Cropdale Trail and roll
  a counter permitting the padded147 walking updates to Cropdale (measured144),
  with137 running updates (measured134). Reject successful escapes failing
  either leg. Show a separate Trail entry -> Cropdale running share, using both
  measured densities, reserve1 point and round share down to0.1%.

- Save enemy_knowledge (revealed weapon/element icons plus Study Foe studied flag)
  in WorldState. Every supported field Escape/break/rare/Mug adapter receives
  carried history and returns selected history; rejected candidates must never
  mutate it. Fully known types draw nothing; duplicates share reveals. Western
  second encounter capture validates menu6630 and failed unadjusted Escape; old
  menu6628 omitted two repeat-Study Foe rolls. No compensating RNG offsets.

- Western Snows -> Scholar License -> Winterbloom tag is 345 RUNNING counter
  updates, without padding. Track Evasive Maneuvers from the actual rare victory:
  an earlier rare enables it at Western entry; a Western rare enables it only
  after the first fight. Flee every other planned encounter using carried levels.
  Protect both the predicted second opening and the movement-error override;
  disable "No second encounter" when perfect movement predicts one. Immutable
  alternatives carry actual Escape/field-return RNG into shared Winterbloom load.
  Scholar guild events have no counter reset. MapInStorySelect is an in-place
  event, declined without starting chapter 2. Winterbloom has a tavern lottery
  even with an empty bench (before sub-member selection); don't skip that draw.
  New Western/town endpoints are source-derived and await retail validation.

- Regular Octopuff is NOT a target in Southern/Eastern Cape Cold Snows.
  Cait remains eligible; Western Winterbloom rare targets are unchanged.
  Preserve real Octopuff encounter generation and mandatory no-surprise rules.
  This supersedes earlier notes accepting regular Octopuff in Cape Cold Snows.

- Rare AoE reaction is aggregated after ALL target damage timers, as in
  LibAilment.Battle_ExecDamageHP. Never roll a kill voice per defeated enemy.
  Retail Cultured Cait + Majestic Snow Marmot + Snow Yak (2026-10-02): entry6514,
  Osvald menu6563, victory/drops6572, field return6574. The former6576 endpoint
  was two duplicate kill reaction rolls, not an incorrect field B/A reset.
- Carry character_progression through every map/party transition: EXP,
  earned/spent JP, character-specific growth, original equipment stats and
  resources. Scripted fight rewards seed the ledger; escaping earns no reward.
  Rare rewards include every accompanying enemy, with source battle bonuses.
  Two Osvald skill purchases spend130 JP; derived levels feed onward encounters.

- Search acceleration must preserve candidate ordering, selected button paths,
  complete CRT traces and Required/no-surprise goals. Snows cost layers cache
  immutable Southern continuations; Eastern search evaluates only newly added
  cost layers, and Flats reuses the computed timing alternatives. Never turn
  a performance optimization into an arbitrary retry cap. The uncached-route
  oracle test compares whole serialized results for both recruitment timings.

- Post-Osvald time switching is removed from the active early-clerics route.
  Flats search projects recruitment with zero additional switches; Southern
  encounter branches stay daytime. Do not offer or display Osvald time-switch
  adjustments. The fixed night/day cleric shop pair stays in the route.
  This supersedes earlier notes permitting post-recruitment time manipulation.

- Formation constraints are removed from the active route: do not require one
  enemy, one shield, City Ratkin I/Night Raven, or an unboosted Throné break.
  Ignore legacy saved highroad_break/highroad_flats_break required preferences.
  Select supported formations by actual downstream manipulation cost. Breaking
  remains an optional strategy when the core can simulate it and it is cheaper;
  multi-enemy direct Escape is valid. Never-surprise remains mandatory.
  src/formation_search.rs compares frozen-R3 Pirro R8 futures and Highroad escape
  allocations through real Flats/clerics/Snows continuations. Required outcomes
  have no arbitrary retry cap; pruning uses an existing complete valid route's
  cost. Cost model stays inputs +10/break +7/time switch. Report the baseline and
  selected costs in Detailed log/export. Earlier decisions and unsupported
  battle actions are outside this optimizer; do not claim universal optimality.
  This supersedes older lone-enemy/unboosted-break target notes below.

- Manipulation widgets are grouped by location/visit, never by rare encounter
  status. Southern Cape Cold Snows includes recruitment, its encounter action
  (Escape or Shadow M) and onward movement. Eastern Cape Cold Snows uses one
  widget for either Escape or Shadow M; Western has its own location widget
  only when its encounter is planned. Show the exact selected enemy formation
  in each encounter widget; group duplicates as Name ×N. Preserve location
  headings when the selected rare target changes.

- HARD RULE: every encounter the route expects must be Normal or Preemptive,
  NEVER Surprise. Numeric optional settings cannot relax this rule. Search at
  the preceding controllable event; encounter-menu boosting cannot undo surprise.
  A Southern rare ends rare searching, NOT travel. Carry its real Shadow M
  victory/field reset and Evasive Maneuvers into Southern onward movement,
  Eastern entry, its first encounter and a successful Escape. Show Eastern
  escape instructions and safe onward running even after an earlier rare;
  do not target another Western rare in this branch. Validate all planned
  encounter openings, including downstream openings after rare victories.

- First rare after clerics AND Osvald may be a Cait or Octopuff in Southern
  or Eastern Cape Cold Snows. Otherwise accept an Eastern Escape, full 167-run
  update traversal without a second encounter, and a non-surprise rare in
  Western Winterbloom Snows (Fld_Snw_2_2, MapID155). These are alternatives:
  never search for a Western rare after an earlier rare is secured.
  Joint Southern/Eastern search allocates actual boost/break input costs.
  Every selected rare formation must die to one Shadow Soulstone (M); reject
  packs with survivors. Core handles item callbacks, one AoE voice, victory,
  and drops; shared Loader handles field reset. After the kill, Osvald learns
  two extra skills and equips Evasive Maneuvers; carried world support is 50%
  for subsequent movement, never before the kill. Source-derived new endpoints
  await retail validation. This supersedes the Cait-only rules below.

- Cait target accepts either a non-surprise Southern Cape Cold Snows Cait
  AFTER Osvald recruitment (allow the real safe 70-update walking approach),
  or the first Eastern Cape Cold Snows Cait. Preserve existing required and
  achieved upstream targets. Compare Southern upstream adjustments against
  the existing Eastern plan including its Southern escape adjustment cost.
  A selected Southern Cait stops at its encounter composition: do not flee
  it or invent Cait combat/after-battle RNG. Show Walk/Run to Osvald explicitly.
  The older Eastern-only requirement below is superseded by this rule.

- Keep every simulated action widget visible in Manipulation even when zero
  extra boosts are needed. Explicitly write "No extra boost adjustment needed";
  never hide a battle/escape widget based on adjustment cost. Keep unavailable
  future events and diagnostic-only data separate as before.

- Early route now includes Gold Ring from the Backstreets Brothel Receptionist
  (80% rand1..100 check) and Boy's guaranteed Antique Coin in New Delsta main
  immediately after Artist's three soulstones. Each new menu visit consumes
  open/check/closing-dialogue draws. Compare ring-before-clothes and
  ring-after-clothes from immutable initial world/RNG; prefer a successful order
  with a non-surprise mansion opening, then before on equal valid cost. The
  second-fight boost search's mansion_approach target now requires BOTH ring
  success and non-surprise, preserving first-fight inputs/achieved fight-two
  conditions and minimizing extra button inputs. Existing Required/numeric
  settings and replay/export carry the joint target. UI shows selected theft
  order and Antique Coin after soulstones; diagnostics include both checkpoints.
  Sale valuables remain in mansion.acquired_items and are excluded from Pirro's
  combat-item catalog. Recompute all downstream events; never add compensating
  offsets. Old retail snapshots omit these two steals and are tested separately.
  New theft route endpoints await retail verification.

- Southern Cape Cold Snows Escape search now REQUIRES a Cait and no surprise
  in the first Eastern Cape Cold Snows encounter. Search by actual button cost
  plus 10 for zero-BP breaks, propagating the actual Escape return, recruitment
  when needed, padded onward movement and shared map load for every candidate.
  Preserve safe onward movement and first-Escape success; no arbitrary retry cap.
  Both timing overrides receive their own complete valid continuation. Project
  Eastern land encounter with party levels [Throne route level,1], progress1,
  carried time/FirstEncount/counter and no intervening CRT actions. Current
  day/night tables have FirstEncount index0 (no fixed group) and Cait rate2%.
  Keep boost instructions in Manipulation and encounter diagnostics/CRT in
  Detailed log. Eastern composition endpoint is before battle initialization;
  no Cait battle simulation or second-encounter guarantee is added here.

- First Southern Cape Cold Snows encounter now has a real flee continuation in
  src/snows_escape.rs. Compare direct first-menu boost/cancel Escape against a
  successful ZERO-BP weapon break and Escape, ordered by inputs +10 per break.
  Carry actual party/menu actors and core snapshots; no failed-Escape trials
  leak into execution. Before Osvald: restore field, run the remaining approach,
  recruit with the actual post-battle FirstEncount=false reset (full counter).
  After Osvald: first Escape must leave a safe full remaining run, including
  night costs. Timing controls select immutable complete branch results.
  User corrected the onward destination to EASTERN Cape Cold Snows (Fld_Snw_1_3),
  not Winterbloom; retain measured 85 run /90 walk updates from Osvald, plus
  two updates of player-error allowance per speed (87 run /92 walk). For a
  post-Osvald encounter, subtract its measured running distance and convert
  the remaining walk distance before adding the full two updates once. The
  Escape search and running-limit calculation must use the same padded budget.
  Pre-Osvald movement stays at 64 running updates with no allowance.
  Before-encounter onward mixed movement interpolates measured densities,
  rounds running share down and reserves one counter point. Its Eastern entry
  uses shared Loader with imported MapID153, no fixed load offset. Instructions
  stay in Manipulation; entry CRT/counter and full trace stay in Detailed log.
  Supersedes old notes that prediction stops before this encounter's battle.
  New Snows/Eastern endpoints are source-derived and await retail validation.

- Post-cleric time manipulation is removed: after buying, always switch ONCE
  from night back to day with shared time/bar/NPC callbacks. The weighted
  Flats search enumerates battle boosts/breaks and post-Osvald switches only.
  Supersedes older notes permitting extra post-purchase cycles.

- Snows routing supersedes the old safe-Osvald-walk goal: run 64 counter
  updates from Southern Cape Cold Snows entry to Osvald, NO leeway. Predict
  pre-recruitment timing when entry counter <=64*12. If inevitable before,
  search only solo Throné's opening for no surprise. Otherwise protect both
  pre-recruitment (movement-error override) and post-recruitment party openings.
  GUI has Before/After Osvald controls backed by immutable branch endpoints,
  preserving the Flats boost plan. After is disabled when perfect movement
  predicts Before; stale overrides cannot select that impossible timing.
  Recruitment's actual B/A reset advances CRT twice and supplies the later
  branch counter; the solo branch starts before those calls. After-Osvald time switches affect only the
  later branch. Old flats_osvald cap is ignored in this default fork; retained
  require_cait API and full route keep their original safe-walk policies.
  Prediction stops at first encounter before battle init; do not propagate
  the hypothetical no-prior-encounter recruitment state as an actual state
  after a pre-Osvald battle without simulating that battle and return callbacks.

- Attack display uses only the weapon/skill name, prefixed by BP spent when
  positive: Dagger, +1 Dagger, +3 Darkest Night. Each command menu has one
  instruction box containing the boost adjustment followed immediately by the
  action (including Escape). Preserve Pirro R3's explicit no-adjustment notice.

- Escape strategy cost model supersedes threshold-first fallback: one boost
  BUTTON press=1, one successful break action=10, one time switch=7. Mansion,
  Highroad and Flats compare complete valid plans in increasing estimated cost;
  numeric legacy strategy caps enable breaking, None/Boost only excludes it.
  GUI hides obsolete strategy threshold numbers and migrates prior Boost only
  preferences once because the user explicitly enabled breaking for this route.
  Outcome numeric/Required settings remain independent and retain semantics.
  src/escape_search.rs enumerates R1 direct Escape or R1 actual break plus R2
  boosts, preserving BP, voice history, actual hit and conditional battle draws.
  No BP spent at R1 means 2BP at R2; spending initial 1BP means no R2 BP.
  Shared core opening_break_menu/FieldBreakMenu clones the actual R2 battle;
  no synthetic RNG offset. Mansion uses its separate real battle snapshot.
  early search cost includes break and BOTH menus' inputs plus all time toggles.
  Required goals still gate candidates before optional searches.
- New Pirro R8 highroad_break_max_boost_repeats Required/numeric target selects
  a lone City Ratkin I/Night Raven with an actual successful 0BP dagger break
  available on Highroad. R3/victory/no-grape are frozen; joint Required R8 goals
  precede optional additions, which preserve achieved goals. GUI defaults this
  Required; API default0 preserves old target selection. Replay/export preserves
  the exact R8 path. Targets establish break availability; fastest Escape may
  still use direct Escape or a 1BP break when cheaper for downstream outcomes.
  Supersedes prior notes saying R2 break manipulation is missing.

- Highroad boost search can target the next Flats encounter's real unboosted
  opening break. highroad_flats_break_max_boost_repeats=None means Required;
  numeric limits are optional extra input pairs preserving required/achieved
  harbor walking and non-surprise outcomes. Backend compatibility default0;
  fork GUI defaults Required. Rebuild both field loads and actual rare-replaced
  composition, then replay core first_action_break: only a lone City Ratkin I
  or Night Raven with a successful 0BP dagger hit qualifies. Surprise, multiple
  enemies and a missed Night Raven hit fail. This is an upstream target, not
  automatic promotion of Flats Escape's saved Boost only setting, and does not
  yet add a separate R2 post-break boost search. Keep diagnostic success/composition
  in Detailed log; Manipulation displays changed Highroad inputs only.

- User suspended Cait manipulation. Default early-route goal is any first Snows
  encounter WITHOUT surprise; preserve clerics, safe village run, safe Flats
  movement and safe daytime Osvald recruitment. GUI uses osvald_encounter and
  early_approach_max_boost_repeats; old endpoint/budget names are aliases and
  stored numeric/Required preferences survive. early_require_cait=true is an
  API-only opt-in for retained Cait tests. Do not silently require Cait or drop
  the non-surprise goal. Time controls and weighted optimization remain active.

- Early Cait fork now permits post-purchase time cycles (end DAY) and post-Osvald
  toggles (end either DAY or NIGHT; user confirmed both). Minimize extra boost
  inputs +7 per actual toggle, including the constant two shop toggles. Weighted
  cost layers compare all allocations across these control points; Required
  goals gate every candidate. Immutable route projections are cached. Numeric
  optional allowances keep their two-input-unit/pair scale but now bound extra
  WEIGHTED cost (4 =8 units), not merely boost repeats. Settings are separated
  into Flats encounter, After cleric purchases and After recruiting Osvald.
  Replay every shared time lottery/bar/NPC refresh. Toggles preserve counter and
  FirstEncount; final encounter uses the actual DayTime/Night volume and movement
  decrement. Separate post-recruitment trace precedes encounter checkpoints;
  offset local encounter counts from after_osvald_switches exactly once.
  Old fixed-phase 4807-repeat prediction is superseded by weighted search.

- Active branch: `strategy/abandoned-village-fork`. GUI now selects
  `route_end=osvald_cait`: first village tag/return -> Flats night near clerics
  -> buy soulstones -> immediate DAY -> Southern Cape Cold Snows -> recruit
  Osvald, skip story/tutorials -> first land encounter IN Snows. No Cape Cold.
  `src/early_cait.rs` replays shared Loader callbacks and imported encounter
  lottery. Search at Flats Escape jointly requires safe village run, clerics,
  encounter-free 72 walking/68 running Flats updates, 70 walking updates to
  Osvald and an actual Cait replacement with Normal/Preemptive opening. Each
  target has independent Required/numeric optional policy across the three
  control points. Numeric misses retain required/achieved outcomes. Shop
  time switches do not reset the counter; shop location only splits the safe
  total movement budget. Confirmed same total distances, day after buying.
  No rare equipment/support bonuses; party actual Throné route level + Lv1
  Osvald. Endpoint is composition finalized BEFORE enemy actor initialization.
  No Cait battle simulation yet. Source-derived, awaiting live validation.
  See docs/early-clerics-cait.md. FirstVillage and Full APIs remain compatible.

- Previous fork boundary (still accessible via API): `main` preserves the complete
  previous strategy. This fork's GUI sends `route_end=first_village` and stops
  after the FIRST Abandoned Village entry/tag and tutorial dismissal, before
  returning to Flats. Flats Escape search requires escape and a safe village
  run only; it must not gate on Snows, Osvald, Mug or later outcomes. Shared
  Loader carries the endpoint's CRT and world state in `flats_escape.village_visit`.
  Future settings are hidden without deleting stored preferences. Full-route
  APIs remain available with `route_end=full` (the API compatibility default).
  The following older notes describe the preserved full strategy.

- Crackridge first encounter is IN Fld_Wld_1_2 (Crackridge Harbor: Anchorage),
  ENT_FLD_WLD_1_2_LND, daytime progress1/danger5. User confirmed location.
  crackridge::plan first compares day switching in Flats after cleric purchases
  before fast travel versus Harbor after travel, preferring established Harbor
  if safe. Only if both Surprise, replay Mug R2/Cape Cold weighted search jointly
  for clerics and no surprise. Earlier plans are frozen. New independent
  crackridge_approach_max_boost_repeats defaults Required; numeric optional
  ceiling adds2 weighted input units per repeat above clerics baseline. Core
  new_party uses actual Throné route level + Lv1 Osvald, first traveler and
  Study Foe per DISTINCT enemy type. Search minimal boost cycles for first
  successful Escape (mandatory), restore field with shared Loader. Actions
  only in Manipulation, CRT checkpoints/trace in Detailed log. Source-derived,
  not retail-validated. See docs/crackridge-voyage.md and core party audit.

- Added crackridge::after_market: ordinary cleric purchases -> fast travel to
  tagged New Delsta Harbor at night -> immediate day switch with shared NPC
  refresh -> SC_LinerShip_Cty_to_Wld (MapID189 ocean Fld_Ocn_1_1) -> cmd500
  MapID85 Fld_Wld_1_2 -> FinishPre counter B/A (EncountReset=true). Importer
  includes both new maps, placements, streamed levels and event commands.
  Source audit tools/audit-crackridge-voyage.cjs and docs/crackridge-voyage.md.
  GUI appends crackridge_voyage after selected black_market route and shows
  its endpoint/checkpoints in Detailed log only. No fixed draw offsets. Assumes
  skipped voyage, sufficient money, no encounter/path actions. Awaiting retail
  validation. Shop transactions do not track inventory/gold in WorldState.

- Cleric approach measured40 counter updates at either speed from daytime
  Flats entry (from Abandoned Village) to the cleric spawns. black_market::Route
  exports cleric_reach and at_clerics; use42 updates (40 measured +2 player-error
  leeway), zone-derived walk9/run12 =>378/504 cost. Mixed budget counts whole
  running updates, reserves1point.
  Carry the walking reference counter through night switch; safe mixtures share
  identical CRT. Manipulation shows start/destination, run/walk limit and night
  switch instruction; diagnostics in Detailed log. Supersedes unspecified-walk
  notes below. Pirro Manipulation no longer displays early/late latent labels.

- Repeat budget unit changed: one repeat = cancel/reboost pair, TWO extra
  button presses beyond normal action boosting. extra_repeats=(len-finalBP)/2.
  Larger climbs get no budget discount. All shared capped searches and Pirro's
  conditional-voice BFS use this unit. Minimum-input paths prefer single-level
  repeated boosts before minimizing instruction lines. For15 total CRT calls,
  finalBP2: [1,2,1,2,1,2,1,2,1,2,1,2,1,2] =14 presses,6 extra pairs.
  Numeric defaults stay4 (=8 extra presses), so capped choices may change.
  Settings explain this definition. This supersedes all old climb-count notes.


- Clerics search now jointly enumerates Mug R2 boost inputs and post-Mug Cape
  Cold time switches. Objective: button presses +7 per individual time switch,
  INCLUDING village day/final Flats night switches. Odd Cape switch count ends
  daytime: skip village time change AND its placement callback entirely.
  Shared loader executes each toggle and NPC refresh; never fixed CRT offsets.
  Search increasing weighted-cost layers, prefer fewer Cape switches on ties.
  Numeric cleric setting caps boost repeats only; rotations remain candidates.
  Entry6154 now selects +2 boost draws [1,2,1,2], one Cape switch to day,
  Mug6231 -> final night6268. Current title4916 is separately verified via GUI.
  Source-derived repeated-time-switch endpoints await retail validation.


- Mansion policies are separately configurable: mansion_run_max_boost_repeats
  is Required(None) by default or a numeric optional counter-search budget.
  mansion_escape_max_boost_repeats is additional direct-Escape boost budget
  beyond the counter plan (default4), then dagger break; None means Boost only.
  Both strategies preserve achieved/required counter safety. Optional safety
  miss exports reaches_pirro=false and stops downstream Pirro prediction.
  Removed automatic prerequisite-budget promotion: later Required choices must
  never silently rewrite other user settings. Current defaults still solve the
  seed0/title4916 full route. Settings expose independent controls; JSON exports
  both settings. Tests cover required/optional, strategy caps, break replay and
  unsafe optional escape blocking downstream prediction.

- Boost path tie-break: shared optimal_inputs/with_cap minimize button presses,
  then grouped instruction lines on the shortest-path graph. Render exact chosen
  path; do not reorder labels alone. Mug +7 now displays 0→1 ×2, 1→2 ×4.


- Black-market route added in src/black_market.rs: Mug R2 explicit boost inputs,
  fast travel to tagged Abandoned Village, switch DAY, enter Flats, then switch
  NIGHT near the clerics. Shared Loader evaluates map/time/NPC callbacks;
  target flag25503 AND both cleric placements0900/1000. Both time switches reroll.
  No approach distance given: assume no intervening encounter and do not treat
  carried counter as remaining walking budget. Source-derived, not live-validated.
  Independent mug_clerics_max_boost_repeats defaults Required; numeric optional.
  Search replays entire Mug and route, minimizes total button inputs, preserves
  upstream outcomes, updates the conditional R3 finisher. Baseline entry6154:
  +7 draws via [1,0,1,2,1,2,1,2,1,2], Mug ends6236, final night6268.
  GUI shows boost adjustment only; full checkpoints/trace in Detailed log.
  See docs/black-market.md and data/source/black-market-fast-travel-audit.json.

- Mug completion fixed and live-validated to6299/0xDC86178D. R3 staff hit
  triggers kill voice6293 and delayed JustBreak timer6294 even on a kill.
  Drop6296 -> battle end6297 -> field B/A6299. Osvald attack voice duration
  is5689 ticks, shorter than the9600-tick hit. Tracer detached/registers restored.
  Fixture cape-cold-mug-completion-live.json retains134 continuous calls.


- Enemy initialization now uses imported initial_idle_frames + sync flags, not
  a Mug exception. NPC SnwSm001_Mle lacks BAT_IDLE_ENEMY130; RandomInteger(0)
  makes no CRT call. See OT2BattleEngineRust/docs/enemy-initialization.md.

- Resolved Mug initialization discrepancy: Cape Cold NPC uses BP + timing draws,
  without the ordinary enemy sprite-frame draw. Core derives this from the initial resource and synchronization flags. Replay2 captured 181 calls including reload; last
  continuous segment6154..6191 validates night through first menu6192.
  Tracer detached with original debug registers restored. Compact fixture
  cape-cold-mug-first-menu-live.json preserves callsites and raw-trace SHA256.
  Later battle actions remain source-derived, not live-validated.

- Cape Cold Mug prediction: src/cape_cold_mug.rs and standalone core `mug`.
  User confirmed seed0+6154 on daytime arrival and original-gear Lv1 Osvald.
  Shared set_time(4) MUST be followed by refresh_npcs: black-market1 plus
  eight night placement frames =>6163. Preview timer1 and Rob opening-line
  lottery1 =>6165 battle initialization. Both before-line events and success
  event have EncountReset=false. First menu6192 retail-validated. Later source prediction: menus6218/6289,
  damage53/226/11, victory/drop6297, field B/A return6299 (0xDC86178D). GUI now appends post_battle_field.cape_cold_mug after selected
  Cape Cold arrival via after_cape_cold (no endpoint if town is not reached).
  Detailed log has damage, command-menu checkpoints, final CRT and merged
  chronological field/battle trace. Core Mug counts already include route
  offset: add only crt_setup.battle_entry_advances when displaying totals.
  Manipulation shows only the R3 staff finisher when needed, or an error.
  See docs/cape-cold-mug.md and tools/audit-cape-cold-mug.cjs. General
  per-character EXP/JP ledger absent; current route computes Throné EXP only.

- Manipulation UI rule (2026-09-29): show only instructions that change the
  player's actions for this prediction: actual boost adjustments, break/Escape
  alternatives, early/late latent, and run/walk limits. Hide event cards when
  no adjustment is needed. RNG states/counts, counters, damage-less/encounter
  reports, and other status-only cards belong in Detailed log. Keep route
  events and calculations intact; card visibility must not skip simulation.
  Required-outcome failures must remain visible so an invalid plan cannot look
  complete. This supersedes older Manipulation diagnostic-card instructions.

- Continue completed Osvald recruitment with src/cape_cold.rs to Cape Cold
  (Twn_Snw_1_2_A, map169), carrying party[7,6], skipped-story flags and CRT.
  User measured52 walking/49 running updates; add2 to BOTH =>54*9=486 walking,
  51*12=612 running. Reserve1, interpolate route-distance cost486+126*f,
  round share down0.1% and cost up; label estimate, not exact switch timing.
  All first counters>=500 can walk, so this adds no boost-search target.
  Current title4942/one Flats cycle: post-recruit6132/counter552 -> run51.5%,
  walk48.5% -> Cape Cold6154/0xA3471B82. Shared Loader evaluates5 levels,
  13 eligible NPC frames and4 counter draws; no fixed+22. BarEventLabel=None,
  no additional CRT BeginPlay in actor/parent audit. Endpoint source-derived,
  awaiting retail validation. Expose post_battle_field.cape_cold separately
  after selected Flats plan; do not run extra town loads during boost search.
  Compact running/entryCRT card, full trace/checkpoints in Detailed log.
  See docs/cape-cold.md and tools/audit-capecold.cjs.

- Extend village_return through src/osvald.rs: load Fld_Snw_1_2 with shared
  Loader, then 70 measured walking counter updates to Osvald (daytime9, require
  631). User confirmed help/recruit, skip introductory story, keep him active,
  dismiss tutorials. Execute imported MS_GAK_00_0100 with choices15Yes/61Yes/
  70No; FinishPre refreshes placements and B/A, tutorials66/71 add no draws.
  Carry party[7,6], flags and native integer event variables (7410 add,7420 ==).
  No running measurement, no extra padding, no hypothetical endpoint if unsafe.
  flats_osvald_max_boost_repeats is independent, defaults Required, under the
  Flats encounter control point. Jointly satisfy Escape/village run/both walks
  before optional budgets, gate breaks too and retain achieved prior targets.
  Compact recruitment CRT + safe walk in Manipulation; full callbacks in log.
  2026-09-29 loader fix: package lookup must be case-insensitive like UE FName.
  The cooked Fld_Snw_1_2_sound was silently omitted by an exact _Sound lookup.
  Live inventory confirms seven new levels, three NPCs and retained shared level;
  ec60c0 -> 11c0100 consumes its LevelColor CRT draw before incoming counter B/A.
  Six-cycle route now matches reported Snows6135 (arrival6121, +14). This is
  endpoint/inventory validation, not a full per-call capture. Re-optimized current
  seed0/title4942 route needs ONE Flats cycle: Snows6130/counter727, 70 walking
  updates leave97, recruitment6132/0x0DF53D34. Keep measured distance70 unchanged;
  old one-cycle counter627/69-only conclusion was based on the missing call.
  Recruitment remains source-derived, not live-validated.
  See docs/osvald-recruitment.md and tools/audit-osvald-recruit.cjs.

- Extend Flats Escape through the Abandoned Village tag/return using
  src/village_return.rs and shared Loader (Twn_Cty_1_2_A -> Fld_Cty_1_2).
  Returned Flats has FirstEncount=true; never reuse Escape's normal counter.
  User measured 70 walk / 66 run updates to Southern Cape Cold Snows; add two
  safety updates to each: 72*9=648 walk, 68*12=816 run, requiring 649/817.
  Estimate running DISTANCE with cost=648+168*f, round share down/cost up,
  leave one point, label estimate. End before loading Snows. The same Flats
  encounter boost menu controls Escape, safe village run AND post-return walk.
  flats_snow_max_boost_repeats defaults Required (None), or optional additional
  repeats beyond the actual Escape/village plan. Every direct/break candidate
  must preserve required targets. Preserve earlier route plans. Show a compact
  post-village walking/running card with returned CRT; full callbacks in log.
  Source audit is tools/audit-abandoned-village.cjs. No new BeginPlay CRT calls;
  first-town fast-travel tutorial 62 sets flags without RNG/counter reset, bar
  event None. Endpoints await live validation. Baseline uniform-state odds:
  72.1550324466% for the padded walk; 78.4661458805% for measured70. See
  docs/abandoned-village-return.md and examples/village_snow_odds.rs.

- Search policy update (2026-09-29, user approved): outcome checkboxes are
  **Required outcome**, not Unlimited. Numeric 0..256 budgets remain optional
  improvements. Solve all required targets and their prerequisites together
  at the same control point BEFORE optional improvements; preserve required
  outcomes if an optional budget fails. A required second fight includes safe
  checkpoint walking; required Pirro no-grape includes victory; required Flats
  non-surprise includes safe Harbor walking. Escape strategy thresholds are
  separate: **Boost only** disables the break preference, while a numeric
  threshold tries a complete valid break plan before further direct boosts.
  Every strategy must meet all required route targets. Preserve old saved
  `unlimited` values/API aliases as required (or boost-only for strategies).
  Report unreachable required outcomes explicitly. This supersedes older
  Unlimited/finite-prerequisite descriptions below. See docs/route-searches.md.

- Group Settings by the last controllable event: First battle (boost inputs for
  checkpoint avoidance and the one-enemy-action second fight), Second fight
  (boost inputs for a non-Surprise first mansion encounter), Mansion encounter
  (boost inputs before Escape for first-try success and the remaining run).
  Each card names the searched actions; retain independent target caps and
  Unlimited options. The mansion-approach cap defaults to 4. Searches propagate
  forward from a frozen earlier plan; never spend the first-fight budget to
  solve the mansion opening. src/second_manipulation.rs replays candidate inputs
  through the independent battle core and all town/story/steal/encounter events.
  Preserve any achieved one-enemy-action result, minimize extra button presses,
  then CRT draws, then choose the latest menu. Normal and Preemptive pass;
  Surprise fails. Carry the selected second boost plan into GUI instructions,
  checkpoints, export/replay and the subsequent Escape search. Keep an explicit
  limit-failure fallback. Hide the redundant successful Escape status sentence.

- This repository owns the speedrun assistant, field encounters, route state,
  local GUI, and prediction exports.
- Import the sibling `OT2BattleEngineRust` library through Cargo. Never copy its
  battle implementation or bundled battle data here. The dependency is one-way.
- The battle repository must continue to build and run independently for sweeps.
- All simulation is headless and tracks the main CRT stream only. The viewer is
  a separate inspection layer; no game graphics/audio runtime belongs here.
- Opening GUI/API inputs are at TITLE SCREEN before New Game as Throné, with
  both opening cutscenes skipped. Add the captured 12 CRT draws, or 13 when
  Btl_SinExpose is recreated. This transition belongs to the assistant; the
  battle library still accepts the state immediately before initialization.
  Never add only the optional shared draw or count the transition twice.
- Keep RNG boundaries explicit. Field predictions end after enemy composition,
  before enemy actor initialization, turn order and command-menu voice draws.
- Prefer source-derived rules and captured regression fixtures. Missing data
  must report an error rather than silently approximate an encounter.
- The supported fresh-new-game Throné post-battle frame resolves its map, zone,
  night period, progress 0, FirstEncount, three-member party and flag 25304 off.
  It is valid after MS_TOU_10_0300 and before MS_TOU_10_0400. Never confuse task
  progress 30 with global main progress or draw an extra initial counter reset.
- Other map membership/story flags remain caller inputs until supported.
- Town-loading evaluates persistent world state in src/world.rs, not fixed
  per-map offsets. The fresh Throné initializer follows actual event branches;
  do not union flags from unexecuted branches. Black Market is a time-change flag lottery, not an extra
  draw on every map load. All 33 FlagRandomControlDB rules, callback gates/order,
  party/bench, NPC state and resident levels matter for general support. See
  docs/town-loading-audit.md. Imported data covers the sewer and New Delsta;
  unsupported maps/commands/tavern branches must fail. Keep the live +110 only
  as a regression result, never as a runtime offset or pre-counted stage list.
- The measured post-opening walk is 830 -> 146: 684 points = 57 night updates.
  Repeated user measurements confirm 57 walking updates. Running currently
  estimates the same count, costing 912 (16 per update), requiring 913; this
  is not a measured running minimum. The first counter max is 899. Target
  walking (counter >= 685) when searching boost cycles to avoid encounters.
  For mixed movement, replace at most min(57, floor((counter - 685) / 4))
  walking updates with running, only when counter >= 685. This is an estimate
  pending separate opening running/mixed measurements. UpdatePlayerStep discards
  excess displacement when assigning the current position to PrePlayerPos;
  speed and switch locations can change update counts. Percentages refer to
  assumed updates, not measured distance/time; round down and retain a positive
  counter. Never apply the Flats 61/56 ratio to another route.
  Re-simulate from the chosen player menu through both resets; never model
  battle manipulation by offsetting the final RNG without replaying combat.
  Field encounter forecasts stop at the cutscene trigger if reached before an
  encounter. Opening results separately project MS_TOU_10_0400: two new NPC
  frame draws, then counter B/A, no extra tutorial reset. Mark this +4 projection
  as source-derived/awaiting live validation. Do not project it if walking is
  unsafe, and do not reuse the earlier encounter zone after flag 25304 is set.
- From that reachable checkpoint, project the first guard's Ambush with one
  normal path-action menu opening (+1 CRT timer draw), scripted dialogue (+0),
  0410 counter B/A (+2), then skipped rejoin event 0500 counter B/A (+2).
  Keep the +5 projection source-derived pending live trace validation. The
  final counter comes from the second pair, guests 15/16 rejoin, 8004 is set,
  25304 clears and the regular encounter actor is restored. Do not mutate or
  replace the earlier post-battle field frame; expose separate event boundaries.
- Assess enemy-free opening victories from actual enemy turn reports, including
  misses. Do not change the fixed targeting routine or rely only on order slots.
- Manipulation combines both battles into one compact run view. The
  Run simulation action always searches and applies the optimal first-fight
  boost adjustment for a safe checkpoint walk, then seeks a second-fight victory
  with all three order conditions met and exactly one enemy action. Settings has
  a separate saved field for each EVENT: encounter avoidance and the second
  fight, each defaulting to 4. Never use one global cap for all events or reset
  a cap per action. Establish the cheapest encounter-dodge plan within its own
  cap; report no solution if none fits. Fight two may add its independently
  configured number beyond the actual dodge cost, not the unused allowance, summed over
  the plan. If no combined plan fits, retain the encounter-dodge plan and label
  the skipped second-fight target. Normal action boosts are excluded; 0->2 x5
  has four extra repeats. Count climbs, not individual BP presses or CRT draws.
  Optimize inputs under the repetition constraint; a longer path may use fewer
  repeats. Preserve the selected plan's repeat_cap in replay/export queries.
  Each event has its own Unlimited checkbox, saved independently. Use None/null
  for unlimited, never a large numeric cap. Finite settings accept 0..256.
  Unlimited searches grow past the initial 256-draw window until an optimal
  match is found or all CRT offsets are exhausted; preserve fewest-input ranking.
  A full CRT cycle is 2^32 draws. Capped searches cover all paths permitted by
  their repeat allowance. Exports must preserve unlimited settings and any
  selected plan beyond 256 draws. Keep the previous numeric value when toggling
  Unlimited off. Migrate the legacy
  single setting to fight two only. Replay the
  full route with candidate-specific EXP, levels and resources. A safe first
  counter alone is insufficient for the combined target, but is valid fallback.
  Apply this search on initial page load too. If no adjustment is needed,
  keep the zero-extra-boost plan. Never switch this button to a baseline run or
  require a separate Find adjustment click. Exports replay the selected plan.
  The First battle card contains damage-less status, boost inputs, then popup CRT and
  counter as small diagnostics. Safe running limit has checkpoint-popup CRT
  as secondary information. The final card shows whether the second fight has
  only one enemy action, action/hit counts, and CRT after the fight (after drops).
  The requested order conditions are R1 Throné before right grunt, R1 Scaracci
  before left grunt, R2 Throné before leader; expose individual checks in Detailed
  log and count actual enemy actions, including misses. Double Thrust's two
  damage hits still count as one enemy action. Probability estimates must specify
  their seed population and must not imply route-conditioned odds from uniform
  second-battle entry samples. Do not add separate battle-exit, checkpoint-arrival,
  Ambush/rejoin or second-entry CRT widgets. Keep callback traces, checkpoints,
  action history, resource tables and long explanations in Detailed log.
  Damage-less means victory with zero damage received; an enemy miss may still
  be damage-less even though the separate no-enemy-actions check fails.
  Order the run view by event timing: damage-less, dodge inputs, popup state
  and counter, movement budget and checkpoint state, second-fight exit. Seed advance counts are
  primary; hex states secondary. Show the whole optimized boost routine,
  including normal final boost, as grouped climbs (e.g. 0->2 x2), one per line.
  Label each adjustment with round, actor and command-menu ordinal within that
  round (e.g. R2 · Scaracci · 2nd command menu). Count actual player menus,
  including Defend; exclude enemy actions and restart the count each round.
  Omit x1 and the Extra label. Show explicit reductions when needed and the
  exact action/target afterward. Repeated climbs return to their starting BP
  between repetitions, ending at the last destination. Optimize total added button presses, counting reductions
  and normal final boost; then fewer extra CRT draws, then the latest menu.
  Replay the complete chosen input path in the battle core without adding the
  automatic normal boost a second time. Respect available BP (opening max 2).
  Keep the run view free of repeated setup summaries and explanatory prose.
  Display and copy CRT states in hex only across all viewer screens; keep
  advance counts, damage and encounter counters in ordinary decimal notation.
- Second Chapter 1 battle ENG_EVE_THI_C01_020 follows skipped 0600 after rejoin.
  Assume the user-specified short walk is encounter-free. 0600 starts battle
  inside its rich sequence, without FinishPre: +0 before initialization, NOT
  +2 from its EncountReset flag. The serial/skip handoff stops before cleanup.
  Keep event cleanup separate from battle entry; see data/source/throne-second.json.
  src/second.rs supplies EXP/level/inventory and calls the independent battle
  core. Default party is level 2; a level-up restores HP/SP. Its state and trace
  counts are local to second entry; entry/after route counts are relative to
  first entry, so add second.entry.draws exactly once in the viewer. End after
  drops without inventing field-return resets. Keep second-fight logs in Detailed log.
- After a completed second victory, expose a separate source-derived city
  projection (never change the battle's after-drops boundary). Skip 0700 and
  0800, load New Delsta main then backstreets, dismiss radar tutorial 7:
  +110 CRT, matched by the 2026-09-27 live capture. Includes 11 streamed-level
  draws, 59+26 NPC frames, and seven B/A counter pairs. The backstreets count
  includes the off-map Epilogue_PLAYER_IS_Throne NPC enabled by flag 2206;
  never infer its eligibility from its name. Keep the shared level resident;
  Night->Night and the popup add no draws. See docs/throne-city.md and its
  reproducible source audit. Add only a compact expected post-city-popup
  diagnostic to Manipulation, with the full breakdown in Detailed log.
- The 2026-09-27 live capture resolves the city discrepancy: second drops at
  5264, city +110, radar popup dismissed at 5374 (seed 0). The earlier 5270
  battle exit was wrong. Core fixes gate startup voice on BattleStartEvent,
  use AI ordinary slot 1/charged slot 0 and account for Darkest Night reaction
  rolls. Preserve live fixtures and do not add a fixed minus-six correction.
  All second-battle calls/menus and city caller categories match this capture;
  other route/save states remain source-derived predictions.
- Parlor continuation is: enter parlor -> first cutscene -> run to next trigger
  -> Yes -> two further cutscenes -> New Delsta backstreets outside the entrance.
  src/parlor.rs carries city.after_popup and city.world_after into the same
  world Loader. Import/run 0900, explicit Yes branch of 0910, then 1000/1100.
  0910 has EncountReset=false; 1000 changes Night to Day, invoking the Black
  Market lottery, and FinishPre occurs before its serial 1100 transition.
  Use town-loading for both maps, never fixed +50 or pre-counted NPCs/levels.
  ed9bc0 unloads the outgoing field AND BattleMapNameList even if the next
  field uses the same Btl_Twn_Cty_B. Only the separately held Btl_SinExpose is
  retained. The live replay matches every one of 50 calls and 24 NPC identities,
  5374 -> 5424 / 0x080E80B0. Keep these checkpoints/callbacks in Detailed log;
  Keep mansion entry in Detailed log, without a Manipulation card. Preserve the resulting world state.
- The next parlor visit carries that state through 1200 -> serial 1300 map
  arrival, fixed dialogue and Steal tutorial 31 dismissal. Scripted Day->Dusk refreshes
  existing placements before assigning time, ages timers by one, and does not
  dispatch the Black Market lottery. Flag 8011 enables NPC_TOU_10_0500_D000.
  The prompt-skipped source prediction is 5424 -> 5467 / 0xB8E6F54D (+43), not
  yet live-validated. Evaluate both map loads; never bake in that offset. Keep
  arrival distinct from after_popup: both are 5467 for this route, since 1300
  has EncountReset=false and its fixed dialogue/tutorial adds no draw. The final
  world state sets 8012 and tutorial flags 1029/1329 and clears 514. Evaluate the
  implicit FinishPre placement refresh; do not infer a popup counter reset.
  See docs/parlor-revisit.md; arbitrary time delays are outside this prediction.
- Continue from parlor_revisit.after_popup through src/mansion.rs: steal clothes
  (1400 dialogue/Save popup), Boy's Shadow Soulstone, load New Delsta main,
  steal all three Artist soulstones in one menu visit, then skip 1500 -> 1600.
  Preserve both parlor stages in calculation, Detailed log and JSON; hide their
  compact cards. Mansion entry and lookout Ambush also stay in Detailed log
  and JSON only; do not show a Mansion entry widget in Manipulation.
  User confirmed mansion control at seed-0 advance 5562 / 0xDDCC3032 on 2026-09-28.
  Evaluate map50 and progression, never bake in the +27 from Artist or +95 from
  the second parlor popup. FinishPre refreshes dirty placements before counter
  rolls, including the city dummy enabled by 8014 and five mansion NPCs by 8015.
  Steal preview draws once per menu, attempts once per item even at 100%, and
  generic success dialogue once on closure; clothes uses the story override.
  No extra menu openings or automatic time changes are assumed. See
  docs/throne-mansion-arrival.md for the confirmed endpoint and source evidence.
- Mansion entry now has a separate nested lookout_ambush stage. Open the normal
  menu once, then run 1700; flag 8016 replaces N000 with sleeping placement N010,
  and 25305 clears. The placement evaluator supplies the new NPC-frame draw.
  Expected 5562 -> 5564 / 0x5121903C: menu +1, replacement +1, no success/dialogue
  lottery, no counter reset. Keep 1800 separate and unexecuted. This endpoint is
  source-derived, not yet live-confirmed. Preserve mansion.after_entry/world_after
  at the confirmed entry boundary; carry lookout_ambush.world_after onward.
  The Ambush operation preserves the caller's remaining counter. The approach
  is included in the measured full mansion route below; do not deduct it twice.
- src/mansion_escape.rs continues from lookout Ambush using 161 total running
  counter updates from mansion entry to the Pirro trigger, measured 2026-09-28.
  Original equipment, solo Throné level 2, full HP/SP from 0700/1100 command 2310,
  night talent enabled by 1600 clearing flag 652. Active zone is
  ENT_DNG_CTY_1_1_LND at Night/progress 0/danger 3, decrement 12 when running.
  First encounter is one Diamante Guard I. Keep field selection and combat entry
  boundaries separate; party timing was already drawn by the field selector.
  Use the independent core's escape::FirstEscapeBattle; handle normal,
  preemptive and surprise starts, including attacks before the first player menu.
  Search boost inputs for BOTH first-Escape success without attacking/breaking
  and post-return counter > remaining_updates * 12. Escape return rolls B/A
  with FirstEncount=false; do not use the earlier trigger reset or add drops.
  This event has its own repeat cap (default 4) and Unlimited option. The full
  mansion-entry-to-Pirro distance is hardcoded at 161 running counter updates;
  do not expose a setting or API override. Minimize boost button presses then CRT draws;
  end at zero selected BP before Escape. Carry JSON settings, exact input path,
  route-relative trace and world state. Stop before 1800/Pirro cutscene.
  Keep the compact Escape card after the second fight, full trace in Detailed log. This
  stage is source-derived, awaiting live Escape verification; do not call it
  validated merely because continuity/regression tests pass. See docs/mansion-escape.md.

- Continue a successful mansion Escape through src/pirro.rs. 1800 sets 8017,
  then RI_MS_TOU_10_1500 starts ENG_BOS_THI_C01_010 directly: +0 CRT before
  initialization, no map reload, no premature EncountReset/FinishPre. Preserve
  Escape HP/SP; reset BP to 1. Copy second-fight inventory plus mansion's four
  stolen soulstones. The battle core owns Pirro mechanics/routine; this repo
  only orchestrates. R7 chooses the heal branch from the R8 preview. Display
  only boost adjustment and "early latent" / "late latent" in Manipulation.
  Keep outcome, rounds, HP and victory-after-drops CRT in Detailed log/export.
  Omit the known turn-by-turn routine from Manipulation; keep all
  actions/menu checkpoints and source limitations in Detailed log and export.
  Show errors/defeat instead of invented winning endpoints. Preserve the battle's
  after-drops boundary; the separate post-Pirro stage below handles 1900/2000.
  Pirro/Escape live CRT alignment remains unverified;
  see docs/throne-pirro.md. The previous stop-before-1800 boundary describes
  mansion_escape alone; pirro is a separate following stage.
  Pirro's reported seed-0 startup boundary 5595 -> 5631 now matches: +0 serial
  cutscene and +36 core initialization. TalkPlayerID fixes Throné as the start
  speaker, bypassing the random-speaker draw. Do not add a route offset to
  compensate. This endpoint does not validate later boss actions or Escape.
  The 2026-09-28 +5678 capture validates all 203 core CRT calls and six command
  menus, ending this defeat at5881 / 0x1FB4FF1B. Fixes preserve weakness-reveal
  and phase-frame draws and suppress the broken-hit timer when the command's
  ability-camera flag is active. The item builder overrides USE_ITEM's flag.
  No route offset. Winning/boost-search paths still need live coverage;
  do not generalize this captured loss into full boss validation.
  Entry5595 R7 menu is5798: core excludes the normal-enemy full-BP reset and
  reroll from Pirro's scripted AI. Keep serial route propagation; no offsets.
  Latent unlock grants55/80. The no-grape policy checks readiness at R8;
  if unavailable, max-boost swords still occur at R8, followed by R9 latent
  Wind + Ice and R10 sword. Search and exact replay both use this core policy.
  Show "late latent" for that delayed branch, "early latent" otherwise;
  no round label or full routine. Failed searches show adjustment unavailable,
  not an invented latent instruction.

- Post-Pirro prediction in src/pirro_aftermath.rs is wired into the GUI/API as
  post_battle_field.pirro_aftermath and is also available through
  examples/pirro_aftermath.rs. Carry the boss's
  actual after_victory and world_at_entry through return B/A, skipped 1900,
  then 2000/backstreets and map46/Eastern New Delsta Highroad. Never add a
  separate 1800 cleanup. 2000's final normal mission completes scenario700
  before loading backstreets; remove active story/prologue placement gates.
  Its 1550 opt0 is Night -> Dawn, old-phase NPC refresh and one timer tick,
  no Black Market lottery. The 2026-09-29 live replay verifies boss exit5900
  and endpoint5962 / 0x6E0026C2 (+62: 12 streamed levels,34 NPCs,14 counter
  calls,2 canoe BeginPlay calls), counter862. Earlier5959 omitted the boss
  death-slow timer in the core (+1) and canoe initialization in the world (+2).
  Import canoe actors from the actual owning Level actor array, preserving order;
  draw once per actor on level creation and on field restoration after battle;
  the live Highroad Escape below verifies the latter. Never execute presentation.
  All305 battle and62 ending calls match states and caller categories; other
  seeds/progression remain source-derived. No fixed offset.
  See docs/pirro-aftermath.md and tools/audit-pirro-aftermath.cjs. This extends
  the earlier stop-at-drops scope. Preserve data/source/live-pirro-aftermath-5900.json.
  Continue only completed victories, after choosing/replaying the final boost
  plan. Preserve route-relative draw counts; the viewer adds the initial-seed
  offset once. Show Eastern New Delsta's after-cutscenes CRT beneath the compact
  Pirro boost/latent instructions, advances primary and hex secondary. Put all
  callbacks, endpoint and validation note in Detailed log/export. On unsupported
  aftermath display its error while keeping the battle result; no endpoint on defeat.

- Pirro has its own Settings card with independent victory and no-grape caps
  (each defaults to four with its own Unlimited option). Secure the cheapest
  victory within its budget first; no-grape gets its additional allowance
  beyond the actual victory repeat cost, never unused victory allowance.
  Skip no-grape if victory cannot be secured. Keep the winning plan when
  no-grape fails. Legacy pirro_max_boost_repeats/storage migrate to no-grape
  only; victory defaults to four. Exports preserve both caps and exact inputs.
  The victory/no-grape search uses Throné's R3 menu, ending at 3 BP for Darkest Night, for
  victory AND Throné first on R8 / no grape. Freeze the upstream route and
  label target failures explicitly. Use actual button-path BFS with core voice previews:
  the boss-specific 50% max-boost voice makes 2->3 cost three OR four CRT calls.
  Do not use the first/second-fight fixed-cost offset optimizer here. Minimize
  total input count, then actual draws. Export/replay exact pirro_boost_inputs;
  command-menu checkpoints precede them. The displayed path includes the normal
  final boost and names R3 / Throné / 1st command menu. No extra automatic boost.

- src/highroad.rs continues the chosen Pirro victory through the ending loader,
  immediate Day switch, skipped Al introduction and first highroad land encounter.
  Run imported SC_FC13_Tutorial_0100 commands/flags and actual field selection;
  never hardcode the +3 Al projection. Derive level from route EXP and preserve
  unchanged equipment. Highroad selection/startup remains source-derived; the
  6002-menu Escape and field-return calls are live-verified below. Keep new
  encounter info only in Detailed log/export; Manipulation
  shows Highroad Escape boost inputs and its field-return CRT. The first-menu core rejects Surprise
  combat and stops at the Escape check. See docs/highroad.md.
  Add a separate Pirro R8 boost-path search for a non-surprise highroad opening,
  with its own default-four/Unlimited cap. Both early/late no-grape branches use
  3 BP swords at R8; early latent activates before boost, late latent on R9.
  Freeze R3 and replay all later battle/world calls; preserve victory/no-grape.
  Do not spend R3's budget or apply R8 swords to the healing branch. Preserve exact
  pirro_r8_boost_inputs in export/replay. The highroad encounter's Escape search
  has another independent cap and its own Settings card. Escape divides party
  level sum by four; surprise divides by actual party size, never conflate them.
  Highroad Escape now tries boost repeats within its configured threshold first;
  if none succeeds, try a source-simulated immediate break of a single enemy at
  the untouched opening menu, then guaranteed R2 Escape. If no such break works,
  continue the boost search without a limit automatically. Initial Unlimited
  searches directly for Escape. Check actual hit rolls, not just weakness and
  shields. Show the selected boosts or weapon/BP then Escape, no enemy info in
  Manipulation. Keep the R8 surprise budget independent; candidate evaluation
  for that search stops at field selection without running downstream Escape.

- Successful Highroad Escape includes field_return: imported field environment
  BeginPlay callbacks, then counter B/A with FirstEncount=false, after the
  selected Escape attempt. Use Loader::restore_field_after_battle; resident
  levels avoid constructor draws but not these field actor callbacks. Exclude
  battle levels and do not rerun NPC placements. src/flats.rs then uses
  the carried world in Loader.load_map("Fld_Cty_1_2"), assuming no second
  Highroad encounter or intervening CRT actions. Keep both boundaries distinct;
  no fixed +14 load offset and no duplicate return reset. Show field-return CRT
  under Highroad Escape and Flats entry CRT in the next compact card, advances
  first and hex secondary. Include full traces and state in Detailed log/export.
  Do not project either endpoint when Escape is unavailable. Seed0/title4942's
  five-cycle plan is live-verified from menu6002 through five boost draws,
  Escape check6008, two gondola callbacks6010 and counter B/A6012 / 0x2E4773FC,
  counter1448. Preserve data/source/live-highroad-escape-6002.json and its importer;
  regression compares every state, caller category and gondola identity. The
  earlier6010 prediction omitted field restoration's gondola callbacks, not a
  voice or counter reset. Flats now predicts6026 / 0xF0F1E302, counter717;
  this downstream endpoint still awaits live validation. All56 running counter
  updates to harbor cost672 and leave45. See docs/new-delsta-flats.md.
  User separately measured61 walking updates on Flats, costing549 and leaving168
  at counter717. src/flats.rs owns both full-route measured budgets; GUI/JSON
  and examples/flats_trace.rs use these independent counts. Strictly positive
  exit thresholds are550 walking and673 running. Uniform-state odds are
  96.8431569636% walking and62.4802573584% running. Do not infer an exact mixed
  percentage from the two totals. Mansion's161 is already a running measurement.
  Highroad now has a separate Flats-walking allowance, default4 additional
  repeats beyond the actual Escape plan, with independent Unlimited. API key
  highroad_flats_max_boost_repeats. Freeze earlier route inputs; replay R1 boost
  cycles through successful Escape, shared field restoration and Flats loading,
  requiring counter>=550. Keep already-safe plans. If capped search fails, retain
  Escape and explicitly flag the Flats target failure. A safe break fallback is
  retained; an unsafe one can be replaced by a direct R1 Escape candidate within
  this allowance. Do not execute abandoned trial attacks. Preserve settings,
  selected inputs and escape.flats_search metadata through GUI/export/replay.
  Flats displays estimated running fraction of route DISTANCE using separate
  measured densities: cost(f)=61*9*(1-f)+56*12*f=549+123*f. Reserve1 counter point,
  round running share down to0.1%, cost up. Counter>=673 permits100% measured
  running; intermediate shares must say estimate because switch-phase overshoot
  is not simulated. Do not reuse the opening's equal-update-share formula.

- src/harbor.rs continues reachable daytime Flats through an on-foot harbor tag
  and immediate return. Both loads use the same world Loader; import map44,
  evaluate progression/NPC gates and the two ordered harbor gondola callbacks.
  Loader::visit_map records ordinary visit/icon metadata with zero CRT calls;
  scripted load_map callers retain their own visit gates. Do not skip a map's
  initialization because it was visited or shares the previous battle package.
  Harbor is a field, so the first-town fast-travel tutorial is ineligible.
  Returned Flats gets a fresh first-encounter counter. Expose compact return
  advances/hex/counter in Manipulation, both traces/world states in Detailed log
  and post_battle_field.harbor_return. Omit projection when walking is unsafe.
  Seed0/title4942 predicts6026 ->6039 ->6053 /0x04A0B097, counter515. The +13/+14
  totals are evaluated results, never fixed offsets; flag6717 alone enables nine
  more daytime harbor NPC draws. This endpoint remains source-derived pending
  live verification. See docs/harbor-return.md and tools/audit-harbor.cjs.

- Flats -> Abandoned Village uses src/flats_escape.rs, beginning at harbor
  return. Fixed170 running counter updates (user measured~165 plus allowance),
  daytime decrement12. Subtract ceil(entry_counter/12) to get remaining updates;
  require a strictly positive counter on arrival. Ends BEFORE loading/tagging
  the village. No distance setting. Use imported field selection and battle-core
  FieldEscapeMenu/first_action_break, never copy battle logic into the assistant.
  Highroad settings add highroad_flats_approach_max_boost_repeats: four extra
  repeats beyond the actual safe-harbor plan, with independent Unlimited. Keep
  Escape and walking safe; replay both harbor transitions before testing surprise.
  Keep prior plan and flag failure when capped. Do not spend earlier budgets.
  flats_escape_max_boost_repeats is a separate default-four/Unlimited setting:
  limited direct Escape plus safe remaining run, then immediate single-enemy
  break plus safe run, otherwise unlimited direct boosts. Reject a break whose
  reset is too short. Every candidate starts at the original menu; no trial draws
  leak into the final route. Shared field restoration uses Flats callbacks.
  Manipulation gives inputs/run/field-return CRT; full encounter and trace live
  in Detailed log/export. At seed0/title4942 the old Highroad five-repeat plan
  yields Surprise; the new joint plan uses seven repeats and one Flats repeat,
  returning from harbor at6055/counter609 and escaping with1503, leaving75
  after119 remaining updates. New endpoints await live verification. Preserve
  historical Highroad/harbor fixtures with the new approach allowance set to0.
  Baseline second-encounter estimate37.0678% at170 updates (25.9763% at165)
  assumes independent uniform reset starting states, with within-pair CRT
  correlation preserved. Do not present it as exact route-conditioned odds.
  See docs/flats-escape.md and examples/flats_second_encounter_odds.rs.

## Resolved Eastern Snows Fireball retail mismatch (2026-10-07)
Seed 0/title advance 12034178, before-Osvald branch, Eastern Frost Mole I x2:
user confirms Fireball, then Throne four 0->1->0 cycles and Escape; observed
field return 12035657. Original model predicted12035667. Field spell importer
was missing Osvald type18 Fireball/Icewind voice rows; corrected from Mug data,
with regression asserting one Fireball ability-voice draw. Exact same action
replay now predicts12035668, so this correction DOES NOT resolve the report.
Retail capture snows-fireball-live-20261007.json confirmed the full sequence.
Fixes: import ability voice rows; aggregate single-damage-key multi-target break
timer after reaction voice; allow next-player ally-break acknowledgement.
Throne acknowledges Osvald after7 voice checks instead of17. Exact original
Fireball + four boost cycles now returns12035657. Regression fixture/test:
OT2BattleEngineRust/data/source/live-snows-fireball-20261007.json and
tests/snows_fireball_live.rs. This validates the captured path, not every spell.
Diagnostic: examples/audit_snows_spell.rs replays saved profile-safe-defend.json.

- Field spell voice completeness audit: import type18 Darkest Night for Throné in both field profiles, plus Fireball/Icewind for Osvald. supported_field_spells_have_eligible_voice_rows checks each available spell actually selects a voice and consumes one CRT draw. Ally-break acknowledgement is shared by normal and scripted attacks. Shared delayed break timer currently handles single-damage-key area actions; do not claim arbitrary multi-hit area timing is retail-validated.


- Second Snows capture clarification: user confirmed NO boost before Fireball, then3 Throne cycles. snows-boost-fireball-live-20261007.json is misleadingly named; it is an UNBOOSTED Fireball replay and returns12035656 exactly as corrected core predicts. Do not infer voice suppression from it. The optimized one-Osvald/three-Throne plan remains unverified by these captures. Temporary pre-spell search restriction was removed before deployment.

- Boosted field spell breaks: permit +1 Fireball/Icewind/Darkest Night at the opening menu and price the actual boost path (no mandatory final cancel). Replay the real BP damage tier and resources; reject victory or any surviving unbroken enemy. Physical zero-BP policies remain unchanged. Snows/Eastern searches must evaluate odd cost layers. Regression compares [1] Fireball with [1,0] on the retail two-mole fixture: both survive/break, RNG endpoints match, and boosted cast saves one input.

- Route update: remove Gold Ring and Antique Coin steals entirely. Keep clothes, Boy Shadow Soulstone and Artist three soulstones. Second-fight adjustment only protects the mansion approach; no ring success/order target. Propagate actual remaining events through all downstream RNG/searches.

- Latest route: restore Gold Ring only (80% steal). Compare before/after clothes at equal travel cost and select success plus safe mansion approach via second-fight boosts. Antique Coin stays removed. Show chosen theft order and ring checkpoint; propagate the three actual steal-session draws.

- Fixed timer-refresh route: enter Winterbloom TOWN, dismiss the chapter prompt without starting story, immediately switch to Night. Fast travel to New Delsta Harbor at Night; immediately switch to Day before boarding for Crackridge. Use shared manual_time_switch callbacks, preserve encounter counters and carry RNG through both Western encounter-count branches and upstream safety search. These fixed actions belong in Detailed log, not extra variable-action Manipulation widgets.

- Retail Winterbloom trace 2026-10-09: seed0 absolute19286216 -> town/story19286249 -> manual Night19286262. Manual stable-phase switches call native f5cb60 -> ed58b0 placement refresh, not RefreshBarCharacter. Remove tavern draw from manual_time_switch; retain map-load/recruitment tavern callbacks. Fixture/test live-winterbloom-night-20261009 validates all46 states and NPC labels. Do not subtract a fixed endpoint offset.

- Tavern trigger audit: RefreshPlacementOnChangeLevel explicitly calls RefreshBarCharacter AFTER RefreshSpawnCurrentLevel; never infer the former from native ed58b0 alone. Manual switches, transitional-phase placement refresh and FinishPre dirty-placement cleanup do not inherit map-load tavern lottery. Legacy duplicated calls removed. See docs/tavern-loading.md.

- Southern Cropdale correction: route uses Fld_Fst_1_4 /MapID132 and ENT_FLD_FST_1_4_LND, not Fld_Fst_1_1 /129. Retail capture2026-10-09 verifies all12 draws19286486->19286498: outgoing2, six level constructors, save-point/info-board NPC frames2, incoming2. Old wrong map added a nonexistent GD stream and two flag-animation draws. Regression retail_southern_cropdale_load_20261009.

- Retail Oresrush Mother acceptance:19286550->19286557. Resolved cause: BarMenuStorySelect calls PlayEventForce (e920f0 sets+0x59d), EventLoadLevel e8dd70 bypasses same-map rejection when forced. RequestLoadFieldLevel/OnFieldLevelShown each call ResetEncountVolume. Shared loader preserves forced transitions and FirstEncount; temporary chapter-specific reset patch removed. See docs/oresrush-partitio.md and tools/audit-forced-event-load.cjs.

- Oresrush allowance: first THREE time switches before Partitio and first TWO after are free independently, all after Mother chapter starts. Further switches cost7 boost inputs; leave by day. Search pruning must use the combined five-switch allowance.

- Clockbank continuation: after Inventor scope, visit Western Clockbank Highroad47 then Southern48 with no intervening encounter (user confirmed). Slow/fast measurement117/110, padded120/113; use water-zone shared measured_reach (8 base, Evasive50%, dash1.4 truncated =>4/5). Then visit Clockbank main64, accept Partitio MS_SHO_01 via MapInStorySelect and skip opening MS_SHO_20_0050 (daytime, flag6076, FinishPre counter reset). Fixed checkpoints in Detailed log; movement percentage in Manipulation. No later industrial-district event yet. Source data/source/partitio-clockbank-opening.json. Retail endpoint verification pending.

- Corrected Clockbank movement: raw slow117=68 water+49 land; fast110=66 water+44 land. User now requests SIX padding updates, all LAND: slow68water+55land, fast66water+50land. Calculate each zone separately (day/Evasive: water4/5, land5/7), costs547/680. Display equal fast-distance percentage for EACH terrain section; counter584 =>27.0% fast,73.0% slow. This supersedes the earlier all-water120/113 calculation.

- Inventor Highroad Escape boosts must jointly guarantee Escape AND Southern Clockbank all-slow safety with six land-update padding. Reject downstream entry counters <=547 and continue from the original menu without spending earlier controls. Baseline uniform-CRT safe probability97.089401679%; current counter584 is already safe. Do not treat a short counter as an unrecoverable route error.

- Clockbank movement presentation now starts fast on water, then switches ONCE to slow. Percentage is total unpadded route distance, estimated using slow-count terrain weights68:49. Piecewise water then land costs retain six land-padding updates; counter584 =>36.0% fast then64.0% slow, superseding equal per-terrain27%.

- Partitio Ch2 Clockite continuation: after opening visit Industrial District65, run0100 (links0110),0200 (links0300),0400 (links0410),0500 (factory54 then links0600 returning65),0610 (scripted Clockite x5 and item popup; NOT Purchase). Visit factory54, delivery0615 Yes at command2 links0620, which loads65 before FinishPre; endpoint flags6059+6077. Skip scenes/advance short dialogues, no optional actions. Detailed log includes stages. Source data/source/partitio-clockite-route.json. Runtime integration passes; retail check pending.

- After Clockite delivery fast travel directly to tagged Crackridge Harbor, daytime; predict first land encounter using actual carried party/levels/knowledge/progression and FirstEncount. Require Normal/Preemptive: nearest earlier control is Inventor Highroad boosts, jointly preserving first-try Escape and slow-safe Clockbank. Harbor itself searches boost/cancel for first-try Escape; this NEW return is not one of the earlier mandatory-break encounters. Show a distinct Harbor-after-Clockite widget and entry/menu/return checkpoints. Core additive import-highroad-return also covers Wildlands1_3 encounter groups. Retail validation pending.

- After the post-Clockite Harbor escape, enter Southern Crackridge Wilds Fld_Wld_2_2 (MapID88), land ENT_FLD_WLD_2_2_LND. Current progression selects fixed first base group ENG_FLD_WLD_2_1_LND_DAY_170 (Wild Mole IV x2 + Armor Eater); rare replacement still applies. Harbor Escape boost search also requires this Southern encounter Normal/Preemptive, using actual carried progression/party. Southern searches its own first-try Escape. No extra EXP from either escape. Separate location widget plus Detailed log checkpoints; retail validation pending.

- Latest continuation: Southern Crackridge -> Western Crackridge Fld_Wld_2_1/87 encounter -> tag Crackridge Twn_Wld_2_1_A/101 -> return Western encounter -> Merchant first license GUILD_003/job4 (SYS_MER_GUILD_0010+0030; no reset) -> Western Gravell Fld_Wld_3_2/90 -> Night -> Chubby Cait ENE_NML_OTR_150 and no surprise. Every planned opening rejects surprise. Strict nearest-menu propagation: preceding Escape boosts protect next opening; Western return boosts jointly escape and target Gravell. Carry party stats/EXP/knowledge; no EXP for escapes. Stop before Gravell actions until user specifies routine. Assumes the requested one encounter on each Western pass; travel distances unmeasured. Source data/source/crackridge-gravell-route.json; retail validation pending.

- Conning Harbor Townsperson now steals Pom M ITM_CSM_0080 then Wind Soulstone L ITM_CSM_0400 in ONE menu session: open, two success checks, close. Both PurchaseItem rows ProperSteal8; require both successes with carried level. Second Western Crackridge visit now uses boost/cancel then Wind Soulstone L (2700 fixed wind), not Escape. Shared item sequence/victory/drop/reward/level-up path carries updated stats to Merchant license and safe Chubby Cait Gravell selection. Reject packs the item cannot clear. Earlier Western visit still Escape. Supersedes the two-rare-only EXP assumption.

- User-observed Townsperson optimization: after stealing Pom M and Wind Soulstone L, change the selected path action before closing the window. Suppress the Steal closing-voice draw for THIS session only; total three CRT calls (open + two checks). Record a zero-draw boundary and show the changed closing instruction. Other theft sessions unchanged. Propagate/search from the actual resulting state, never subtract an offset at the route end.

- Gravell setup requested after Merchant License: Throné Inventor, Hikari Merchant+Donate BP+Hired Help, Agnea Stimulate+Bewildering Grace, Partitio Hired Help; everyone A Step Ahead, Hikari Grows on Trees. T1 means A Step Ahead bonus turn. If Agnea last, Thunder L goes BEFORE Agnea on T1 (confirmed). Track real JP/job stats/resources/support unlocks; retain Evasive Maneuvers. Merchant JobID is0, NOT GuildData ID4; corrected license key. Current implemented endpoint remains before Gravell battle init: A Step Ahead/BG100/Guards exact action support is PENDING, so do not emit the old ordinary first-menu RNG as valid. source/gravell-battle-audit.json records source sequences/random tables.

- 2026-10-10: Gravell BG/Guards pending status superseded. All39 max-BP effects execute in battle/grace.rs; complete minimum-input fight search/replay and real victory/all-alive/EXP100 acceptance in gravell_routine. Assistant integrates rewards/progression/field return and boost instructions. Unplanned enemy continuation is rejected, not fabricated. See core docs/gravell-search.md; new flow still awaits end-to-end retail validation.

- Gravell routine revised: T1 earlier merchant +1 Donate BP to later merchant, later merchant +3 Guards, Agnea Defend, Throne Pom M to Agnea; T2 Agnea +3 BG then next player Thunder L. Both merchants learn Donate BP. Strict propagation freezes first Guards state with sufficient damage for Thunder L, then searches BG; victory/EXP100/all-alive unchanged. Supersedes previous Agnea-first/Stimulate/Boots branching routine.

- Latest setup: equip A Step Ahead on all four immediately after Inventor crafting; preserve secondary jobs until Merchant license. All later field openings (Clockite Harbor and both Crackridge passes) must use the extra turn. Do NOT equip Hikari Grows on Trees; derive rewards from actual supports. Partitio learns Hired Help and Donate BP at Merchant setup.

- BG coarse search: after Clockite delivery in Industrial District, alternate night/day in real Loader callbacks, exiting day. Scan exact first-child EXP100 anchors; prioritize departure ~850 draws earlier (650..1100 window), Chubby initialization 214..380 earlier, BG/Guards combined <=100 inputs. Strict forward controls; 32-repeat local exploration plus safe A Step Ahead Defends (8 input equivalents) and Western return Escape vs Wind L (10 action equivalents). Compare complete costs within first viable target, not global optimum. Failed local windows move to later targets; Required outcomes are never silently dropped. Replay all town switches and full BG effects, demand victory/EXP100/all alive. First-effect targeting intentionally misses possible later-effect EXP100 alternatives. New plans await retail validation.

- BG search now accepts its target EXP100 lottery at ANY actually executed child, not only the first. Execute targeting, effects, callbacks and early termination in sequence. Trace must contain the target lottery; final real EXP100/victory/all-alive remain mandatory. Widen coarse arrival windows for child-effect draws (214..1200 battle-entry gap; 650..1900 departure gap). These are search heuristics, not RNG offsets.

- Latest time-cost model: breaking6 input equivalents, Defend3, each actual round transition8. Charge transition once using the resulting battle round, including transitions after a break. Same-round Defend3, round-ending Defend11. A Step Ahead safe Defends are restricted to the bonus round and cost3. These are search costs, never CRT draw overrides. Wind Soulstone victory cost10 and town-switch cost7 remain unchanged.

- 2026-10-10 Clockbank BG optimization: preserve the fast validated baseline and
  apply deterministic bounded cost refinement (10,000 work units total). Search
  extra BG inputs and nearby Guards alternatives, retain only cheaper fully
  replayed plans. Never report a required outcome unavailable because refinement
  ran out of work. Cache by exact battle constructor inputs, not CRT alone;
  never cache truncated failures. See docs/clockbank-bg-search.md for limits,
  benchmark fixture, and the distinction between cheaper plans and global optimality.

- Adaptive Clockbank refinement supersedes fixed margins as eligibility rules:
  total incumbent cost bounds local adjustments; gap estimates only order trials.
  Bounded fair work per target preserves speed and an accepted fallback. Regression
  recovers+4280 at136 switches/cost1217 for the former484-switch outlier.
  See docs/clockbank-bg-search.md for remaining search-quality limits.
