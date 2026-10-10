# Character progression and rare victory

The route carries a per-character ledger in `WorldState.character_progression`:
total EXP, earned/spent JP, level, character growth, original equipment combat
stats and HP/SP/BP/latent resources. Existing `party_experience` exports remain
available; levels and growth are derived from the imported CharacterGrow and
PlayableCharacterDB tables. Scripted wins initialize the carried rewards, party
recruitment preserves Throné and adds Osvald, and escapes award nothing.

Rare victory rewards sum every defeated enemy, including accompanying enemies.
Source `BattleManagerBP.CalcBonusReward` adds break/overkill EXP, one-turn JP and
no-damage money separately. The supported one-action rare clear grants the
one-turn JP bonus. Osvald spends 130 JP on his two additional skills. Level-ups
update the character's own growth parameters, preserve equipment contributions,
and restore HP/SP. Onward encounter timing and escape use these derived levels.
Detailed log shows the resulting levels, EXP, JP and HP/SP/speed.

This remains the supported original-equipment route, not an arbitrary equipment,
skill purchase or battle-action planner. Earlier combat routines continue to use
their imported level-specific configurations; the ledger records their actual
rewards and feeds subsequent field progression.

## Cultured Cait CRT correction

The 2026-10-02 replay has Majestic Snow Marmot, Cultured Cait and Snow Yak in
Western Winterbloom Snows. At seed zero: entry 6514, first command menu 6563,
victory/drops 6572, overworld return 6574. The raw capture is
`../outputs/cultured-cait-live-20261002.json` in the decomp workspace; the filtered
correct attempt is saved in the core's `data/source/live-field-rare-cultured.json`.

`LibAilment.Battle_ExecDamageHP` initializes Kill/Break/OpenWeakness flags, loops
through targets, then chooses one reaction. The simulator incorrectly emitted
one kill reaction per enemy. It now schedules the three damage notification
timers, one kill voice, one victory voice, three item-drop rolls, followed by the
assistant's two field counter rolls. This removes two duplicate CRT calls without
changing field-return behavior or adding a fixed adjustment.

## 2026-10-10 Cait fixed-damage correction

The core omitted EnemyDB.UseCatDamageType for fixed elemental items. Source
BP_ActionCommandBase.CalcAilmentDamageHP applies integer damage / 100 (minimum1)
after resistance/disease multipliers. SSM therefore deals11 to Cultured Cait,
not1100. This still kills its8HP, but does not earn Overkill; the three coast
scorpions also do not meet the double-remaining-HP threshold. Coast EXP is1298,
not1427. Agnea carries61+1298=1359 EXP, Lv14/Speed133. The old total1488
incorrectly selected Lv15/Speed137 and changed the Highroad first speaker.
Shared damage and rare-clear eligibility now apply the imported flag; no fixed
EXP or RNG subtraction is used in the route. Existing observed RNG action
trace is tested independently from changed rewards and downstream stats.

Validation: corrected live API for seed0/title19284723 gives Throné17/134,
Agnea14/133, first menu19287621 and both reported orders. The recorded
cultured-Cait action CRT regression passes with11 damage. Broad library tests
also exposed failures in eastern_escape, highroad, winter_voyage, winterbloom
and snows_escape route assertions; these remain to be investigated. The long
flats joint-search test was stopped; the full suite is not claimed passing.
