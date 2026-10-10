# Throné's Scholar setup after Hikari recruitment

The route assigns Scholar to Throné, purchases Lightning Bolt and Elemental
Barrage, then equips Evasive Maneuvers. Osvald remains on the bench with his
resources, rewards and support equipment intact. His main job does not use
the Scholar License acquired at the guild.

`src/scholar.rs` records assignment, learning, and support equipment/menu
closure separately. All three preserve the full CRT state and encounter
counter. No field load, NPC refresh, battle-return callback or counter reset
is manufactured for menu closure. `RefreshCharacterResource` updates resource
residency, while `SetAfterMainMenu` only updates movement bookkeeping.

Scholar defaults Fireball (`ABI_SET_SCH_010`) and Icewind (`ABI_SET_SCH_020`)
were initialized in the saved job ability array; assigning Scholar exposes
them without purchasing. Lightning Bolt (`ABI_SET_SCH_030`) costs30JP and
Elemental Barrage (`ABI_SET_SCH_060`) costs100JP. Four learned abilities,
including defaults, unlock Evasive Maneuvers (`ABI_SCH_SUP_01`). Unlocking
does not apply the support: the separate equipment callback rebuilds active
party effects. The equipped support restores the50% encounter multiplier
after Osvald's bench replacement disabled it.

`WorldState.job_licenses` holds owned counts; allocation counts secondary jobs
on both active and benched characters. Assignment does not consume the item.
The guild's native first `TaskClear` grants `ITM_EQP_JOB_0006` and notification
flag100. The script's406/411 mutations alone are not a license grant. Raw
`JobData.PermissionFlag=7` is not used as an EventFlag gate.

`CharacterJobs` carries the secondary job, learned source AbilitySet IDs,
unlocked supports and equipped supports. These and the license counts are
included in the exact search key and exported route state. `sync_progression`
and victory rewards use the core's job-aware growth helper, preventing loss
or double application of Scholar bonuses on later level changes.

The core applies Scholar'sMP113%, MATK110% and MDEF105% revisions to the
character-revised base stats before flat equipment contributions. The newly
permitted empty staff slot receives `ITM_EQP_ROD_010`, with zero stats; the
original default dagger remains selected. Current HP/SP are clamped, not
healed. This implementation preserves the route's original equipment and
does not model stat-boosting nuts. Source spell metadata is imported; this
setup does not claim to implement unrequested Elemental Barrage actions.

Tutorial54 is checked after a successful purchase. Support unlock prompt57
and equipment tutorial56 share flags1055/1355. Job tutorial58 and guild
tutorial59 share1057/1357. Earlier Osvald menu flags are carried from the rare
kill, so tutorials already viewed are not repeated. Notification102 is set
on support unlock and cleared when entering support equipment.

`tools/audit-scholar-assignment.cjs` reproduces the cooked rows and native
snippets in `data/source/scholar-assignment-audit.json`. Core stat/equipment
evidence is in the independent battle engine's `docs/scholar-jobs.md`.
Tests cover JP failures without input mutation, license use on the bench,
CRT/counter immutability, no healing, support unlock versus equipment,
serialization/cache identity and later job-aware growth/rewards.

For seed0 with4997 title advances and the selected pre-Osvald/second-Western
encounter branch, the setup retains7104 advances / `0x3641B9C0`, counter726.
Throné has448JP remaining after the130JP purchases. These menu callbacks are
source audited; the preceding Hikari battle endpoint still awaits retail
validation.
