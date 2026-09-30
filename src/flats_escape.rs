//! Harbor return -> first Flats encounter -> Escape -> Abandoned Village entrance.
//! World/route orchestration only; all battle draws come from the battle crate.
use crate::{
    CrtRng, Result, SimError,
    encounter::{self, Boundary, PredictionOptions},
    ensure,
    highroad::{self, Escape, FieldReturn},
    osvald,
    route::MovementBudget,
    village_return,
    world::WorldState,
};
use ot2_battle_engine::escape::Situation;
use serde::Serialize;

/// Measured ~165 running updates, padded by the user to 170. Starts on returning
/// from harbor and ends at the Abandoned Village transition, not its loaded map.
pub const RUN_UPDATES: u32 = 170;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Settings {
    pub max_boost_repeats: Option<u32>,
    /// None requires the post-village walk; finite values are optional extra repeats.
    pub snow_max_boost_repeats: Option<u32>,
    pub osvald_max_boost_repeats: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            max_boost_repeats: Some(4),
            snow_max_boost_repeats: None,
            osvald_max_boost_repeats: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SnowSearch {
    pub max_extra_boost_repeats: Option<u32>,
    pub base_repeats: u32,
    pub added_repeats: u32,
    pub tested_candidates: u64,
    pub target_met: bool,
    pub status: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Prediction {
    pub arrival: CrtRng,
    pub settings: Settings,
    pub level: u32,
    pub encounter: encounter::Prediction,
    pub before_battle_initialization: CrtRng,
    pub run_updates: u32,
    pub first_encounter_update: u32,
    pub remaining_updates: u32,
    pub required_counter: u32,
    pub escape: Option<Escape>,
    pub field_return: Option<FieldReturn>,
    pub remaining_run: Option<MovementBudget>,
    pub at_village_entrance: Option<CrtRng>,
    pub world_at_village_entrance: Option<WorldState>,
    pub village_return: Option<village_return::Prediction>,
    pub village_visit: Option<village_return::Visit>,
    pub early_cait: Option<crate::early_cait::Prediction>,
    pub early_searches: Vec<SnowSearch>,
    pub early_weighted_input_cost: Option<u32>,
    pub early_tested_projections: u64,
    pub snow_search: Option<SnowSearch>,
    pub osvald: Option<osvald::Prediction>,
    pub osvald_search: Option<SnowSearch>,
    pub status: &'static str,
    pub verification: &'static str,
}

/// Field-only selection, shared with the upstream Highroad search. No Escape
/// search is nested inside candidate selection.
pub fn first_encounter(
    arrival: CrtRng,
    world: &WorldState,
    level: u32,
) -> Result<(CrtRng, encounter::Prediction)> {
    ensure(
        world.map == "Fld_Cty_1_2"
            && world.time == 2
            && world.first_encounter
            && world.party == [7]
            && world.visited_maps.contains("Fld_Cty_1_1"),
        "Flats approach requires solo daytime Throné after tagging harbor and returning",
    )?;
    let options = PredictionOptions {
        map: world.map.clone(),
        zone: "ENT_FLD_CTY_1_2_LND".into(),
        period: "DayTime".into(),
        progress: 1,
        first_encounter: true,
        boundary: Boundary::CurrentCounter,
        current_counter: Some(world.encounter_counter),
        party_levels: vec![level],
        dash: true,
        ..Default::default()
    };
    let mut entry = arrival;
    let prediction = encounter::catalog().predict(&mut entry, &options)?;
    Ok((entry, prediction))
}

/// Actual first-action break, after rare replacement and opening hit draws.
pub fn unboosted_opening_break(entry:CrtRng, encounter:&encounter::Prediction, level:u32) -> Result<bool> {
    if encounter.enemies.len()!=1 || !matches!(encounter.situation.as_deref(),Some("Normal"|"Preemptive")) {
        return Ok(false);
    }
    let id=encounter.enemies[0].enemy.id.as_str();
    if !matches!(id,"ENE_NML_FLD_CTY_010"|"ENE_NML_FLD_CTY_120") {return Ok(false);}
    let situation=if encounter.situation.as_deref()==Some("Preemptive") {
        Situation::Preemptive
    } else {Situation::Normal};
    Ok(ot2_battle_engine::field_escape::first_action_break(entry,situation,level,&[id])?
        .is_some_and(|b|b.boost==0))
}

pub fn after_harbor(
    h: &crate::harbor::Prediction,
    level: u32,
    settings: Settings,
) -> Result<Prediction> {
    predict(h.after, &h.world_after, level, settings)
}

pub fn predict(
    arrival: CrtRng,
    world: &WorldState,
    level: u32,
    settings: Settings,
) -> Result<Prediction> {
    predict_to_boundary(arrival, world, level, settings, false, None)
}

pub fn until_first_village(arrival:CrtRng, world:&WorldState, level:u32, settings:Settings) -> Result<Prediction> {
    predict_to_boundary(arrival, world, level, settings, true, None)
}

pub fn until_osvald_cait(arrival:CrtRng, world:&WorldState, level:u32, settings:Settings,
    early:crate::early_cait::Settings) -> Result<Prediction> {
    ensure(early.clerics_max_boost_repeats.is_none_or(|n|n<=256)
        && early.cait_max_boost_repeats.is_none_or(|n|n<=256), "Early route caps must be 0..256 or Required")?;
    predict_to_boundary(arrival, world, level, settings, false, Some(early))
}

fn predict_to_boundary(arrival:CrtRng, world:&WorldState, level:u32, settings:Settings, stop_at_village:bool,
    early:Option<crate::early_cait::Settings>) -> Result<Prediction> {
    ensure(
        settings.max_boost_repeats.is_none_or(|n| n <= 256)
            && settings.snow_max_boost_repeats.is_none_or(|n| n <= 256)
            && settings.osvald_max_boost_repeats.is_none_or(|n| n <= 256),
        "Flats limits must be 0..256, Required outcome or Boost only",
    )?;
    let (entry, encounter) = first_encounter(arrival, world, level)?;
    let trigger = encounter
        .counter_updates
        .ok_or_else(|| SimError("Missing Flats encounter distance".into()))?;
    let decrement = encounter.decrement.unwrap();
    ensure(
        trigger <= RUN_UPDATES,
        "Village approach ends before this encounter",
    )?;
    let remaining = RUN_UPDATES - trigger;
    let required_counter = remaining * decrement + 1;
    let counter = &encounter::catalog().maps[&world.map].counter;
    ensure(
        required_counter <= counter.min + counter.range_a - 1 + counter.range_b - 1,
        "Remaining village run exceeds the maximum reset counter",
    )?;
    let situation = match encounter.situation.as_deref() {
        Some("Normal") => Situation::Normal,
        Some("Preemptive") => Situation::Preemptive,
        Some("Surprise") => Situation::Surprise,
        _ => return Err(SimError("Unknown Flats encounter situation".into())),
    };
    let mut result = Prediction {
        arrival,
        settings,
        level,
        encounter,
        before_battle_initialization: entry,
        run_updates: RUN_UPDATES,
        first_encounter_update: trigger,
        remaining_updates: remaining,
        required_counter,
        escape: None,
        field_return: None,
        remaining_run: None,
        at_village_entrance: None,
        world_at_village_entrance: None,
        village_return: None,
        village_visit: None,
        early_cait: None,
        early_searches: vec![],
        early_weighted_input_cost: None,
        early_tested_projections: 0,
        snow_search: None,
        osvald: None,
        osvald_search: None,
        status: "Surprise opening: increase the Highroad Flats-approach allowance",
        verification: "Source-derived, awaiting live verification. 170 running updates from harbor return to Abandoned Village entrance (measured ~165 plus five-update allowance). Daytime, unchanged equipment, solo route level, no intervening CRT actions. Search minimum-cost direct Escape or an actual single-enemy break with boosts at either command menu. Boost input cost1; additional break cost10. Every selected strategy must also leave a strictly positive counter at the village entrance. A break whose reset is too short is rejected. Uses the shared field-restoration callbacks; never just offsets the final RNG. The village-approach boundary is before loading; the separate village_return then tags the village, returns to Flats and predicts the padded walk to Southern Cape Cold Snows.",
    };
    if situation == Situation::Surprise {
        return Ok(result);
    }
    let volume = &encounter::catalog().volumes[result.encounter.volume.as_ref().unwrap()];
    let danger = volume
        .encounters
        .iter()
        .rev()
        .find(|r| r.table != "None" && r.progress <= 1)
        .ok_or_else(|| SimError("Missing Flats danger row".into()))?
        .danger;
    let enemies: Vec<_> = result
        .encounter
        .enemies
        .iter()
        .map(|e| e.enemy.id.as_str())
        .collect();
    let restore = |attempt: &ot2_battle_engine::field_escape::FieldEscapeAttempt| {
        highroad::restore_field(world, attempt, "Fld_Cty_1_2", "Flats Escape")
    };
    if let Some(early) = early {
        let project = |a:&ot2_battle_engine::field_escape::FieldEscapeAttempt, cleric_extra, osvald_switches| {
            let f=restore(a)?;
            if f.world_after.encounter_counter < required_counter { return Ok(None); }
            let mut w=f.world_after;
            w.encounter_counter-=remaining*decrement;
            crate::early_cait::project_with_policy(f.after,w,level,cleric_extra,osvald_switches,early.require_cait).map(Some)
        };
        let caps=[early.clerics_max_boost_repeats, settings.snow_max_boost_repeats,
            if early.require_cait {settings.osvald_max_boost_repeats} else {Some(0)}, early.cait_max_boost_repeats];
        let mut required=caps.map(|cap|cap.is_none());
        // Cache immutable projections across weighted-cost layers. Search clones
        // the untouched command menu; rejected time switches never leak state.
        use std::{cell::{Cell,RefCell},collections::BTreeMap,rc::Rc};
        let cache=RefCell::new(BTreeMap::<(u32,u64,u32,u32),Option<Rc<crate::early_cait::Prediction>>>::new());
        let tested=Cell::new(0u64);
        let cached = |a:&ot2_battle_engine::field_escape::FieldEscapeAttempt,k,m| -> Result<Option<Rc<crate::early_cait::Prediction>>> {
            let key=(a.after.state,a.after.draws,k,m);
            if let Some(p)=cache.borrow().get(&key) {return Ok(p.clone());}
            let p=project(a,k,m)?.map(Rc::new);
            cache.borrow_mut().insert(key,p.clone()); tested.set(tested.get()+1); Ok(p)
        };
        let search=RefCell::new(crate::escape_search::FieldSearch::new(entry,situation,level,&enemies,danger)?);
        let choose = |ceiling:Option<u32>,goals:[bool;4]| -> Result<Option<(Escape,u32,u32,u32)>> {

            let mut cost=0u32;
            loop {
                if ceiling.is_some_and(|c|cost>c) {return Ok(None);}
                // Equal weighted cost: prefer fewer switches, then fewer at the
                // clerics. The mandatory night/day shop pair is a constant +14.
                for switches in 0..=cost/crate::early_cait::TIME_SWITCH_INPUT_COST {
                    let boost_cost=cost-switches*crate::early_cait::TIME_SWITCH_INPUT_COST;
                    for candidate in search.borrow_mut().candidates(boost_cost,settings.max_boost_repeats.is_some(),None)? {
                        let a=candidate.attempt.as_ref().unwrap();
                        let Some(prefix)=cached(a,0,0)? else {continue};
                        let base=crate::early_cait::outcomes_for(&prefix,early.require_cait);
                        if (goals[0]&&!base[0]) || (goals[1]&&!base[1]) {continue;}
                        for cleric_extra in (0..=switches).step_by(2) {
                            let osvald_switches=switches-cleric_extra;
                            let Some(p)=cached(a,cleric_extra,osvald_switches)? else {continue};
                            if crate::early_cait::outcomes_for(&p,early.require_cait).iter().zip(goals).all(|(met,needed)|!needed || *met) {
                                return Ok(Some((candidate,cleric_extra,osvald_switches,cost)));
                            }
                        }
                    }
                }
                cost=cost.checked_add(1).ok_or_else(||SimError("Weighted early route search exhausted representable cost".into()))?;
            }
        };
        let (mut escape,mut cleric_extra,mut osvald_switches,mut cost)=choose(None,required)?
            .ok_or_else(||SimError("No required early route plan".into()))?;
        for i in 0..4 {
            let Some(a)=&escape.attempt else {break};
            let outcomes=crate::early_cait::outcomes_for(&cached(a,cleric_extra,osvald_switches)?.unwrap(),early.require_cait);
            // Optional improvements preserve all already achieved targets.
            for j in 0..4 {required[j] |= outcomes[j];}
            let base=escape.boost_repeat_count;
            let mut met=outcomes[i];
            let before_tested=tested.get();
            let mut added=0;
            if !met && let Some(extra)=caps[i] {
                let mut goals=required; goals[i]=true;
                // Numeric optional budgets retain their two-input-pair units,
                // now including equivalent time-switch cost instead of boosts only.
                if let Some((candidate,k,m,c))=choose(Some(cost.saturating_add(2*extra)),goals)? {
                    added=candidate.boost_repeat_count.saturating_sub(base);
                    escape=candidate; cleric_extra=k; osvald_switches=m; cost=c; required[i]=true; met=true;
                }
            }
            result.early_searches.push(SnowSearch {max_extra_boost_repeats:caps[i],base_repeats:base,
                added_repeats:added,tested_candidates:tested.get()-before_tested,target_met:met,
                status:if met {"Target met; preceding outcomes preserved"} else {"Optional target not met within budget"}});
        }
        if let Some(a)=&escape.attempt {
            let f=restore(a)?;
            let budget=MovementBudget::new(f.world_after.encounter_counter,remaining,decrement);
            let mut exit=f.world_after.clone(); exit.encounter_counter=budget.counter_before_reset.unwrap();
            result.at_village_entrance=Some(f.after); result.world_at_village_entrance=Some(exit);
            result.early_cait=project(a,cleric_extra,osvald_switches)?;
            let final_outcomes=crate::early_cait::outcomes_for(result.early_cait.as_ref().unwrap(),early.require_cait);
            for (s,met) in result.early_searches.iter_mut().zip(final_outcomes) {
                s.target_met=met;
                s.status=if met {"Target met in the selected joint plan"} else {"Optional target not met within budget"};
            }
            result.remaining_run=Some(budget); result.field_return=Some(f);
            result.early_weighted_input_cost=Some(cost+2*crate::early_cait::TIME_SWITCH_INPUT_COST);
            result.early_tested_projections=tested.get();
            result.status="Early clerics / Osvald / first Snows encounter evaluated";
        }
        result.escape=Some(escape);
        result.verification="Weighted search over direct Escape or break with R1/R2 boosts, post-purchase complete time cycles and post-recruitment time switches, minimizing boost inputs +10 per break +7 per switch including the mandatory night/day shop pair. Required outcomes gate every candidate jointly; numeric optional budgets add two weighted cost units per input-pair allowance and preserve achieved outcomes. Each candidate replays loader/time/NPC/movement/recruitment and actual final-phase encounter selection. No Cape Cold continuation.";
        return Ok(result);
    }
    let project = |attempt: &ot2_battle_engine::field_escape::FieldEscapeAttempt| -> Result<Option<village_return::Prediction>> {
        let field = restore(attempt)?;
        if field.world_after.encounter_counter < required_counter { return Ok(None); }
        let mut at_exit = field.world_after;
        at_exit.encounter_counter -= remaining * decrement;
        village_return::with_world_state(field.after, at_exit).map(Some)
    };
    let choose = |total_limit, require_snow, require_osvald| {
        highroad::escape_plan_with_limit(
            entry,
            situation,
            level,
            &enemies,
            danger,
            settings.max_boost_repeats,
            total_limit,
            |a| {
                if require_snow || require_osvald {
                    let Some(p) = project(a)? else {
                        return Ok(false);
                    };
                    if !p.snow_reach.walking.reaches_reset {
                        return Ok(false);
                    }
                    if require_osvald {
                        Ok(osvald::after_village(&p)?.is_some_and(|o| o.walking.reaches_reset))
                    } else {
                        Ok(true)
                    }
                } else {
                    Ok(restore(a)?.world_after.encounter_counter >= required_counter)
                }
            },
        )
    };
    // Required goals gate both direct Escape and break strategies. Optional
    // walking starts from the actual escape/village-run plan, never its cap.
    let require_osvald = !stop_at_village && settings.osvald_max_boost_repeats.is_none();
    let require_snow = !stop_at_village && (settings.snow_max_boost_repeats.is_none() || require_osvald);
    let mut escape = choose(None, require_snow, require_osvald)?;
    if stop_at_village {
        result.verification="Source-derived route through first Abandoned Village entry/tag and tutorial dismissal. Search only first Escape and a safe village run; no lookahead toward Snows, Osvald or subsequent events. Shared Loader evaluates the village load and carries CRT, counters, progression and residency. No return to Flats is simulated.";
        if let Some(attempt) = &escape.attempt {
            let field = restore(attempt)?;
            let budget = MovementBudget::new(field.world_after.encounter_counter,remaining,decrement);
            ensure(budget.reaches_reset,"Selected Flats plan cannot reach the village")?;
            let mut at_exit=field.world_after.clone();
            at_exit.encounter_counter=budget.counter_before_reset.unwrap();
            result.village_visit=Some(village_return::visit(field.after,at_exit.clone())?);
            result.at_village_entrance=Some(field.after);
            result.world_at_village_entrance=Some(at_exit);
            result.remaining_run=Some(budget);
            result.field_return=Some(field);
            result.status="First Abandoned Village visit complete; route stops here";
        }
        result.escape=Some(escape);
        return Ok(result);
    }
    if let Some(attempt) = &escape.attempt {
        let base = escape.boost_repeat_count;
        let mut met = project(attempt)?.unwrap().snow_reach.walking.reaches_reset;
        let mut search = SnowSearch {
            max_extra_boost_repeats: settings.snow_max_boost_repeats,
            base_repeats: base,
            added_repeats: 0,
            tested_candidates: escape.tested_candidates,
            target_met: met,
            status: "Escape, village run and post-village walk satisfied",
        };
        if !met && let Some(extra) = settings.snow_max_boost_repeats && extra>0 {
            let candidate = choose(Some(base.saturating_add(extra)), true, require_osvald)?;
            search.tested_candidates += candidate.tested_candidates;
            if candidate.attempt.is_some() {
                search.added_repeats = candidate.boost_repeat_count
                    .saturating_sub(base);
                escape = candidate;
                met = true;
            } else {
                search.status = "Optional post-village walk not found within budget; keeping Escape and village run";
            }
        }
        search.target_met = met;
        result.snow_search = Some(search);
    }
    if let Some(attempt) = &escape.attempt {
        let base = escape.boost_repeat_count;
        let village = project(attempt)?.unwrap();
        let keep_snow = require_snow || village.snow_reach.walking.reaches_reset;
        let mut met = osvald::after_village(&village)?.is_some_and(|o| o.walking.reaches_reset);
        let mut search = SnowSearch {
            max_extra_boost_repeats: settings.osvald_max_boost_repeats,
            base_repeats: base,
            added_repeats: 0,
            tested_candidates: escape.tested_candidates,
            target_met: met,
            status: "Safe village run and both walks through Osvald satisfied",
        };
        if !met && let Some(extra) = settings.osvald_max_boost_repeats && extra>0 {
            let candidate = choose(Some(base.saturating_add(extra)), keep_snow, true)?;
            search.tested_candidates += candidate.tested_candidates;
            if candidate.attempt.is_some() {
                search.added_repeats = candidate.boost_repeat_count
                    .saturating_sub(base);
                escape = candidate;
                met = true;
            } else {
                search.status =
                    "Optional Osvald walk not found within budget; keeping the preceding plan";
            }
        }
        search.target_met = met;
        result.osvald_search = Some(search);
    }
    if let Some(attempt) = &escape.attempt {
        let field = restore(attempt)?;
        let budget = MovementBudget::new(field.world_after.encounter_counter, remaining, decrement);
        ensure(
            budget.reaches_reset,
            "Selected Flats plan cannot reach the village",
        )?;
        let mut world_at_exit = field.world_after.clone();
        world_at_exit.encounter_counter = budget.counter_before_reset.unwrap();
        result.at_village_entrance = Some(field.after);
        result.world_at_village_entrance = Some(world_at_exit);
        result.village_return = project(attempt)?;
        result.osvald = osvald::after_village(result.village_return.as_ref().unwrap())?;
        if let Some(s) = &mut result.snow_search {
            s.target_met = result
                .village_return
                .as_ref()
                .unwrap()
                .snow_reach
                .walking
                .reaches_reset;
            if s.target_met {
                s.status = "Escape, village run and post-village walk satisfied";
            }
        }
        result.remaining_run = Some(budget);
        result.field_return = Some(field);
        result.status = "Escape succeeds; run to Abandoned Village without a second encounter";
    } else {
        result.status = "No joint Escape/run solution in the CRT cycle";
    }
    result.escape = Some(escape);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world;
    use highroad::EscapeMethod;
    #[test]
    fn cait_fork_replays_joint_targets_and_freezes_the_incoming_encounter() {
        let w=world();
        let entry=CrtRng {state:0,draws:6050};
        let settings=Settings {max_boost_repeats:None, ..Default::default()};
        let first=until_first_village(entry,&w,5,settings).unwrap();
        let p=until_osvald_cait(entry,&w,5,settings,crate::early_cait::Settings {require_cait:true,..Default::default()}).unwrap();
        assert_eq!(p.before_battle_initialization,first.before_battle_initialization);
        assert_eq!(p.escape.as_ref().unwrap().menu.rng,first.escape.as_ref().unwrap().menu.rng);
        let selected=p.escape.as_ref().unwrap();
        let a=selected.menu.clone().attempt(&selected.inputs.as_ref().unwrap().levels,5).unwrap();
        assert_eq!(a.after,selected.attempt.as_ref().unwrap().after);
        let early=p.early_cait.as_ref().unwrap();
        assert_eq!(crate::early_cait::outcomes(early),[true;4]);
        assert!(p.osvald.is_none() && p.village_return.is_none());
        assert!(early.cleric_reach.walking.reaches_reset);
        assert_eq!(early.remaining_walk.as_ref().unwrap().updates,30);
        assert_eq!(early.movement.walking.cost,early.cleric_reach.walking.cost+early.remaining_walk.as_ref().unwrap().cost);
        let mut l=world::Loader::new(world::catalog(),early.world_at_flats.clone(),early.after_flats).unwrap();
        l.state.encounter_counter=early.cleric_reach.walking.counter_before_reset.unwrap();
        let counter=l.state.encounter_counter;
        l.set_time(4,"test night").unwrap(); l.refresh_bar().unwrap(); l.refresh_npcs("test night").unwrap();
        assert_eq!(l.rng,early.after_night.unwrap());
        assert!(crate::black_market::clerics_present(&l.state));
        for _ in 0..early.post_cleric_switches {
            let time=if l.state.time==4 {2} else {4};
            l.set_time(time,"test shop switch").unwrap(); l.refresh_bar().unwrap(); l.refresh_npcs("test shop switch").unwrap();
        }
        assert_eq!(l.rng,early.after_day.unwrap());
        assert_eq!(l.state.encounter_counter,counter,"Both switches preserve the movement counter");
        l.state.encounter_counter=early.remaining_walk.as_ref().unwrap().counter_before_reset.unwrap();
        let o=osvald::with_world_state(l.rng,l.state).unwrap();
        assert_eq!(o.after_recruitment,early.osvald.as_ref().unwrap().after_recruitment);
        let after=o.after_recruitment.unwrap(); let w=o.world_after.unwrap();
        assert_eq!(w.party,[7,6]); assert_eq!(w.time,2);
        let mut time_loader=world::Loader::new(world::catalog(),w.clone(),after).unwrap();
        for _ in 0..early.post_osvald_switches {
            let time=if time_loader.state.time==2 {4} else {2};
            time_loader.set_time(time,"test Osvald switch").unwrap(); time_loader.refresh_bar().unwrap(); time_loader.refresh_npcs("test Osvald switch").unwrap();
        }
        assert_eq!(time_loader.rng,early.after_osvald_switches.unwrap());
        assert_eq!(time_loader.state.encounter_counter,w.encounter_counter);
        assert_eq!(time_loader.state.first_encounter,w.first_encounter);
        let mut rng=time_loader.rng;
        let e=encounter::catalog().predict(&mut rng,&PredictionOptions {
            map:osvald::MAP.into(),zone:"ENT_FLD_SNW_1_2_LND".into(),period:if time_loader.state.time==2 {"DayTime"} else {"Night"}.into(),progress:1,
            first_encounter:w.first_encounter,boundary:Boundary::CurrentCounter,current_counter:Some(w.encounter_counter),
            party_levels:vec![5,1],dash:true,..Default::default()
        }).unwrap();
        assert_eq!(rng,early.before_battle_initialization.unwrap());
        assert_eq!(serde_json::to_value(e).unwrap(),serde_json::to_value(&early.encounter).unwrap());
        assert_eq!(p.early_weighted_input_cost,Some(selected.weighted_input_cost+
            crate::early_cait::TIME_SWITCH_INPUT_COST*(1+early.post_cleric_switches+early.post_osvald_switches)));
        assert!(early.post_cleric_switches>1 || early.post_osvald_switches>0,"Exercise the new time-switch control points");
        // Independently exhaust every cheaper phase allocation for this selected
        // Escape state, including ending the encounter at night.
        let switches=early.post_cleric_switches-1+early.post_osvald_switches;
        for total in 0..switches {
            for k in (0..=total).step_by(2) {
                let q=crate::early_cait::project_with_policy(p.at_village_entrance.unwrap(),
                    p.world_at_village_entrance.clone().unwrap(),5,k,total-k,true).unwrap();
                assert_ne!(crate::early_cait::outcomes(&q),[true;4]);
            }
        }
        assert!(crate::early_cait::project_with_switches(p.at_village_entrance.unwrap(),
            p.world_at_village_entrance.clone().unwrap(),5,1,0).is_err());
        let optional=until_osvald_cait(entry,&world(),5,Settings {
            snow_max_boost_repeats:Some(0),osvald_max_boost_repeats:Some(0),..settings
        },crate::early_cait::Settings {clerics_max_boost_repeats:Some(0),cait_max_boost_repeats:Some(0),require_cait:true}).unwrap();
        assert_eq!(optional.escape.as_ref().unwrap().attempt.as_ref().unwrap().after,first.escape.unwrap().attempt.unwrap().after);
        assert!(optional.early_searches.iter().any(|s|!s.target_met));
    }
    #[test]
    fn village_fork_stops_at_tag_and_ignores_all_later_search_targets() {
        let w = world();
        let mut covered = 0;
        for seed in 0..24 {
            let entry = CrtRng { state: seed, draws: 6050 };
            let settings = Settings { max_boost_repeats: Some(4),
                snow_max_boost_repeats: Some(0), osvald_max_boost_repeats: Some(0) };
            let p = until_first_village(entry, &w, 5, settings).unwrap();
            let required = until_first_village(entry, &w, 5, Settings {
                snow_max_boost_repeats: None, osvald_max_boost_repeats: None, ..settings
            }).unwrap();
            assert!(p.village_return.is_none() && p.osvald.is_none());
            assert!(p.snow_search.is_none() && p.osvald_search.is_none());
            assert_eq!(serde_json::to_value(&p.escape).unwrap(), serde_json::to_value(&required.escape).unwrap());
            let Some(v) = &p.village_visit else {
                assert_eq!(p.encounter.situation.as_deref(), Some("Surprise"));
                continue;
            };
            covered += 1;
            assert!(p.escape.as_ref().unwrap().attempt.as_ref().unwrap().success);
            assert!(p.remaining_run.as_ref().unwrap().reaches_reset);
            assert_eq!(v.world_after.map, village_return::VILLAGE);
            assert!(v.world_after.visited_maps.contains(village_return::VILLAGE));
            assert_eq!(v.world_after.party, [7]);
            let mut loader = world::Loader::new(world::catalog(),
                p.world_at_village_entrance.clone().unwrap(), p.at_village_entrance.unwrap()).unwrap();
            loader.visit_map(village_return::VILLAGE).unwrap();
            loader.dismiss_tutorial(62).unwrap();
            assert_eq!(loader.rng, v.after);
            assert_eq!(serde_json::to_value(loader.state).unwrap(), serde_json::to_value(&v.world_after).unwrap());
            assert_eq!(serde_json::to_value(v).unwrap(), serde_json::to_value(required.village_visit.unwrap()).unwrap());
            // The full route still uses precisely this arrival before its return load.
            let full = predict(entry, &w, 5, settings).unwrap();
            let old = full.village_return.unwrap();
            assert_eq!(old.at_village, v.after);
            assert_eq!(serde_json::to_value(old.world_at_village).unwrap(), serde_json::to_value(&v.world_after).unwrap());
        }
        assert!(covered > 0);
    }
    fn world() -> WorldState {
        let mut w = world::throne_after_second_state().unwrap();
        w.map = "Fld_Cty_1_2".into();
        w.time = 2;
        w.party = vec![7];
        w.first_encounter = true;
        w.active_scenarios.clear();
        w.prologues.clear();
        w.story_types.clear();
        w.encounter_counter = 515;
        w.visited_maps.insert("Fld_Cty_1_1".into());
        w.resident_levels = world::catalog().maps[&w.map]
            .streams
            .iter()
            .cloned()
            .collect();
        w
    }
    #[test]
    fn distance_and_fallbacks_replay_without_mutating_arrival() {
        let w = world();
        let original = serde_json::to_value(&w).unwrap();
        let mut covered = [false; 4];
        for seed in 0..160 {
            let rng = CrtRng {
                state: seed,
                draws: 6050,
            };
            let p = predict(
                rng,
                &w,
                5,
                Settings {
                    max_boost_repeats: Some(0),
                    snow_max_boost_repeats: Some(0),
                    osvald_max_boost_repeats: Some(0),
                },
            )
            .unwrap();
            assert_eq!(
                (
                    p.first_encounter_update,
                    p.remaining_updates,
                    p.required_counter
                ),
                (43, 127, 1525)
            );
            if p.encounter.situation.as_deref() == Some("Surprise") {
                covered[3] = true;
                assert!(
                    p.escape.is_none()
                        && p.field_return.is_none()
                        && p.at_village_entrance.is_none()
                );
                continue;
            }
            let s = p.escape.as_ref().unwrap();
            match s.method {
                EscapeMethod::WithinLimit => covered[0] = true,
                EscapeMethod::BreakThenEscape => covered[1] = true,
                EscapeMethod::UnlimitedFallback => covered[2] = true,
                _ => panic!("Unexpected method"),
            }
            let r = p.field_return.as_ref().unwrap();
            assert!(s.attempt.as_ref().unwrap().success);
            assert_eq!(r.arrival, s.attempt.as_ref().unwrap().after);
            assert!(r.world_after.encounter_counter >= 1525);
            assert!(!r.world_after.first_encounter);
            assert_eq!(r.after.draws - r.arrival.draws, 2); // Flats has no gondolas.
            assert_eq!(p.at_village_entrance, Some(r.after));
            assert_eq!(
                p.remaining_run.as_ref().unwrap().counter_before_reset,
                Some(r.world_after.encounter_counter - 1524)
            );
            assert_eq!(
                p.world_at_village_entrance
                    .as_ref()
                    .unwrap()
                    .encounter_counter,
                r.world_after.encounter_counter - 1524
            );
            if s.break_action.is_none() && let Some(inputs) = &s.inputs {
                let replay = s.menu.clone().attempt(&inputs.levels, 5).unwrap();
                assert_eq!(replay.after, s.attempt.as_ref().unwrap().after);
                // Ascending repeat search must reject all cheaper direct paths.
                for n in 0..inputs.extra_repeats {
                    let a = s
                        .menu
                        .clone()
                        .attempt(&(0..n).flat_map(|_| [1, 0]).collect::<Vec<_>>(), 5)
                        .unwrap();
                    if a.success {
                        let f = highroad::restore_field(&w, &a, "Fld_Cty_1_2", "test").unwrap();
                        assert!(f.world_after.encounter_counter < p.required_counter);
                    }
                }
            } else {
                let b=s.break_action.as_ref().unwrap();
                assert_eq!(b.at_attack_menu, s.menu.rng);
                assert_eq!(s.attempt.as_ref().unwrap().chance_percent, 100);
                let replay=ot2_battle_engine::field_escape::opening_break_menu(p.before_battle_initialization,
                    if p.encounter.situation.as_deref()==Some("Preemptive"){Situation::Preemptive}else{Situation::Normal},
                    5,&[p.encounter.enemies[0].enemy.id.as_str()],b.weapon,&b.attack_inputs).unwrap().unwrap()
                    .attempt(&b.escape_inputs).unwrap();
                assert_eq!(replay.attempt.after,s.attempt.as_ref().unwrap().after);
            }
        }
        assert_eq!(covered, [true; 4]);
        assert_eq!(serde_json::to_value(&w).unwrap(), original);
    }

    #[test]
    fn village_walk_search_preserves_escape_and_run_and_replays_both_map_loads() {
        let w = world();
        let mut changed = 0;
        for i in 0..24 {
            let mut entry = CrtRng::new(0);
            entry.advance(i * 7919);
            let baseline = predict(
                entry,
                &w,
                5,
                Settings {
                    max_boost_repeats: None,
                    snow_max_boost_repeats: Some(0),
                    osvald_max_boost_repeats: Some(0),
                },
            )
            .unwrap();
            if baseline.escape.is_none() {
                continue;
            }
            let p = predict(
                entry,
                &w,
                5,
                Settings {
                    max_boost_repeats: None,
                    snow_max_boost_repeats: None,
                    osvald_max_boost_repeats: Some(0),
                },
            )
            .unwrap();
            assert_eq!(
                p.before_battle_initialization,
                baseline.before_battle_initialization
            );
            let selected = p.escape.as_ref().unwrap();
            let v = p.village_return.as_ref().unwrap();
            assert!(selected.attempt.as_ref().unwrap().success);
            assert!(p.remaining_run.as_ref().unwrap().reaches_reset);
            assert!(v.snow_reach.walking.reaches_reset);
            assert!(v.world_after.first_encounter);
            assert!((500..=899).contains(&v.world_after.encounter_counter));
            assert!(
                v.world_at_village
                    .visited_maps
                    .contains(village_return::VILLAGE)
            );
            let mut loader = crate::world::Loader::new(
                crate::world::catalog(),
                p.world_at_village_entrance.clone().unwrap(),
                p.at_village_entrance.unwrap(),
            )
            .unwrap();
            loader.visit_map(village_return::VILLAGE).unwrap();
            loader.dismiss_tutorial(62).unwrap();
            assert_eq!(loader.rng, v.at_village);
            loader.visit_map("Fld_Cty_1_2").unwrap();
            assert_eq!(loader.rng, v.after);
            assert_eq!(
                serde_json::to_value(loader.state).unwrap(),
                serde_json::to_value(&v.world_after).unwrap()
            );
            let mut previous = v.arrival;
            for trace in &v.trace {
                assert_eq!(trace.before, previous);
                previous = trace.after;
            }
            assert_eq!(previous, v.after);
            if !baseline
                .village_return
                .as_ref()
                .unwrap()
                .snow_reach
                .walking
                .reaches_reset
            {
                changed += 1;
                let old = baseline
                    .escape
                    .as_ref()
                    .unwrap()
                    .inputs
                    .as_ref()
                    .unwrap()
                    .extra_repeats;
                let n = selected.inputs.as_ref().unwrap().extra_repeats;
                assert!(n > old);
                // Every cheaper successful direct Escape must fail one route condition.
                for repeats in 0..n {
                    let a = selected
                        .menu
                        .clone()
                        .attempt(&(0..repeats).flat_map(|_| [1, 0]).collect::<Vec<_>>(), 5)
                        .unwrap();
                    if !a.success {
                        continue;
                    }
                    let f = highroad::restore_field(&w, &a, "Fld_Cty_1_2", "test").unwrap();
                    if f.world_after.encounter_counter < p.required_counter {
                        continue;
                    }
                    let mut at_exit = f.world_after;
                    at_exit.encounter_counter -= p.remaining_updates * 12;
                    assert!(
                        !village_return::with_world_state(f.after, at_exit)
                            .unwrap()
                            .snow_reach
                            .walking
                            .reaches_reset
                    );
                }
                let enough = predict(
                    entry,
                    &w,
                    5,
                    Settings {
                        max_boost_repeats: None,
                        snow_max_boost_repeats: Some(n - old),
                        osvald_max_boost_repeats: Some(0),
                    },
                )
                .unwrap();
                assert_eq!(enough.village_return.unwrap().after, v.after);
                let short = predict(
                    entry,
                    &w,
                    5,
                    Settings {
                        max_boost_repeats: None,
                        snow_max_boost_repeats: Some(n - old - 1),
                        osvald_max_boost_repeats: Some(0),
                    },
                )
                .unwrap();
                assert!(!short.snow_search.unwrap().target_met);
                assert_eq!(
                    short.field_return.unwrap().after,
                    baseline.field_return.unwrap().after
                );
            }
        }
        assert!(
            changed > 0,
            "Must exercise an adjustment for the newly added target"
        );
    }

    #[test]
    fn required_osvald_walk_includes_snow_prerequisite_and_optional_cap_keeps_plan() {
        let w = world();
        let mut changed = 0;
        for i in 0..32 {
            let mut entry = CrtRng::new(0);
            entry.advance(i * 7919);
            let settings = Settings {
                max_boost_repeats: None,
                snow_max_boost_repeats: Some(0),
                osvald_max_boost_repeats: Some(0),
            };
            let baseline = predict(entry, &w, 5, settings).unwrap();
            let Some(old_escape) = &baseline.escape else {
                continue;
            };
            let old = old_escape.inputs.as_ref().unwrap().extra_repeats;
            let p = predict(
                entry,
                &w,
                5,
                Settings {
                    osvald_max_boost_repeats: None,
                    ..settings
                },
            )
            .unwrap();
            assert!(
                p.village_return
                    .as_ref()
                    .unwrap()
                    .snow_reach
                    .walking
                    .reaches_reset
            );
            let o = p.osvald.as_ref().unwrap();
            assert!(o.walking.reaches_reset && o.after_recruitment.is_some());
            let selected = p.escape.as_ref().unwrap();
            let n = selected.inputs.as_ref().unwrap().extra_repeats;
            if n == old {
                continue;
            }
            changed += 1;
            assert!(n > old);
            for repeats in old..n {
                let a = selected
                    .menu
                    .clone()
                    .attempt(&(0..repeats).flat_map(|_| [1, 0]).collect::<Vec<_>>(), 5)
                    .unwrap();
                if !a.success {
                    continue;
                }
                let f = highroad::restore_field(&w, &a, "Fld_Cty_1_2", "test").unwrap();
                if f.world_after.encounter_counter < p.required_counter {
                    continue;
                }
                let mut exit = f.world_after;
                exit.encounter_counter -= p.remaining_updates * 12;
                let v = village_return::with_world_state(f.after, exit).unwrap();
                assert!(
                    !osvald::after_village(&v)
                        .unwrap()
                        .is_some_and(|o| o.walking.reaches_reset)
                );
            }
            let enough = predict(
                entry,
                &w,
                5,
                Settings {
                    osvald_max_boost_repeats: Some(n - old),
                    ..settings
                },
            )
            .unwrap();
            assert_eq!(
                enough.osvald.unwrap().after_recruitment,
                o.after_recruitment
            );
            let short = predict(
                entry,
                &w,
                5,
                Settings {
                    osvald_max_boost_repeats: Some(n - old - 1),
                    ..settings
                },
            )
            .unwrap();
            assert!(!short.osvald_search.unwrap().target_met);
            assert_eq!(
                short.field_return.unwrap().after,
                baseline.field_return.unwrap().after
            );
        }
        assert!(changed > 0);
        // Break preference must meet both walks too, rather than bypass them.
        let mut breaks = 0;
        for seed in 0..64 {
            let p = predict(
                CrtRng::new(seed),
                &w,
                5,
                Settings {
                    max_boost_repeats: Some(0),
                    ..Default::default()
                },
            )
            .unwrap();
            if let Some(s) = p.escape {
                assert!(p.osvald.unwrap().after_recruitment.is_some());
                if matches!(s.method, EscapeMethod::BreakThenEscape) {
                    breaks += 1;
                }
            }
        }
        assert!(breaks > 0);
    }

    #[test]
    fn distances_use_update_ceil_and_keep_positive_remainder() {
        let mut w = world();
        for (counter, trigger, required) in [
            (504, 42, 1537),
            (505, 43, 1525),
            (516, 43, 1525),
            (517, 44, 1513),
            (899, 75, 1141),
        ] {
            w.encounter_counter = counter;
            let p = predict(CrtRng::new(7), &w, 5, Settings::default()).unwrap();
            assert_eq!(
                (p.first_encounter_update, p.required_counter),
                (trigger, required)
            );
        }
        w.time = 4;
        assert!(predict(CrtRng::new(0), &w, 5, Settings::default()).is_err());
    }
}
