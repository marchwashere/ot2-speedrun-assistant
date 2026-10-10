# Pirro round-3 identification route (work in progress)

Branch: strategy/pirro-r3-identification. The existing main route remains unchanged.

Confirmed scope:
- Do not search/manipulate the route before Pirro round3.
- Rounds1 and2 use unboosted Darkest Night, with existing equipment and items.
- Identify CRT using the standalone Pirro voice identifier at round3.
- Start planning from immediately after the last recorded2->3 boost:3BP selected.
- Do not replay battle initialization or charge normal0->3 boosts at this boundary.
- Preserve character progression, inventory and story state for downstream events;
  dropping early manipulation does not mean resetting progression.

The identifier exports initial/current CRT states and modulo2^31 positions.
Current means after the last recorded boost. Initial means before the first
recorded2->3 boost, not battle initialization. Retain the observation sequence
for reconstructing the voice state and validating its draw count.

A current CRT state alone is not a complete battle checkpoint. Round1/2 damage,
HP/SP, generated current/next turn queues, shuffled Pirro AI tables, and used
voice history must be reconstructed or carried from a validated checkpoint.
Never manufacture these by simulating an unrelated seed and overwriting its RNG.

Pending user input: fully restore HP/SP before Pirro, or supply actual incoming
resources. Once established, implement and validate checkpoint reconstruction,
search continuation and the replacement UI. This document is a specification,
not a claim that the new simulation entry point is already implemented.
