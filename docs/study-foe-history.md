# Saved Study Foe history

EnemyDB identity keys the core EnemyHistory ledger: studied, revealed weapon
icons and revealed elemental icons. BattleConfig accepts it; field Escape,
break, rare-item and Mug adapters expose history-aware entry points and return
updated history. Existing convenience APIs explicitly start with empty history.
The assistant carries the ledger through maps, recruitment, successful escapes
and victories. Candidate clones cannot mutate the incoming world.

Source: BattleEnemyBP restores SaveEnemyData.WeaknessOpen / IsPreparation,
OpenWeakIcon selects only closed icons, and ExecSpecialAction_Preparation
deduplicates enemy types. First Study Foe reveals one icon; repeat preparation
reveals two, bounded by the remaining icons. Duplicate copies share openings
without extra draws. Normal damage/item revelations also persist.

The 2026-10-02 capture is continuous from6494. Rare victory returns6574.
Second encounter actor initialization starts6581. Its previously prepared
Marmot and Yak each select twice (bounds3,2), at6600..6603. The first player
menu is6630; the unadjusted Escape roll80.34% fails. The old fresh-history
projection ended initialization6628 and incorrectly used roll11.49%.
Core regression replays every state through the recorded failed check.

The separately reported Lv14 versus predictedLv15 EXP discrepancy is not
resolved by this change; both levels give the captured menu and Escape rate.
