# Hikari recruitment in Ryu

Galdera also completes this fight, obtaining the two guaranteed Fire Soulstone
drops from Ruffian Soldier I. It accepts Hikari, skips Chapter 1, and dismisses
the full-party replacement dialog with Hikari still on the bench. The active
party remains Throné / Osvald / Agnea / Partitio, Osvald retains Evasive
Maneuvers, and Throné receives no Scholar setup. Subsequent map/tavern loads use
the real bench and completed Hikari prologue; the fight is not replayed later.

Any% Hikari continues from the selected safe Ryu arrival through the
first Hikari introduction, its mandatory fight, invitation, Chapter1 skip,
Osvald replacement and tutorial dismissal. Battle simulation is implemented
in the independent `ot2-battle-engine` crate; `src/hikari.rs` executes the field
and recruitment callbacks with the shared world loader.

## Battle boundary and routine

`MS_KEN_00_0100` selects YES at5010/index47, sets37501 and requests event battle
at600/index66. The imported encounter-volume row resolves
`ENM_EVE_WAR_C00_010 -> ENC_EVE_WAR_C00_010 -> ENG_EVE_WAR_C00_010` directly.
The native event path uses Group[0], Normal start and no rare enemy; it does
not run ordinary field encounter selection, surprise or player-timing draws.
The battle map is already in Ryu's resident streams.

All four existing characters participate: Throné, Osvald, Agnea and Partitio.
Hikari has not joined yet. At the first player menu, use one regular Ice
Soulstone. It deals260 to the ice-weak leader and200 to each grunt, leaving
only the leader with20HP. At the next player menu, attack it with that actor's
unboosted original default weapon. The core preserves native initialization,
Study Foe knowledge, turn order, conditional voice/timer calls, reachable
leader actions, damage and drops. It rejects failure instead of inventing a
victory or third action. Lower-level parties outside this route can fail.

The supplied current party has original gear and all four Chapter1 latent
unlocks. A core sweep covers1,024 initial CRT states and all first/second
player actors for EXP[1530,1467,0,0]. This verifies the implemented model,
not retail CRT alignment.

## Return, recruitment and persistent state

The event victory returns to the field and rolls the real B/A encounter
reset before playing `MS_KEN_00_0300`. The old0100 event is replaced at the
serial handoff: its decline-path FinishPre is not called. The victory event
chooses YES127, joins Hikari at3000/index133, and chooses NO138 to skip his
Chapter1. Join/skip do not average party levels or award EXP to Hikari.
He starts at level1, EXP0, JP0 with his original equipment.

With four active members, native join initially puts Hikari on the bench.
At3005/index148, the full-party dialog swaps Osvald for Hikari. The final
active party is[7,1,8,4]; the bench is[6]. Osvald keeps the battle's EXP/JP,
equipment, resources and purchased skills in the persistent character ledger.
Support skills are rebuilt from active members: Evasive Maneuvers is inactive
after this swap. The completed Hikari prologue adds one progression point;
Throné's active Mother scenario701 survives.

The event's final flags refresh eligible ordinary NPCs before its separate
FinishPre B/A reset at179. Tutorial67 at181 adds no further counter reset.
Ryu has no tavern lottery. Both field resets still use the map's first-counter
flag; this event battle does not increment the ordinary field encounter count.
Fixed dialogue, event-only actor presentation and55/65 fades make no CRT draw.

Participants receive source battle bonuses and level/resource growth before
Hikari joins. The assistant does not yet maintain a complete field inventory:
the core records one Ice Soulstone consumed and exact drop labels, while the
route assumes the user has the prescribed item. Fixed skip-story item grants
consume no CRT and are outside that inventory ledger.

## Display and verification

Manipulation adds only the action the player must take: replace Osvald with
Hikari. Detailed log adds both command-menu CRT states, victory and field-return
checkpoints, final recruitment RNG/counter, the battle log and all five EXP/JP
ledgers. Existing movement instructions and upstream manipulation stay intact.

For the current seed0/title4997/shared-recreation-off configuration, Before
Osvald and the Western second-encounter branch predict Ryu entry7000,
first recruit menu7063 (Throné), second menu7087 (Agnea), victory7096,
field return7098, and recruitment completion **7104 /0x3641B9C0**, counter726.
The participants' earned reward is61EXP/66JP/825leaves. Active levels are
15/1/5/5 (Throné/Hikari/Agnea/Partitio); benched Osvald remains15.
The exported comparison retains every pre-recruitment field and selected
input exactly; only the new Hikari prediction and explanatory text are added.

`tools/audit-hikari-recruitment.cjs` reproduces cooked data/native source
evidence in `data/source/hikari-recruitment-audit.json`. Regression tests
cover exact event branches, immutable input, continuous battle/field trace,
two distinct counter resets, participant-only rewards, growth, party swap and
bench persistence. The new recruitment endpoint still needs retail validation.
