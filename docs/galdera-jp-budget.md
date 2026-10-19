# Galdera JP deadline

Accept a JP100 route only when the resulting carried ledger can fund all four
active characters' planned purchases. Do not count future Gigantes or late
boss rewards; a Shipwreck candidate may include Gigantes because it has already
been simulated and awarded. No extra battles or skill purchases are performed
by this check.

| Character | Planned paid commands | Total JP |
|---|---|---:|
| Throné | All Thief, five additional Warrior (Deal More Damage) | 14,260 |
| Osvald | Three additional Scholar, three Merchant, five Warrior, four Arcanist | 13,890 |
| Partitio | Three Merchant, two Dancer, four Warrior, all Cleric | 12,020 |
| Temenos | Two Dancer, all Cleric | 9,760 |

`galdera_jp::audit` computes the cost of remaining commands, then compares it
with `jp_earned - jp_spent`. Defaults are free; existing commands are not paid
twice, while unrelated purchases still reduce the available balance. Job
costs were checked against the extracted JobData tables. No Warrior divine
skill is required for Deal More Damage.

Seed 0x3054BA63, Pirro first/154 HP, observations
`NNNSNSNNNNSSSSSSSSNNSSSSSNSNSN` selected a preemptive Nameless Isle JP100.
The old engine falsely awarded Domination: 12,210 JP instead of 11,100.
Before that victory Throné had earned 2,399 and Osvald 2,374. Correct totals
are 13,499 and 13,474: short by 761 and 416 respectively. Even Gigantes's
45 JP leaves deficits of 716 and 371. The early rare was a Cultured Cait.

The core now distinguishes the actual A Step Ahead bonus turn from preemptive
turn 1. All Roque, Nameless and Shipwreck acceptance paths validate the funded
skill plan. Insufficient results are skipped, not presented as completed
manipulations. A Roque fallback whose maximum reward cannot fund the plan is
not searched indefinitely.

Tests: `galdera_jp`,
`preemptive_3054ba63_cannot_borrow_domination_or_later_boss_jp`.
Full-route reproduction and budget assertions:
`cargo run --offline --release --example audit_jp_budget -- fixed`.

Verified corrected replay: Roque Island Harbor, Normal, 12,760 JP; two Tropu
switches within the free allowance, ten extra inputs. Throné ends with 15,159
available (14,260 required), Osvald with 13,874 available (12,630 remaining).
Partitio and Temenos also meet their budgets. Full local replay took 3.7 seconds,
including the unchanged EXP100 search and progressive prefix.
