# Galdera: Chubby victory to Eastern Sai Sands

The second Conning Creek visit goes straight to Western Conning Creek Coast.
Do not visit the tavern or grant its Critical Nut (L) here; the menu-opening and
Steal draws are absent. The planned pickup is deferred until after obtaining the
book in Stormhail Snows: return to Conning Creek by day, steal Critical Nut (L)
from the tavern Head Cook, then switch path action and close. This later errand
is beyond the current executable route, which ends at JP ×100 victory.

Corrected active sequence: Eastern Sai Sands entry -> one Eastern encounter and
100% Escape -> Southern Sai Sands entry -> one Southern encounter and 100%
Escape -> Sai main town. Both openings must be Normal or Preemptive. The Eastern
battle's weakness reveals, Escape draw and field-reset callbacks feed the actual
Southern map load. Both fights appear as separate manipulation cards. Galdera
always discards MapInStorySelect prompts, including Sai, even if a caller passes
an explicit chapter choice; Any% retains its original chapter selections.

The continuation now also loads Southern Sai Sands (`Fld_Dst_2_3`) and predicts
its planned first encounter. The same finisher/Conning timing candidates must
satisfy both coast traversal safety and a Normal or Preemptive Southern opening.
The source encounter catalog supplies the actual formation and danger level;
the battle adapter retains carried levels/stats/supports and enemy knowledge.
At the actual first player menu, execute Escape with no extra boost inputs.
Native `success_percent` must report 100% with the current party, and the real
escape check must succeed. Restore field control, enemy knowledge and both
post-Escape counter-reset callbacks through the shared loader.

Eastern -> Southern traversal has no supplied movement measurement. The route
models the Southern entry and first encounter without inventing an Eastern
encounterless movement guarantee or a second Southern encounter budget.

The Galdera continuation freezes the chosen EXP ×100 routine and all earlier
controls. It searches the last player's actual Thunder Soulstone (L) command
menu using the battle engine's boost frontier. Inputs end at zero BP, and the
whole fight is replayed: victory, EXP ×100 and all four living travelers remain
required. Reward/drop callbacks and level growth then feed the field loader.

After the victory, switch to day before entering Gravell. The player confirmed
the fight-to-Gravell leg has no encounter; no unmeasured movement is invented.
Tag Gravell, fast travel to the previously tagged Conning Creek, then enter
Western Conning Creek Coast. For each finisher candidate, compare normal daytime
travel with switching to night immediately upon landing in Conning Creek and
back to day immediately upon coast entry. Both switches execute real placement
and flag callbacks. The player assigns this landing/entry pair zero time cost.

Search fewest additional boost presses first; at equal cost, choose the largest
safe running share. Prefer no switch pair when running shares tie. Never reopen
the selected Guards/BG plan to reduce the downstream cost. If the finishing
actor has no BP, only the two travel timing options are available.

The measured coast leg is 113 walking updates or 107 running updates. Add three
updates to each endpoint (116/110). Read the exact daytime land decrement from
the encounter catalog, including equipped Evasive Maneuvers. Enumerate whole
running updates, rounding the remaining walking updates up using the measured
endpoint ratio; require cost strictly less than the starting counter. Display
the running distance share rounded down to 0.1%, followed by the walking share.
Detailed log retains the initial counter, per-speed costs and positive exit
counter. Movement consumes no CRT. Eastern Sai Sands then uses the shared map
loader, including both counter resets and all imported stream/NPC callbacks.

Mixed distance shares still depend on the measured endpoint ratio and the three
updates of leeway; they are not a frame-by-frame movement replay. These new
endpoints await retail validation.

To regenerate loading data, run `tools/import-town-loading.cjs` followed by
`tools/import-travelers-bag.cjs` (which restores the separately imported bag
scripts). The source importer now includes Eastern Sai Sands, `Fld_Dst_2_4`.
