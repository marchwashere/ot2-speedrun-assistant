# Route search boundaries

## Required outcomes and optional improvements

An outcome with **Required outcome** checked is a constraint. At each control
point, solve all checked targets and their prerequisites together, then try
optional improvements. An unchecked numeric value is an additional repeat
budget, default 4 (0..256). If an optional target cannot be achieved, retain the
required plan and report the unmet optional target. Unused allowance is not
transferred between optional targets. Each toggle preserves its numeric value.

The Highroad and Flats **Boost repeats before trying a break** fields are
strategy preferences, not outcome budgets. First look for a complete valid
direct-Escape plan within that threshold; then consider an actual immediate
single-enemy break and next-menu Escape; otherwise continue direct boost search.
All strategies must satisfy every required route target. **Boost only** skips
the break fallback. Failed trial actions never enter the selected route.

This is optimization per control point, not global route-time optimization or
backtracking across earlier controls. If a required target cannot be reached
from the frozen earlier plan, report it as unavailable; do not present the
partial prediction as a complete valid route.

Settings groups targets by the last controllable event on this route:

| Control point | Searched action | Targets |
| --- | --- | --- |
| First battle | Extra boost/cancel inputs at one party menu, ending at the fixed action's BP | Safe checkpoint walk, then one enemy action in fight two |
| Second fight | Extra boost/cancel inputs at one party menu, ending at the fixed action's BP | First mansion encounter is Normal or Preemptive |
| Mansion encounter | Boost/cancel at Throné's first menu, ending at 0 BP, then Escape | First Escape succeeds and the remaining run reaches Pirro without another encounter |
| Pirro R3 | Boost/cancel ending at 3 BP Darkest Night | Victory, victory without Healing Grape |
| Pirro R8 | Boost/cancel ending at 3 BP swords | Highroad encounter is Normal or Preemptive; preserve the R3 plan and victory |
| Highroad encounter | First-menu boost/cancel → Escape, or immediate break → Escape | Escape, walk safely to Harbor, avoid surprise after Harbor return |
| Flats encounter | First-menu boost/cancel → Escape, or immediate break → Escape | Escape, run to Abandoned Village without another encounter, then tag/return and walk safely to Southern Cape Cold Snows |

Required downstream targets at the same control point include their
prerequisites, regardless of the prerequisites' optional budgets:

- Required second fight: jointly search safe checkpoint walking and exactly
  one enemy action; a zero optional checkpoint allowance cannot block this.
- Required no-grape: jointly search victory and the no-grape branch; a zero
  optional victory allowance cannot block this.
- Required Flats non-surprise: jointly search successful Escape, safe Harbor
  walk, and the returned-Flats encounter. The optional walking allowance and
  break preference cannot override these required conditions.

Optional targets are attempted in route order from the required plan. They
preserve preceding achieved targets. Search costs are attributed to the joint
required plan first; only subsequent additions count as optional repeats.

Each downstream search freezes the earlier selected plan. RNG and progression
continue through every event; the second search does not treat the battle exit
as a fixed offset. It replays battle mechanics, rewards, both city maps, parlor
visits, steals, mansion entry, lookout Ambush and field encounter selection.
The Escape search then starts from that selected candidate's field frame.

Second-fight adjustment preserves victory and any achieved one-enemy-action and
order conditions. It checks all reached party menus with available BP. Inputs
minimize extra button presses, then extra CRT draws, then prefer the latest
menu. Normal action boosts are included in the displayed input path, excluded
from extra-repeat counts, and executed exactly once. No adjustment is chosen
when the baseline opening already passes. A finite-budget failure keeps the
baseline and reports the unmet target. Required has no fixed numeric budget.

API: outcome parameters accept `4|required`; legacy `unlimited` still means
required. Strategy parameters accept `4|boost_only` (legacy `unlimited` also
means boost-only). JSON keeps the original null caps for compatibility and adds
`crt_setup.search_policies` with explicit required/optional modes. Final
`required_outcomes` reports whether all requested constraints actually hold and
lists missing/unreachable targets. For example,
`mansion_approach_max_boost_repeats=required` controls mansion surprise avoidance.
Exports carry `second_boost_menu`, `second_boost_advances`, and optional
`second_boost_repeat_cap` to replay the selected input path. Omitting the second
plan searches; an explicit zero plan replays without adjustments.

Regression: seed 0/title advance 4904 with default earlier budgets initially
predicts Surprise. One extra draw at second-fight party menu 6 (two added button
presses) predicts Normal while keeping the one-enemy-action result. The opening
battle is identical, downstream checkpoints change, and exported replay agrees.
This is simulator regression coverage, not live-game validation of the new
second-fight adjustment or mansion Escape stage.
# Osvald extension

The Flats encounter also controls the fresh Southern Cape Cold Snows counter
and the 70-update walk to Osvald. `flats_osvald_max_boost_repeats` defaults to
Required outcome, with an independent optional numeric allowance. Required
Osvald walking includes the prior post-village walk as a prerequisite even if
that target's own budget is optional. Every direct/break candidate must meet
all required targets. Optional Osvald improvements preserve earlier achieved
walks and use additional repeats beyond the actual selected plan.
