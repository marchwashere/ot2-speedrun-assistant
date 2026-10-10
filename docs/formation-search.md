# Formation search

The active route no longer requires a lone one-shield enemy or a zero-BP break
on Highroad or Flats. The two obsolete Settings controls are removed, and old
saved Required preferences for them are ignored during execution. No-surprise
remains mandatory for every planned encounter.

After cleric purchases the route switches back to day exactly once. No
post-Osvald time switches are searched or displayed; the search uses battle
boosts/breaks for the later targets while recruitment retains its actual draws.

For the early-clerics route, `formation_search` freezes the selected Pirro R3
and earlier decisions, enumerates actual R8 boost paths and successful Highroad
Escape strategies, then evaluates the carried Flats/Harbor/Flats encounter,
clerics, recruitment and Snows continuation. Compare extra R8 inputs plus
Highroad inputs plus downstream adjustment cost, using the existing model:
one button press = 1, break = 10, time switch = 7. Any supported formation may
win this comparison. A supported break remains optional; multi-enemy packs may
use direct Escape. Multi-enemy combat/break routines are not invented.

The original complete valid route supplies an incumbent upper bound. Candidate
prefixes with no possibility of improving it are pruned; Required outcomes have
no arbitrary retry limit. Equivalent field states are reused only when the
earlier prefix was no more expensive. State keys include the complete carried
world, CRT state and draw count. Rejected candidates cannot mutate execution.

Existing optional outcomes achieved by the initial route are preserved. The
selected path is replayed through the real core and loader, including EXP and
the actual field counter. Export retains exact R8 inputs; replay searches only
the downstream portion. Detailed log/export includes baseline and selected
cost. This comparison covers the supported R8/Highroad inputs and existing
downstream strategies, not every possible battle action or earlier decision.
New route endpoints still require retail verification.
