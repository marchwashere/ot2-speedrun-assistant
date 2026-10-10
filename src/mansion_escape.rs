//! Mansion entry -> lookout -> first random encounter -> first Escape attempt
//! -> Pirro trigger. Distances are counter UPDATE counts, not counter points.
use crate::{
    CrtRng, Result,
    encounter::{self, Boundary, PredictionOptions},
    ensure,
    manipulation::{BoostInputs, optimal_inputs, optimal_inputs_with_cap},
    world::WorldState,
};
use ot2_battle_engine::{
    Resources,
    escape::{Attempt, FirstEscapeBattle, Situation},
    opening_crt::RngSpan,
};
use serde::Serialize;

pub const RUN_UPDATES: u32 = 161;
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Settings {
    pub max_boost_repeats: Option<u32>,
    /// Required when None; optional counter-search budget otherwise.
    pub run_max_boost_repeats: Option<u32>,
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            max_boost_repeats: Some(4),
            run_max_boost_repeats: None,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Solution {
    pub reaches_pirro: bool,
    pub break_then_escape: bool,
    pub extra_advances: u32,
    pub inputs: BoostInputs,
    pub weighted_input_cost: usize,
    pub escape_inputs: Option<BoostInputs>,
    pub at_escape_menu: Option<CrtRng>,
    pub attempt: Attempt,
    pub after_escape: CrtRng,
    pub after_field_return: CrtRng,
    pub counter: u32,
    pub counter_at_trigger: u32,
    pub trace: Vec<RngSpan>,
    pub world_at_trigger: WorldState,
}
#[derive(Debug, Serialize)]
pub struct Prediction {
    pub counter_base_repeats: u32,
    /// Fixed route distance, including the lookout approach.
    pub run_updates: u32,
    pub settings: Settings,
    pub after_lookout: CrtRng,
    pub first_encounter: encounter::Prediction,
    pub before_battle_initialization: CrtRng,
    pub at_command_menu: CrtRng,
    pub round: u32,
    pub available_bp: u8,
    pub enemy_actions_before_menu: usize,
    pub initial_trace: Vec<RngSpan>,
    pub entry_counter: u32,
    pub decrement: u32,
    pub first_encounter_update: u32,
    pub remaining_updates: u32,
    pub required_counter: u32,
    pub baseline_success: bool,
    pub baseline_counter: Option<u32>,
    pub solution: Option<Solution>,
    pub status: &'static str,
    pub verification: &'static str,
}
fn at(entry: CrtRng, local: CrtRng) -> CrtRng {
    CrtRng {
        state: local.state,
        draws: entry.draws + local.draws,
    }
}
fn trace_at(entry: CrtRng, rows: &[RngSpan]) -> Vec<RngSpan> {
    rows.iter()
        .map(|r| RngSpan {
            reason: r.reason,
            actor: r.actor,
            before: at(entry, r.before),
            after: at(entry, r.after),
        })
        .collect()
}

/// Continue the supported serial route, including its fixed resource recovery.
pub fn from_second(
    second: &crate::second::Prediction,
    settings: Settings,
) -> Result<Option<Prediction>> {
    let Some(m) = &second.mansion else {
        return Ok(None);
    };
    // Opening 4..6 EXP + second 7..9 EXP => level 2 (level 3 needs 16).
    // Both 0700 and 1100 run StartFullRecovery. No HP/SP spending follows them
    // before this encounter; retain latent gauge, which 2310 does not restore.
    ensure(
        crate::world::catalog().events["MS_TOU_10_1100"]
            .commands
            .iter()
            .any(|c| c.cmd == 2310),
        "Missing parlor full-recovery event",
    )?;
    let mut resources = ot2_battle_engine::BattleConfig::throne_second(0)?.players[0]
        .resources
        .clone()
        .unwrap();
    resources.latent = second.state.actors[0].resources.latent;
    predict(
        m.lookout_ambush.after,
        &m.lookout_ambush.world_after,
        resources,
        settings,
    )
    .map(Some)
}

/// Field-only boundary shared by the second-fight search and Escape search.
pub fn first_encounter(
    after_lookout: CrtRng,
    world: &WorldState,
) -> Result<(CrtRng, encounter::Prediction)> {
    ensure(
        world.map == "Dng_Cty_1_1"
            && world.time == 4
            && world.flag(8016)
            && !world.flag(8017)
            && !world.flag(25305)
            && !world.flag(652)
            && world.party == [7]
            && world.first_encounter,
        "Mansion Escape requires the solo Throné night route after lookout Ambush, before the first random encounter",
    )?;
    let options = PredictionOptions {
        map: world.map.clone(),
        zone: "ENT_DNG_CTY_1_1_LND".into(),
        period: "Night".into(),
        progress: 0,
        first_encounter: true,
        boundary: Boundary::CurrentCounter,
        party_levels: vec![2],
        dash: true,
        current_counter: Some(world.encounter_counter),
        ..Default::default()
    };
    let mut entry = after_lookout;
    let first_encounter = encounter::catalog().predict(&mut entry, &options)?;
    ensure(
        first_encounter.group.as_deref() == Some("ENG_DNG_CTY_1_1_LND_DAY_110"),
        "Unsupported first mansion encounter",
    )?;
    Ok((entry, first_encounter))
}

pub fn predict(
    after_lookout: CrtRng,
    world: &WorldState,
    resources: Resources,
    settings: Settings,
) -> Result<Prediction> {
    ensure(
        settings.max_boost_repeats.is_none_or(|n| n <= 256) && settings.run_max_boost_repeats.is_none_or(|n| n <= 256),
        "Mansion Escape cap must be 0..256 or unlimited",
    )?;
    let (entry, first_encounter) = first_encounter(after_lookout, world)?;
    let decrement = first_encounter.decrement.unwrap();
    let trigger = first_encounter.counter_updates.unwrap();
    ensure(
        RUN_UPDATES >= trigger,
        "Measured path ends before the first encounter",
    )?;
    let remaining = RUN_UPDATES - trigger;
    let required = remaining * decrement + 1;
    let situation = match first_encounter.situation.as_deref() {
        Some("Normal") => Situation::Normal,
        Some("Preemptive") => Situation::Preemptive,
        Some("Surprise") => Situation::Surprise,
        _ => return Err(crate::SimError("Unknown battle situation".into())),
    };
    let battle = FirstEscapeBattle::mansion(entry.state, situation, resources, true)?;
    let menu = at(entry, battle.state().rng);
    let bp = battle.state().actors[0].resources.bp;
    let counter = &encounter::catalog().maps["Dng_Cty_1_1"].counter;
    let baseline = battle.clone().attempt(&[], 3)?;
    let baseline_counter = baseline.success.then(|| {
        let mut rng = baseline.after;
        counter.reset(&mut rng, false)
    });
    let mut prediction = Prediction {
        counter_base_repeats: 0,
        run_updates: RUN_UPDATES,
        settings,
        after_lookout,
        first_encounter,
        before_battle_initialization: entry,
        at_command_menu: menu,
        round: battle.state().round,
        available_bp: bp,
        enemy_actions_before_menu: battle.enemy_actions_before_menu.len(),
        entry_counter: world.encounter_counter,
        initial_trace: trace_at(
            entry,
            battle
                .state()
                .opening_crt
                .as_ref()
                .unwrap()
                .trace
                .as_deref()
                .unwrap(),
        ),
        decrement,
        first_encounter_update: trigger,
        remaining_updates: remaining,
        required_counter: required,
        baseline_success: baseline.success,
        baseline_counter,
        solution: None,
        status: "No plan within repeat limit",
        verification: "Source-derived, awaiting live first-encounter/Escape trace. Original level-two equipment; Compare direct first Escape with actual 1 BP dagger break and any legal next-menu boosts, charging one per boost input and ten for breaking. No second encounter is mandatory for both strategies. 161 running updates measured from mansion entry, including the lookout approach. Ends at the Pirro trigger BEFORE its cutscene or map load. No extra field actions or CRT calls en route.",
    };
    // A strict positive counter is required at the trigger. Reject impossible
    // distance targets before attempting an unlimited CRT search.
    let maximum = counter.min + counter.range_a - 1 + counter.range_b - 1;
    if required > maximum && settings.run_max_boost_repeats.is_none() {
        prediction.status = "Remaining run exceeds maximum counter";
        return Ok(prediction);
    }
    let max_bp = bp.min(3);
    // First find the minimum input cost for the mandatory safe counter. This
    // evaluates a hypothetical reset even if that Escape roll would fail; it
    // never exposes that trial as a playable solution.
    let mut base_cost = usize::MAX;
    let counter_limit=settings.run_max_boost_repeats.map_or(u32::MAX,|n| n*u32::from(max_bp)*u32::from(max_bp+1)/2);
    for extra in 0..=counter_limit {
        if 2*u64::from(extra).div_ceil(u64::from(max_bp)) >= base_cost as u64 { break; }
        let inputs=match settings.run_max_boost_repeats {
            Some(cap)=>{let Some(p)=optimal_inputs_with_cap(extra,bp,0,cap)? else {continue}; p},
            None=>optimal_inputs(extra,bp,0)?,
        };
        let attempt=battle.clone().attempt(&inputs.levels,3)?;
        let mut rng=attempt.after;
        if counter.reset(&mut rng,false) >= required && inputs.input_count < base_cost {
            base_cost=inputs.input_count;
            prediction.counter_base_repeats=inputs.extra_repeats;
        }
    }
    let require_safe_counter=base_cost!=usize::MAX;
    // Numeric budget is additional to mandatory counter manipulation. If it
    // cannot also win the first Escape check, switch to a real dagger break.
    let mut best_inputs=usize::MAX;
    for break_then_escape in [false,true] {
        if break_then_escape && settings.max_boost_repeats.is_none() {continue;}
        let cap=None;
        let break_cost=if break_then_escape {(crate::escape_search::BREAK_INPUT_COST+crate::escape_search::ROUND_INPUT_COST) as usize} else {0};
        let final_bp=u8::from(break_then_escape);
        let max_draws=cap.map_or(u32::MAX,|n| n*u32::from(max_bp)*u32::from(max_bp+1)/2);
        for extra in 0..=max_draws {
            if 2*u64::from(extra).div_ceil(u64::from(max_bp))+u64::from(final_bp)+break_cost as u64 >= best_inputs as u64 { break; }
            let inputs=match cap {
                Some(c)=>optimal_inputs_with_cap(extra,bp,final_bp,c)?,
                None=>Some(optimal_inputs(extra,bp,final_bp)?),
            };
            let Some(inputs)=inputs else {continue};
            if inputs.input_count+break_cost >= best_inputs {continue;}
            let break_menu=if break_then_escape {battle.clone().break_menu(&inputs.levels)?} else {None};
            if break_then_escape && break_menu.is_none() {continue;}
            let escape_bp=break_menu.as_ref().map_or(0,|m|m.state().actors[0].resources.bp.min(3));
            let max_escape_draws=if escape_bp==0 {0} else {
                (best_inputs.saturating_sub(break_cost+inputs.input_count+1)/2).saturating_mul(escape_bp as usize) as u32
            };
            for escape_draws in 0..=max_escape_draws {
            let escape_inputs=optimal_inputs(escape_draws,escape_bp,0)?;
            let candidate_cost=inputs.input_count+break_cost+escape_inputs.input_count;
            if candidate_cost>=best_inputs {continue;}
            let at_escape_menu=break_menu.as_ref().map(|m|at(entry,m.state().rng));
            let attempt=if let Some(m)=&break_menu {m.clone().attempt_broken(&escape_inputs.levels)?}
                else {battle.clone().attempt(&inputs.levels,3)?};
            if !attempt.success {continue;}
            let after_escape=at(entry,attempt.after);
            let mut after_field_return=after_escape;
            let value=counter.reset(&mut after_field_return,false);
            if require_safe_counter && value < required {continue;}
            let mut trace=trace_at(entry,attempt.state.opening_crt.as_ref().unwrap().trace.as_deref().unwrap());
            trace.push(RngSpan { reason:"escape_field_counter_reset_b_then_a",actor:None,
                before:after_escape,after:after_field_return });
            let mut world_at_trigger=world.clone();
            world_at_trigger.first_encounter=false;
            world_at_trigger.encounter_counter=value.saturating_sub(remaining*decrement);
            best_inputs=candidate_cost;
            prediction.solution=Some(Solution { reaches_pirro:value>=required,break_then_escape,extra_advances:extra+escape_draws,inputs:inputs.clone(),weighted_input_cost:best_inputs,
                escape_inputs:break_then_escape.then_some(escape_inputs),at_escape_menu,attempt,
                after_escape,after_field_return,counter:value,counter_at_trigger:world_at_trigger.encounter_counter,
                trace,world_at_trigger });
            }
        }
    }
    if let Some(s)=&prediction.solution {
        prediction.status=if !s.reaches_pirro {"Escape found; optional no-second-encounter target not met"} else if s.break_then_escape {"Break then Escape · run to Pirro"} else {"Escape first try · run to Pirro"};
    }
    Ok(prediction)
}
