# Oresrush controls and Conning Creek Coasts

The active village fork now continues after Canalbrine's water Escape through
Northern Conning Creek Coast (`Fld_Sea_2_2`), Western Conning Creek Coast
(`Fld_Sea_2_3`) and Conning Creek (`Twn_Sea_2_1_A`). No Osvald Chapter 3 starts.
Party remains Throné/Hikari/Agnea/Partitio; Osvald remains benched.

Both land encounters occur after a manual switch to night, just before triggering
the first encounter; switch back to day immediately after each fight. With the
carried Progress5, both tables use the fixed first group of four Sea Scorpions.
Rare replacement still runs: Cultured Cait has the source 2% gate. The route must
kill a Cultured Cait pack in either coast and escape the other pack successfully
on the first attempt. Both openings must be Normal or Preemptive. A second Cait
is not required and, if present in the other fight, is escaped.

Shadow Soulstone M has fixed 1100 dark damage. Sea Scorpion has865 HP and is not
dark-weak; Cultured Cait has8 HP. The actual four-character item callback sequence,
English voice duration per identity, aggregate AoE kill reaction, victory voice,
enemy drops and real B/A field reset are replayed. EXP/JP bonuses and resource/
level growth are awarded to the active party before the next map/encounter.
The benched ledger is unchanged. Learning/equipping Osvald's support is not repeated.

## Search

Compare Oresrush switches after Mother’s Chapter 2 opening, before and after Partitio. Each side has two free switches; further switches cost seven inputs. Exit by day; odd/odd allocations replay recruitment at night. Cropdale has no switches. Freeze the earlier Escape and reject Oresrush candidates that lose either walking-safe Ryu leg. Each allocation receives strict forward propagation; choose the least combined time-switch and downstream boost cost, preferring Northern Cait on ties. Stop when the remaining allocations cannot beat that cost. Earlier route
controls are fixed before extending to the coasts. Each nearest control is
searched independently in increasing boost count:

- Hikari second menu: first safe Canalbrine opening.
- Canalbrine menu: successful Escape and safe Northern coast opening.
- Northern coast menu: successful Escape (or SSM if a Cait is already present),
  safe Western opening, and Western Cultured Cait if not killed in Northern.
- Western coast menu: successful Escape, or SSM for the remaining Cait target.

No cross-menu cost allocations are searched. This preserves simulation and all
required outcomes but can use more boost inputs than the former joint optimizer.
The legacy cost-layer helper remains only for regression comparisons. All
selected before/after encounter alternatives still receive complete results.
## Loader and presentation

`tools/audit-conning-loading.cjs` inspects every placed Blueprint/component,
parent and embedded level function. New shared-loader actor rules cover the
battlefield flag variants and coastal lamps:47 begin-play draws in Conning Creek,
derived from actor arrays (not a route offset). The two field maps have no such
environment callbacks. Tavern refresh draws for appearance, then each requested
slot selects an eligible benched character if available and rolls for an item.
An excluded Osvald skips selection but does not skip the hidden placeholder's
item roll. See `tavern-loading.md` for the corrected call path.

Manual time switches use flag-lottery, tavern and NPC callbacks. They do not
invent a counter reset. Walking to the night switch is unmeasured: the route
assumes one intended encounter on each coast and does not claim a second-encounter
distance guarantee. Post-fight and incoming-map counters are actual source rolls.

Manipulation displays the Cropdale switch instructions and one card per coast
with formation, first actor, complete boost inputs, SSM/Escape and the day switch.
Zero adjustments remain visible. Detailed log includes all intermediate RNG
spans, town arrival, counters and source-verification status. Settings describes
the controls and required targets. Endpoints are source-derived and await retail
validation; the existing retail regression traces remain applicable upstream.

2026-10-03 strict-propagation switch search: saved title-4997 prefix selected
20 Cropdale switches (1 before Agnea, 19 after), zero Eastern Escape inputs,
and 22 downstream boost inputs, total cost 148 including 18 paid switches.
Extension benchmark: 114.5 ms versus 23.3 ms without switch candidates and
about 16.8 s for the former joint optimizer. These are local single samples;
full request also includes earlier route search. Endpoints await retail validation.

Correction: the first-safe Canalbrine opening must not freeze out Northern Cait.
For each Cropdale candidate, keep the propagated Western fallback and actively
search Canalbrine boosts for safe Northern Cait. Compare complete water/coast
input costs; bound further Northern candidates by the current complete cost.
Northern wins ties, including equal-cost candidates across Cropdale counts.
Saved4997 now selects16 switches,10 Canalbrine repeats,Northern SSM without extra
boosts,and3 Western Escape repeats: cost124 versus prior Western plan148.
Local extension benchmark105ms. This still uses greedy nearest-menu propagation
within each location strategy, not exhaustive cross-menu allocations.

## Harbor continuation

After town entry, continue through exit5160 to `Twn_Sea_2_1_B`/Map125,
whose GameTextEN label is Conning Creek: Harbor. LevelTriggerTable specifies
Harbor trigger02, WALK, no flag mutations or conditional destination. The
shared loader imports Harbor's streamed levels and placement gates, carries
the party/EXP/JP/story/time state, and records a separate `harbor` entry after
`town`. Harbor has no tavern. Hidden battle-backdrop environment callbacks stay
excluded, while its stream constructor remains modeled. Detailed log includes
the resulting RNG, counter and individual draws; no fixed-action Manipulation
card is added. Harbor endpoint awaits retail verification.

Harbor includes a morning-only sidequest NPC placement. Native placement filter
`FUN_140ec4900` trigger cases4/5 require phase1/3 respectively; the loader now
recognizes eMORNING/eEVENING instead of failing on these rows. Manual daytime
is phase2 and does not spawn morning-only NPCs. Their story gates remain active.
