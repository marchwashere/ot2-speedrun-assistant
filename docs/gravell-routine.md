Gravell requested routine (pending exact battle execution)

Setup after Merchant license is CRT neutral. Merchant JobID0 vs guild ID4.
Throne Inventor; Hikari Merchant learns Donate BP then Hired Help; Agnea learns
Stimulate then Bewildering Grace; Partitio learns Hired Help. Cycle Inventor
through all characters to unlock A Step Ahead using four available inventions;
finally leave it on Throne. Equip A Step Ahead on all, Grows on Trees on Hikari.
Preserve Throne Evasive Maneuvers. JP and job/equipment/resource changes tracked.

T1 is A Step Ahead extra turn, T2 first normal turn (user confirmed).
Agnea first:
  If Throne precedes both merchants, Stimulate the first merchant; else Defend.
  First merchant Defend. Throne Springy Boots on Agnea. Other merchant Pom M on Agnea.
  T2 Agnea +3 Bewildering Grace; require EXP100.
  If ongoing, first merchant +2 Hired Help Guards; require >=2 hits on Chubby Cait.
  Next available actor Thunder Soulstone L.
Agnea not first:
  First actor Pom M on Agnea. Agnea +3 Bewildering Grace; require EXP100.
  First remaining merchant Defend; remaining actor Thunder L.
  If Agnea is last, Thunder L is used BEFORE her on T1 (user confirmed).
  T2 merchant +2 Hired Help Guards; require >=2 hits on Chubby Cait.

Do not claim routine guaranteed: reject BG candidates that make later actions
unavailable, kill party, lose the Cait, or fail actual victory/EXP100 requirements.
Source BG max-boost selection: four effects, total weight997, EXP100 weight1.
Each chosen effect can consume additional CRT; never model four adjacent draws.

Core update 2026-10-10: bonus-turn initializer, timed Stimulate/Springy Boots,
Pom M/Thunder L, all24 opening plans, resumable BG lottery, Guard preparation
lottery, Cait attack-damage formula and minimum-input boost frontier are implemented
and tested. See the core docs/gravell-search.md for their precise boundaries.
Still required: BG child execution, Guards construction/timed hit callbacks,
full routine continuation and EXP100 rewards, then route/UI wiring. The assistant
still stops before battle init; no partial lottery is presented as a winning fight.
