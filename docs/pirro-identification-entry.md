# Pirro round-3 identification route

Branch: `strategy/pirro-r3-identification` in both repositories.

## Player flow

- Keep the earlier route, equipment, soulstones and scripted fights. Earlier RNG
  manipulation is not predicted or searched on this branch.
- Enter Pirro with full HP (standard profile261HP/40SP), then unboosted Darkest
  Night on rounds1 and2.
- First select who acts first in round3: Left=Throné, Right=Pirro. Then enter
  Throné’s current HP and confirm with Enter. Both fields are mandatory before
  any voice observations can be entered.
- On Throné's R3 menu select0->1->2 once. For each2->3 input, record Left for a
  normal line or Right for the special “Pirro!” line. Return3->2 between samples.
- When one candidate remains, the page changes its title, displays the identified
  seed, plays a short tone where browser audio is allowed, and starts simulation.
  Stop at3BP. The hotkeys require page focus; they are not global game hotkeys.
- Current CRT after the final recorded input becomes the seed; advances restart
  at zero. No duplicate initialization or0->3 draws are added. Subsequent R3
  adjustment starts from selected3BP, and existing downstream searches continue.

The identifier searches all2^31 observable CRT states, without a guessed seed
window. Bit31 does not change CRT outputs; the returned seed is canonical.
Its server cache retains only the last exact transcript/candidate set. Undo or
changed transcripts rebuild from the full state space. Observations entered
while filtering queue into the transcript and stale responses cannot launch a
simulation.

## Battle reconstruction

A unique CRT state does not alone identify the earlier battle state. Standard
R1/R2 replay can reach the same observed state through different starts. In
samples these differed by whether Pirro already acted in R3. Further boost
observations cannot resolve identical CRT futures. The visible R3 order resolves
this; the user additionally requires current HP as a consistency check.

Replay candidate initial states1..511 draws before the first recorded boost,
using the fixed full-health profile and standard R1/R2 policy. Retain only exact
matches for the observed state, R3 order, current HP, and complete voice transcript; reject
zero or multiple distinct matching checkpoints rather than selecting arbitrarily.
This is a bounded reconstruction of the supported two-turn prefix, not a bound
on RNG identification or the required boost search. Additional actions before
identification are outside this entry contract.

Preserve HP/SP, damage, AI tables, current/next queues, voice history and other
combat state. Rebase only the draw epoch and unobservable top bit. R3 search is
breadth-first by actual input count, with state+voice+selected-BP deduplication;
accept real victory without the Healing Grape branch. R8 then secures the safe,
Throné-breakable Highroad opening using the existing strict propagation policy.

`data/source/pirro-identification-world.json` is the standard route's authored
world checkpoint, extracted from `outputs/clockbank-adaptive-live-route.json`
(`prediction.post_battle_field.pirro_battle.world_at_entry`). It carries story
flags, residency and the standard14 pre-Pirro EXP. Its old RNG/counter is not used
as the simulation seed. Combat resources/AI are reconstructed independently.
The original inventory profile supplies the same soulstones and one grape.
Additional prior kills, gear changes or a different early route are not supported
by this checkpoint. Real Pirro rewards, latent and subsequent levels propagate.

## Validation

- Exact checkpoint reconstruction/state equality across128 sampled battle seeds,
  using R3 turn order and no HP input.
- Recorded40-voice replay preserves the complete battle state and resets draws0.
- Ambiguous prefixes without order are rejected.
- Explicit selected3BP action matches the ordinary fully boosted action's state;
  unrelated menus reject checkpoint/resume calls without mutation.
- Existing Pirro and boost tests pass.
- Exhaustive identifier -> full downstream route sample completes, with no nested
  simulation errors. Browser testing confirms both arrows, identification at34
  observations, automatic simulation, and the route beginning at Pirro.
- This entry mode still needs retail validation. All earlier core retail fixtures
  remain applicable to the combat code; they do not substitute for that check.
