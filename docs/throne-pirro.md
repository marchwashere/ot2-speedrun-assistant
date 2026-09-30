# Mansion Escape to Pirro

`src/pirro.rs` consumes the successful mansion Escape solution. The remaining
run uses no CRT calls and leaves a positive counter at the trigger. Carry HP/SP
from Escape, reset battle BP to one, retain the second battle's inventory and
add the four soulstones stolen by the route. No invented recovery or items.

MS_TOU_10_1800 on map 50 sets flag 8017 and starts rich sequence
RI_MS_TOU_10_1500. Its StartBattle key hands off to EVM_BOS_THI_C01_010 and names
MS_TOU_10_1900 as the continuation after combat. This is the same loaded map.

Native StartEventBattle (140eb2700) writes command 600 and the +0x69e handoff
flag. The skip handler (140eb67a0) returns zero while that flag is set;
StartBattleStart (140e965b0) suspends the parent. Consequently the event's
EncountReset=true does not run FinishPre before battle. The projected transition
is +0 before initialization. `tools/audit-pirro-transition.cjs` regenerates the
asset/native evidence and assertions in data/source/throne-pirro-transition.json.

The standalone core simulates all player/enemy actions. The route captures its
initial state, command menus, action results, callback trace, policy branch,
last CRT boundary and any error. Victory alone exposes after_victory, after
drops. No 1900 cleanup, field return or post-boss map is predicted here.

Manipulation shows only boost adjustment and "early latent" / "late latent".
Outcome, round count, HP, CRT and the known routine stay in Detailed log/export.
Detailed log includes every action and command-menu
checkpoint, including the latent extra menu. Counts add the boss entry offset
exactly once. Export contains the same prediction.

Run simulation now searches boost inputs at Throné's R3 command menu for
both victory and Throné first in the R8 preview (no Healing Grape). Earlier
route choices and R1/R2 remain frozen. The Pirro Settings card has separate
victory and no-grape limits, each defaulting to four with its own Unlimited
option. First find the cheapest winning plan within the victory limit.
No-grape may then use its allowance beyond that plan's actual repeat count;
unused victory allowance is not transferable. If victory cannot be secured,
do not run the no-grape search. If no-grape cannot be secured, retain the
winning plan, including its grape if required. Ordinary final boosting to
3 BP is excluded. Exports preserve both limits and both search-stage results.
The legacy joint limit migrates to no-grape only; victory defaults to four.

Search actual button paths breadth first, replaying the battle core and
minimizing total button presses, then actual CRT draws. The boss-specific
THIEF_eON_BOOST_Lv3_ENG_BOS_THI_C01_010 voice is a 50% candidate before the
generic max-boost voice: 2->3 consumes three OR four calls. Fixed-cost offset
search from the earlier fights is invalid here. Core preview carries the
voice-used mask; equivalent search nodes retain selected BP, climb direction,
and the repetition budget. Exports store the complete selected BP path in
pirro_boost_inputs, so no approximate offset reconstruction is involved.

Regression tests repair the known early-critical defeats, exercise the
healing/no-healing branches, compare all shorter button paths, and check
trace boundaries, independent zero/Unlimited budgets, winning fallback,
actual-cost budget accounting and exact export replay.
These checks establish simulator consistency, not live-game alignment.

The user-reported 2026-09-28 startup checkpoint is seed 0: pre-cutscene 5595
to first command menu 5631. The transition remains +0; the core now applies
the group's fixed Throné battle-start speaker instead of an extra random
speaker draw. The resulting initialization is +36. An integration regression
checks that boundary through this route. The subsequent +5678 live capture
matches all six player menus and all 203 CRT calls after correcting the
new-weakness reaction, phase-change frame draw, and item callback gate. It
reproduces the reported 38-damage R4 critical and R7 defeat, ending at5881 /
0x1FB4FF1B. GetActionCommandForItem enables the ability-camera flag, which
suppresses PlayFinishBlow's timer; the generic USE_ITEM record alone was
misleading. The core imports that scalar gate without running graphics.
No route offset is applied. Winning and boost-search paths remain source-derived
pending live captures; prior odds need recomputing after these corrections.
The entry5595 user-reported R7 menu at5798 is now covered by the core:
Pirro must not run the normal-enemy full-BP reset/threshold lottery. The
assistant uses that corrected core directly, without a route offset.

Latent unlock sets55/80. The no-grape policy keeps R8 max-boost swords when
latent is unavailable and moves activation to R9 for Wind + Ice, then R10
sword. Both searches and exact replays execute this core branch. Manipulation
shows "late latent" when delayed and "early latent" otherwise; Detailed log contains the actual actions
and both R9 menus. At entry5595, the corrected policy still ends at5896, with
R8 menu5806 and R9 menus5832/5836; this remains a prediction, not a live trace.

Source-derived with partial boss coverage, awaiting Escape verification. Some critical/order
rolls cause the supplied routine to lose; report defeat instead of a winning
RNG state. Longer alternative policies reaching the unsupported charged
multi-order phase stop explicitly in the core. See the battle repository's
docs/throne-pirro.md for the combat port and its limits.
