#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

//! Local inspection shell. The battle and all RNG logic stay in the library.
use ot2_battle_engine::{
    Battle, BattleConfig, CrtRng, ItemCatalog, Outcome, SimError, routine::ThroneOpeningRoutine,
};
use ot2_speedrun_assistant::manipulation::{BoostPlan, EventRepeatCaps};
use serde_json::{Value, json};
use std::{
    env,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::Command,
    time::Duration,
};

const HTML: &str = include_str!("../../viewer/index.html");
const CSS: &str = include_str!("../../viewer/viewer.css");
const JS: &str = include_str!("../../viewer/viewer.js");

fn parse_seed(text: &str) -> Result<u32, SimError> {
    let text = text.trim();
    let (digits, radix) =
        if let Some(hex) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
            (hex, 16)
        } else {
            (text, 10)
        };
    if digits.is_empty() || !digits.chars().all(|c| c.is_ascii() && c.is_digit(radix)) {
        return Err(SimError(
            "Enter a decimal seed or 0x-prefixed hexadecimal seed.".into(),
        ));
    }
    u32::from_str_radix(digits, radix)
        .map_err(|_| SimError("Seed must be between 0 and 4294967295 (0xFFFFFFFF).".into()))
}

#[cfg(test)]
fn simulate(seed: u32) -> Result<Value, SimError> {
    simulate_with_plan(seed, BoostPlan::default())
}

#[cfg(test)]
fn simulate_with_plan(seed: u32, plan: BoostPlan) -> Result<Value, SimError> {
    simulate_route(
        seed,
        plan,
        Default::default(),
        Some(4),
        Some(BoostPlan::default()),
        Default::default(),
        None,
        Default::default(),
        None,
        Default::default(),
    )
}
fn simulate_route(
    seed: u32,
    plan: BoostPlan,
    mansion_settings: ot2_speedrun_assistant::mansion_escape::Settings,
    approach_cap: Option<u32>,
    second_plan: Option<BoostPlan>,
    pirro_settings: ot2_speedrun_assistant::pirro::Settings,
    pirro_plan: Option<Vec<u8>>,
    highroad_settings: ot2_speedrun_assistant::highroad::Settings,
    pirro_r8_plan: Option<Vec<u8>>,
    flats_escape_settings: ot2_speedrun_assistant::flats_escape::Settings,
) -> Result<Value, SimError> {
    plan.validate()?;
    let config = BattleConfig::throne_opening(seed)?;
    let actors: Vec<_> = config.players.iter().map(|p| {
        json!({"name":p.name,"side":"player","max_hp":p.stats.max_hp,"max_sp":p.stats.max_sp})
    }).chain(config.enemies.iter().enumerate().map(|(i,e)| {
        json!({"name":format!("Enemy {} ({}) · {}",i+1,if i == 0 { "right" } else { "left" },e.name),"side":"enemy","max_hp":e.stats.max_hp,"max_sp":e.stats.max_sp})
    })).collect();
    let mut battle = Battle::new_opening(config, ItemCatalog::builtin()?, true)?;
    let initial_state = battle.state().clone();
    let mut routine = ThroneOpeningRoutine::default();
    let mut steps = Vec::new();
    let mut reports = Vec::new();
    let mut assessment = ot2_speedrun_assistant::route::BattleAssessment::default();
    let mut player_menu = 0;
    let mut manipulation = None;
    while battle.state().outcome == Outcome::Ongoing {
        if steps.len() == 16 {
            return Err(SimError("Routine exceeded its action budget.".into()));
        }
        let round = battle.state().round;
        let before = battle.state().rng;
        let actor = battle.current_actor().expect("ongoing actor");
        if battle.side(actor) == Some(ot2_battle_engine::Side::Player) {
            player_menu += 1;
        }
        let (mut report, inputs) = if battle.side(actor) == Some(ot2_battle_engine::Side::Player) {
            plan.step(player_menu, &mut battle, &mut routine)?
        } else {
            (routine.step(&mut battle)?, None)
        };
        if let Some(inputs) = inputs {
            let mut after_inputs = before;
            after_inputs.advance(
                u64::from(plan.advances)
                    + u64::from(inputs.normal_bp) * u64::from(inputs.normal_bp + 1) / 2,
            );
            manipulation = Some(
                json!({"plan":plan,"inputs":inputs,"actor":actor,"round":round,"rng_before":before,"rng_after_inputs":after_inputs}),
            );
        }
        // Keep command-menu checkpoints before the extra boost inputs.
        report.rng_before = before;
        assessment.observe(
            round,
            battle.side(report.actor).expect("report actor"),
            &report,
        );
        steps.push(json!({"round":round,"report":report,"actors_after":battle.state().actors}));
        reports.push(report);
    }
    if plan.advances > 0 && manipulation.is_none() {
        return Err(SimError("The requested boost menu was not reached".into()));
    }
    let field = if battle.state().outcome == Outcome::Victory {
        let options = ot2_speedrun_assistant::route::throne_post_battle_options(battle.state())?;
        let reach = ot2_speedrun_assistant::route::throne_reset_reach(
            options.current_counter.expect("exit counter"),
        )?;
        let checkpoint = reach
            .walking
            .reaches_reset
            .then(|| ot2_speedrun_assistant::route::throne_checkpoint(battle.state().rng));
        let ambush = checkpoint
            .as_ref()
            .map(ot2_speedrun_assistant::route::throne_ambush);
        let second_battle = ambush
            .as_ref()
            .map(|a| {
                ot2_speedrun_assistant::second_manipulation::predict(
                    a.after_rejoin,
                    &battle,
                    &reports,
                    approach_cap,
                    second_plan,
                )
            })
            .transpose()?;
        let mut pirro_battle = None;
        let mut pirro_aftermath = None;
        let mut highroad = None;
        let mut flats = None;
        let mut harbor_return = None;
        let mut flats_escape = None;
        let mut cape_cold = None;
        let mut cape_cold_mug = None;
        let mansion_escape = second_battle.as_ref().map(|second| {
            match ot2_speedrun_assistant::mansion_escape::from_second(second, mansion_settings) {
                Ok(p) => {
                    if let Some(escape) = &p {
                        pirro_battle = Some(
                            match ot2_speedrun_assistant::pirro::from_route(
                                second,
                                escape,
                                pirro_settings,
                                pirro_plan,
                            ) {
                                Ok(mut pirro) => {
                                    if let Some(boss) = &mut pirro {
                                        match ot2_speedrun_assistant::highroad::from_battle(boss, second, highroad_settings, pirro_r8_plan) {
                                            Ok(Some((ending, prediction))) => {
                                                flats = prediction.field_return.as_ref().map(|field| {
                                                    match ot2_speedrun_assistant::flats::after_highroad(field) {
                                                        Ok(p) => {
                                                            harbor_return = match ot2_speedrun_assistant::harbor::after_flats(&p) {
                                                                Ok(p) => p.map(|p| {
                                                                    flats_escape = Some(match ot2_speedrun_assistant::flats_escape::after_harbor(&p, prediction.level, flats_escape_settings) {
                                                                        Ok(e) => {
                                                                            cape_cold = e.osvald.as_ref().and_then(|o| match ot2_speedrun_assistant::cape_cold::after_osvald(o) {
                                                                                Ok(p) => p.map(|p| {
                                                                                    cape_cold_mug = match ot2_speedrun_assistant::cape_cold_mug::after_cape_cold(&p) {
                                                                                        Ok(m) => m.map(|m| json!(m)),
                                                                                        Err(e) => Some(json!({"error": e.to_string()})),
                                                                                    };
                                                                                    json!(p)
                                                                                }),
                                                                                Err(e) => Some(json!({"error": e.to_string()})),
                                                                            });
                                                                            json!(e)
                                                                        },
                                                                        Err(e) => json!({"error": e.to_string()}),
                                                                    });
                                                                    json!(p)
                                                                }),
                                                                Err(e) => Some(json!({"error": e.to_string()})),
                                                            };
                                                            json!(p)
                                                        },
                                                        Err(e) => json!({"error": e.to_string()}),
                                                    }
                                                });
                                                pirro_aftermath = Some(json!(ending));
                                                highroad = Some(json!(prediction));
                                            }
                                            Ok(None) => {}
                                            Err(e) => {
                                                highroad = Some(json!({"error": e.to_string()}));
                                                pirro_aftermath = Some(match ot2_speedrun_assistant::pirro_aftermath::after_battle(boss) {
                                                    Ok(ending) => json!(ending), Err(e) => json!({"error": e.to_string()}) });
                                            }
                                        }
                                    }
                                    json!(pirro)
                                },
                                Err(e) => json!({"error":e.to_string()}),
                            },
                        );
                    }
                    json!(p)
                }
                Err(e) => json!({"error":e.to_string()}),
            }
        });
        Some(json!({
            "profile":ot2_speedrun_assistant::route::throne_field_profile(),
            "reset_reach":reach,
            "checkpoint":checkpoint,
            "ambush":ambush,
            "second_battle":second_battle,
            "mansion_escape":mansion_escape,
            "pirro_battle":pirro_battle,
            "pirro_aftermath":pirro_aftermath,
            "highroad":highroad,
            "flats":flats,
            "harbor_return":harbor_return,
            "flats_escape":flats_escape,
            "cape_cold":cape_cold,
            "cape_cold_mug":cape_cold_mug,
            "options":options,
            "starting_state":battle.state().rng.state
        }))
    } else {
        None
    };
    Ok(json!({
        "initial_seed":seed, "actors":actors, "initial_state":initial_state,
        "steps":steps, "routine_id":ThroneOpeningRoutine::ID, "routine":routine, "state":battle.state(),
        "post_battle_field":field, "battle_assessment":assessment, "manipulation":manipulation
    }))
}

#[derive(Clone)]
struct CrtSetup {
    crackridge_approach_max_boost_repeats: Option<u32>,
    mug_clerics_max_boost_repeats: Option<u32>,
    seed: u32,
    advances: u64,
    recreate_shared_level: bool,
    event_caps: EventRepeatCaps,
    boost_plan: BoostPlan,
    second_plan: Option<BoostPlan>,
    approach_cap: Option<u32>,
    mansion_settings: ot2_speedrun_assistant::mansion_escape::Settings,
    pirro_settings: ot2_speedrun_assistant::pirro::Settings,
    pirro_plan: Option<Vec<u8>>,
    highroad_settings: ot2_speedrun_assistant::highroad::Settings,
    pirro_r8_plan: Option<Vec<u8>>,
    flats_escape_settings: ot2_speedrun_assistant::flats_escape::Settings,
}
fn parse_repeat_cap(value: &str, maximum: u32) -> Result<u32, SimError> {
    let cap = value.parse::<u32>().map_err(|_| {
        SimError(format!(
            "Boost repeat cap must be a whole number from 0 to {maximum}"
        ))
    })?;
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) || cap > maximum {
        return Err(SimError(format!(
            "Boost repeat cap must be a whole number from 0 to {maximum}"
        )));
    }
    Ok(cap)
}
fn parse_event_repeat_cap(value: &str) -> Result<Option<u32>, SimError> {
    if value == "required" || value == "unlimited" {
        Ok(None)
    } else {
        parse_repeat_cap(value, 256).map(Some)
    }
}
fn parse_strategy_repeat_cap(value: &str) -> Result<Option<u32>, SimError> {
    if value == "boost_only" {
        Ok(None)
    } else {
        parse_event_repeat_cap(value)
    }
}
impl CrtSetup {
    fn parse(query: &str) -> Result<Self, SimError> {
        let mut seed = None;
        let mut advances = None;
        let mut recreate_shared_level = None;
        let mut boost_advances = None;
        let mut boost_menu = None;
        let mut encounter_max_boost_repeats = None;
        let mut second_fight_max_boost_repeats = None;
        let mut boost_repeat_cap = None;
        let mut mansion_approach_max_boost_repeats = None;
        let mut second_boost_menu = None;
        let mut second_boost_advances = None;
        let mut second_boost_repeat_cap = None;
        let mut mansion_escape_max_boost_repeats = None;
        let mut mansion_run_max_boost_repeats = None;
        let mut pirro_victory_max_boost_repeats = None;
        let mut pirro_no_grape_max_boost_repeats = None;
        let mut pirro_boost_inputs = None;
        let mut pirro_r8_boost_inputs = None;
        let mut highroad_approach_max_boost_repeats = None;
        let mut highroad_escape_max_boost_repeats = None;
        let mut highroad_flats_max_boost_repeats = None;
        let mut highroad_flats_approach_max_boost_repeats = None;
        let mut flats_escape_max_boost_repeats = None;
        let mut flats_snow_max_boost_repeats = None;
        let mut flats_osvald_max_boost_repeats = None;
        let mut mug_clerics_max_boost_repeats = None;
        let mut crackridge_approach_max_boost_repeats = None;
        for part in query.split('&') {
            let (key, value) = part
                .split_once('=')
                .ok_or_else(|| SimError("Invalid CRT setup.".into()))?;
            match key {
                "seed" if seed.is_none() => seed = Some(parse_seed(value)?),
                "advances" if advances.is_none() => {
                    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(SimError(
                            "Advances must be a nonnegative whole number.".into(),
                        ));
                    }
                    advances = Some(value.parse::<u64>().map_err(|_| {
                        SimError("Advance count exceeds 18446744073709551615.".into())
                    })?);
                }
                "recreate_shared_level" if recreate_shared_level.is_none() => {
                    recreate_shared_level = Some(match value {
                        "true" => true,
                        "false" => false,
                        _ => {
                            return Err(SimError(
                                "Recreate shared level must be true or false.".into(),
                            ));
                        }
                    });
                }
                "boost_advances" if boost_advances.is_none() => {
                    boost_advances = Some(
                        value
                            .parse::<u32>()
                            .map_err(|_| SimError("Invalid boost cycle count".into()))?,
                    );
                }
                "boost_menu" if boost_menu.is_none() => {
                    boost_menu = Some(
                        value
                            .parse::<u32>()
                            .map_err(|_| SimError("Invalid boost menu".into()))?,
                    );
                }
                "encounter_max_boost_repeats" if encounter_max_boost_repeats.is_none() => {
                    encounter_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                // The legacy parameter always controlled only fight two.
                "second_fight_max_boost_repeats" | "max_boost_repeats"
                    if second_fight_max_boost_repeats.is_none() =>
                {
                    second_fight_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "boost_repeat_cap" if boost_repeat_cap.is_none() => {
                    boost_repeat_cap = Some(parse_repeat_cap(value, u32::MAX)?);
                }
                "mansion_approach_max_boost_repeats"
                    if mansion_approach_max_boost_repeats.is_none() =>
                {
                    mansion_approach_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "second_boost_menu" if second_boost_menu.is_none() => {
                    second_boost_menu = Some(parse_repeat_cap(value, 6)?);
                }
                "second_boost_advances" if second_boost_advances.is_none() => {
                    second_boost_advances = Some(parse_repeat_cap(value, u32::MAX)?);
                }
                "second_boost_repeat_cap" if second_boost_repeat_cap.is_none() => {
                    second_boost_repeat_cap = Some(parse_repeat_cap(value, u32::MAX)?);
                }
                "mansion_run_max_boost_repeats" if mansion_run_max_boost_repeats.is_none() => {
                    mansion_run_max_boost_repeats=Some(parse_event_repeat_cap(value)?);
                }
                "mansion_escape_max_boost_repeats"
                    if mansion_escape_max_boost_repeats.is_none() =>
                {
                    mansion_escape_max_boost_repeats = Some(if value == "boost_only" { None } else { parse_event_repeat_cap(value)? });
                }
                "pirro_victory_max_boost_repeats" if pirro_victory_max_boost_repeats.is_none() => {
                    pirro_victory_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                // Legacy joint limit migrates to the optional no-grape target.
                "pirro_no_grape_max_boost_repeats" | "pirro_max_boost_repeats"
                    if pirro_no_grape_max_boost_repeats.is_none() =>
                {
                    pirro_no_grape_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "highroad_approach_max_boost_repeats"
                    if highroad_approach_max_boost_repeats.is_none() =>
                {
                    highroad_approach_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "highroad_escape_max_boost_repeats"
                    if highroad_escape_max_boost_repeats.is_none() =>
                {
                    highroad_escape_max_boost_repeats = Some(parse_strategy_repeat_cap(value)?);
                }
                "highroad_flats_max_boost_repeats"
                    if highroad_flats_max_boost_repeats.is_none() =>
                {
                    highroad_flats_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "highroad_flats_approach_max_boost_repeats"
                    if highroad_flats_approach_max_boost_repeats.is_none() =>
                {
                    highroad_flats_approach_max_boost_repeats =
                        Some(parse_event_repeat_cap(value)?);
                }
                "flats_escape_max_boost_repeats" if flats_escape_max_boost_repeats.is_none() => {
                    flats_escape_max_boost_repeats = Some(parse_strategy_repeat_cap(value)?);
                }
                "flats_snow_max_boost_repeats" if flats_snow_max_boost_repeats.is_none() => {
                    flats_snow_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "flats_osvald_max_boost_repeats" if flats_osvald_max_boost_repeats.is_none() => {
                    flats_osvald_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "mug_clerics_max_boost_repeats" if mug_clerics_max_boost_repeats.is_none() => {
                    mug_clerics_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "crackridge_approach_max_boost_repeats" if crackridge_approach_max_boost_repeats.is_none() => {
                    crackridge_approach_max_boost_repeats = Some(parse_event_repeat_cap(value)?);
                }
                "pirro_boost_inputs" | "pirro_r8_boost_inputs" => {
                    let destination = if key == "pirro_boost_inputs" {
                        &mut pirro_boost_inputs
                    } else {
                        &mut pirro_r8_boost_inputs
                    };
                    if destination.is_some() {
                        return Err(SimError("Repeated Pirro boost path".into()));
                    }
                    let inputs = value
                        .split(',')
                        .map(|n| parse_repeat_cap(n, 3).map(|n| n as u8))
                        .collect::<Result<Vec<_>, _>>()?;
                    let mut previous = 0u8;
                    for &bp in &inputs {
                        if bp.abs_diff(previous) != 1 {
                            return Err(SimError("Invalid Pirro boost path".into()));
                        }
                        previous = bp;
                    }
                    if previous != 3 {
                        return Err(SimError("Pirro boost path must end at 3 BP".into()));
                    }
                    *destination = Some(inputs);
                }
                _ => return Err(SimError("Unknown or repeated CRT setup parameter.".into())),
            }
        }
        let setup = Self {
            seed: seed.ok_or_else(|| SimError("Initial seed is required.".into()))?,
            advances: advances.unwrap_or(0),
            recreate_shared_level: recreate_shared_level.unwrap_or(false),
            event_caps: EventRepeatCaps {
                encounter: encounter_max_boost_repeats
                    .unwrap_or(EventRepeatCaps::default().encounter),
                second_fight: second_fight_max_boost_repeats
                    .unwrap_or(EventRepeatCaps::default().second_fight),
            },
            boost_plan: BoostPlan {
                advances: boost_advances.unwrap_or(0),
                player_menu: boost_menu.unwrap_or(0),
                repeat_cap: boost_repeat_cap,
            },
            approach_cap: mansion_approach_max_boost_repeats.unwrap_or(Some(4)),
            second_plan: if second_boost_menu.is_some()
                || second_boost_advances.is_some()
                || second_boost_repeat_cap.is_some()
            {
                Some(BoostPlan {
                    player_menu: second_boost_menu.unwrap_or(0),
                    advances: second_boost_advances.unwrap_or(0),
                    repeat_cap: second_boost_repeat_cap,
                })
            } else {
                None
            },
            mansion_settings: ot2_speedrun_assistant::mansion_escape::Settings {
                run_max_boost_repeats:mansion_run_max_boost_repeats.unwrap_or(None),
                max_boost_repeats: mansion_escape_max_boost_repeats.unwrap_or(Some(4)),
            },
            pirro_settings: ot2_speedrun_assistant::pirro::Settings {
                victory_max_boost_repeats: pirro_victory_max_boost_repeats.unwrap_or(Some(4)),
                no_grape_max_boost_repeats: pirro_no_grape_max_boost_repeats.unwrap_or(Some(4)),
            },
            pirro_plan: pirro_boost_inputs,
            highroad_settings: ot2_speedrun_assistant::highroad::Settings {
                approach_max_boost_repeats: highroad_approach_max_boost_repeats.unwrap_or(Some(4)),
                escape_max_boost_repeats: highroad_escape_max_boost_repeats.unwrap_or(Some(4)),
                flats_max_boost_repeats: highroad_flats_max_boost_repeats.unwrap_or(Some(4)),
                flats_approach_max_boost_repeats: highroad_flats_approach_max_boost_repeats
                    .unwrap_or(Some(4)),
            },
            pirro_r8_plan: pirro_r8_boost_inputs,
            mug_clerics_max_boost_repeats: mug_clerics_max_boost_repeats.unwrap_or(None),
            crackridge_approach_max_boost_repeats: crackridge_approach_max_boost_repeats.unwrap_or(None),
            flats_escape_settings: ot2_speedrun_assistant::flats_escape::Settings {
                max_boost_repeats: flats_escape_max_boost_repeats.unwrap_or(Some(4)),
                snow_max_boost_repeats: flats_snow_max_boost_repeats.unwrap_or(None),
                osvald_max_boost_repeats: flats_osvald_max_boost_repeats.unwrap_or(None),
            },
        };
        setup.boost_plan.validate()?;
        if let Some(plan) = setup.second_plan {
            plan.validate()?;
        }
        setup
            .advances
            .checked_add(ot2_speedrun_assistant::opening::transition_draws(
                setup.recreate_shared_level,
            ))
            .ok_or_else(|| {
                SimError(
                    "Title advances plus opening transition exceed 18446744073709551615.".into(),
                )
            })?;
        Ok(setup)
    }
    fn battle_entry_advances(&self) -> u64 {
        // The input is the title frame. The assistant, not the battle core,
        // owns the skipped new-game transition and conditional shared level.
        self.advances
            + ot2_speedrun_assistant::opening::transition_draws(self.recreate_shared_level)
    }
    fn battle_seed(&self) -> u32 {
        let mut rng = CrtRng::new(self.seed);
        rng.advance(self.battle_entry_advances());
        rng.state
    }
    fn json(&self) -> Value {
        let transition = ot2_speedrun_assistant::opening::prepare(
            self.seed,
            self.advances,
            self.recreate_shared_level,
        )
        .expect("validated title setup");
        // Decimal strings keep u64 counts exact in browser JavaScript and exports.
        json!({
            "initial_seed":self.seed,"advances":self.advances.to_string(),
            "search_policies":self.outcome_policies().into_iter().map(|(id,_,cap)| (id,json!({"mode":if cap.is_none(){"required"}else{"optional"},"extra_repeat_budget":cap}))).collect::<serde_json::Map<_,_>>(),
            "input_boundary":"title_screen","route_profile":transition.profile,"cutscenes":"skip_both",
            "title_state":transition.title_state,"transition_draws":transition.draws,
            "transition_checkpoints":transition.checkpoints,
            "recreate_shared_level":self.recreate_shared_level,
            "encounter_max_boost_repeats":self.event_caps.encounter,
            "second_fight_max_boost_repeats":self.event_caps.second_fight,
            "boost_repeat_cap":self.boost_plan.repeat_cap,
            "mansion_approach_max_boost_repeats":self.approach_cap,
            "second_boost_menu":self.second_plan.unwrap_or_default().player_menu,
            "second_boost_advances":self.second_plan.unwrap_or_default().advances,
            "second_boost_repeat_cap":self.second_plan.unwrap_or_default().repeat_cap,
            "mansion_escape_max_boost_repeats":self.mansion_settings.max_boost_repeats,
            "mansion_run_max_boost_repeats":self.mansion_settings.run_max_boost_repeats,
            "pirro_victory_max_boost_repeats":self.pirro_settings.victory_max_boost_repeats,
            "pirro_no_grape_max_boost_repeats":self.pirro_settings.no_grape_max_boost_repeats,
            "pirro_boost_inputs":self.pirro_plan,
            "pirro_r8_boost_inputs":self.pirro_r8_plan,
            "highroad_approach_max_boost_repeats":self.highroad_settings.approach_max_boost_repeats,
            "highroad_escape_max_boost_repeats":self.highroad_settings.escape_max_boost_repeats,
            "highroad_flats_max_boost_repeats":self.highroad_settings.flats_max_boost_repeats,
            "highroad_flats_approach_max_boost_repeats":self.highroad_settings.flats_approach_max_boost_repeats,
            "flats_escape_max_boost_repeats":self.flats_escape_settings.max_boost_repeats,
            "flats_snow_max_boost_repeats":self.flats_escape_settings.snow_max_boost_repeats,
            "flats_osvald_max_boost_repeats":self.flats_escape_settings.osvald_max_boost_repeats,
            "mug_clerics_max_boost_repeats":self.mug_clerics_max_boost_repeats,
            "crackridge_approach_max_boost_repeats":self.crackridge_approach_max_boost_repeats,
            "shared_level_draws":u8::from(self.recreate_shared_level),
            "battle_entry_advances":self.battle_entry_advances().to_string(),
            "battle_seed":self.battle_seed()
            ,"boost_advances":self.boost_plan.advances,"boost_menu":self.boost_plan.player_menu
        })
    }
    fn outcome_policies(&self) -> Vec<(String, &'static str, Option<u32>)> {
        vec![
            ("crackridge_approach".into(), "Avoid surprise in Crackridge Harbor", self.crackridge_approach_max_boost_repeats),
            ("mug_clerics".into(), "Clerics after Flats night switch", self.mug_clerics_max_boost_repeats),
            (
                "osvald_walk".into(),
                "Walk to Osvald without an encounter",
                self.flats_escape_settings.osvald_max_boost_repeats,
            ),
            (
                "village_snow_walk".into(),
                "Walk from Abandoned Village return to Southern Cape Cold Snows",
                self.flats_escape_settings.snow_max_boost_repeats,
            ),
            (
                "checkpoint".into(),
                "Reach opening checkpoint",
                self.event_caps.encounter,
            ),
            (
                "second_fight".into(),
                "One enemy action in second fight",
                self.event_caps.second_fight,
            ),
            (
                "mansion_approach".into(),
                "Avoid surprise in mansion",
                self.approach_cap,
            ),
            (
                "mansion_escape".into(),
                "Mansion Escape and safe run to Pirro",
                self.mansion_settings.run_max_boost_repeats,
            ),
            (
                "pirro_victory".into(),
                "Pirro victory",
                self.pirro_settings.victory_max_boost_repeats,
            ),
            (
                "pirro_no_grape".into(),
                "Pirro victory without Healing Grape",
                self.pirro_settings.no_grape_max_boost_repeats,
            ),
            (
                "highroad_approach".into(),
                "Avoid surprise on Highroad",
                self.highroad_settings.approach_max_boost_repeats,
            ),
            (
                "harbor_walk".into(),
                "Reach Harbor while walking",
                self.highroad_settings.flats_max_boost_repeats,
            ),
            (
                "flats_approach".into(),
                "Avoid surprise after Harbor return",
                self.highroad_settings.flats_approach_max_boost_repeats,
            ),
        ]
    }
    fn check_requirements(&self, result: &Value) -> Value {
        let f = &result["post_battle_field"];
        let safe = |v: &Value| matches!(v.as_str(), Some("Normal" | "Preemptive"));
        let won = f["pirro_battle"]["state"]["outcome"] == "victory"
            && f["pirro_battle"]["routine_error"].is_null();
        let unmet: Vec<_> = self
            .outcome_policies()
            .into_iter()
            .filter(|(_, _, cap)| cap.is_none())
            .filter_map(|(id, label, _)| {
                let met = match id.as_str() {
                    "crackridge_approach" => safe(&f["crackridge_escape"]["encounter"]["situation"]) && f["crackridge_escape"]["attempt"]["success"] == true,
                    "mug_clerics" => f["black_market"]["clerics_present"] == true,
                    "osvald_walk" => f["flats_escape"]["osvald"]["after_recruitment"].is_object(),
                    "village_snow_walk" => f["flats_escape"]["village_return"]["snow_reach"]["walking"]["reaches_reset"] == true,
                    "checkpoint" => f["reset_reach"]["walking"]["reaches_reset"] == true,
                    "second_fight" => {
                        f["second_battle"]["assessment"]["only_one_enemy_action"] == true
                            && f["second_battle"]["assessment"]["order_conditions_met"] == true
                    }
                    "mansion_approach" => {
                        safe(&f["second_battle"]["mansion_approach_search"]["situation"])
                    }
                    "mansion_escape" => f["mansion_escape"]["solution"]["reaches_pirro"] == true,
                    "pirro_victory" => won,
                    "pirro_no_grape" => {
                        won && f["pirro_battle"]["routine"]["heal_before_round_eight"] == false
                    }
                    "highroad_approach" => safe(&f["highroad"]["encounter"]["situation"]),
                    "harbor_walk" => f["flats"]["harbor_reach"]["walking"]["reaches_reset"] == true,
                    "flats_approach" => safe(&f["flats_escape"]["encounter"]["situation"]),
                    _ => false,
                };
                (!met).then(|| json!({"id":id,"label":label}))
            })
            .collect();
        let stages = [
            ("second_battle","Second fight"), ("mansion_escape","Mansion Escape"),
            ("pirro_battle","Pirro fight"), ("highroad","Highroad"),
            ("flats","Flats to harbor"), ("harbor_return","Harbor return"),
            ("flats_escape","Flats encounter"), ("cape_cold","Cape Cold"),
            ("cape_cold_mug","Cape Cold Mug"), ("black_market","Clerics"),
            ("crackridge_voyage","Crackridge voyage"), ("crackridge_escape","Crackridge Escape"),
        ];
        let blocker = if unmet.is_empty() { None } else {
            stages.iter().find_map(|(key,label)| {
                let stage=&f[*key];
                let reason=stage["error"].as_str().or(stage["routine_error"].as_str())
                    .or_else(|| (*key == "mansion_escape" && stage.is_object() && !stage["solution"].is_object())
                        .then(|| stage["status"].as_str()).flatten());
                reason.map(|reason| format!("{label}: {reason}"))
            })
        };
        let not_evaluated: Vec<_> = unmet.iter().filter(|item| {
            let stage=match item["id"].as_str().unwrap_or("") {
                "crackridge_approach"=>"crackridge_escape",
                "mug_clerics"=>"black_market", "osvald_walk"|"village_snow_walk"=>"flats_escape",
                "pirro_victory"|"pirro_no_grape"=>"pirro_battle", "highroad_approach"=>"highroad",
                "harbor_walk"=>"flats", "flats_approach"=>"flats_escape", _=>return false,
            };
            !f[stage].is_object()
        }).cloned().collect();
        json!({"met":unmet.is_empty(),"unmet":unmet,"blocker":blocker,"not_evaluated":not_evaluated})
    }
    // Outcome budgets and strategy thresholds are independent user settings.
    // Never silently promote an optional upstream target for a later event.
    fn execution_settings(&self) -> Self { self.clone() }
    fn simulate(&self) -> Result<Value, SimError> {
        let execution = self.execution_settings();
        let mut result = simulate_route(
            self.battle_seed(),
            self.boost_plan,
            execution.mansion_settings,
            self.approach_cap,
            self.second_plan,
            execution.pirro_settings,
            self.pirro_plan.clone(),
            execution.highroad_settings,
            self.pirro_r8_plan.clone(),
            execution.flats_escape_settings,
        )?;
        result["crt_setup"] = self.json();
        result["effective_search_settings"] = execution.json();
        let cape = &result["post_battle_field"]["cape_cold"];
        if cape["after_loading"].is_object() && cape["world_after"].is_object() {
            let before = serde_json::from_value(cape["after_loading"].clone())?;
            let world = serde_json::from_value(cape["world_after"].clone())?;
            let level=result["post_battle_field"]["highroad"]["level"].as_u64()
                .ok_or_else(||SimError("Missing Throné route level at Crackridge".into()))? as u32;
            match ot2_speedrun_assistant::crackridge::plan(before, world, level, self.mug_clerics_max_boost_repeats, self.crackridge_approach_max_boost_repeats) {
                Ok(p) => {
                    result["post_battle_field"]["crackridge_voyage"] = json!(p.voyage);
                    result["post_battle_field"]["crackridge_escape"] = json!(p.escape);
                    result["post_battle_field"]["cape_cold_mug"] = json!(p.market.mug);
                    result["post_battle_field"]["black_market"] = json!(p.market.route);
                    result["post_battle_field"]["black_market"]["search"] = json!(p.market.search);
                },
                Err(e) => result["post_battle_field"]["crackridge_escape"] = json!({"error":e.to_string()}),
            }
        }
        result["required_outcomes"] = self.check_requirements(&result);
        let second_plan =
            result["post_battle_field"]["second_battle"]["mansion_approach_search"]["plan"].clone();
        if !second_plan.is_null() {
            result["crt_setup"]["second_boost_menu"] = second_plan["player_menu"].clone();
            result["crt_setup"]["second_boost_advances"] = second_plan["advances"].clone();
            result["crt_setup"]["second_boost_repeat_cap"] = second_plan["repeat_cap"].clone();
        }
        let pirro_plan =
            result["post_battle_field"]["pirro_battle"]["boost_search"]["plan"].clone();
        if !pirro_plan.is_null() {
            result["crt_setup"]["pirro_boost_inputs"] = pirro_plan;
        }
        let r8_plan = result["post_battle_field"]["pirro_battle"]["r8_plan"].clone();
        if !r8_plan.is_null() {
            result["crt_setup"]["pirro_r8_boost_inputs"] = r8_plan;
        }
        if !result["post_battle_field"].is_null() {
            let total = self.battle_entry_advances() as u128
                + result["state"]["rng"]["draws"]
                    .as_u64()
                    .expect("battle draws") as u128;
            result["post_battle_field"]["seed"] = json!(self.seed);
            result["post_battle_field"]["advances"] = json!(total.to_string());
        }
        Ok(result)
    }
    fn calculate(self) -> Result<Value, SimError> {
        let search = ot2_speedrun_assistant::manipulation::search_with_event_caps(
            self.battle_seed(),
            self.execution_settings().event_caps,
        )?;
        let prediction = search
            .solution
            .as_ref()
            .map(|solution| {
                Self {
                    boost_plan: solution.plan,
                    ..self
                }
                .simulate()
            })
            .transpose()?;
        Ok(json!({"search":search,"prediction":prediction}))
    }
}

struct Response {
    status: u16,
    mime: &'static str,
    body: String,
    stop: bool,
    filename: Option<String>,
}
impl Response {
    fn new(status: u16, mime: &'static str, body: impl Into<String>) -> Self {
        Self {
            status,
            mime,
            body: body.into(),
            stop: false,
            filename: None,
        }
    }
    fn json(status: u16, value: Value) -> Self {
        Self::new(status, "application/json; charset=utf-8", value.to_string())
    }
    fn error(status: u16, message: &str) -> Self {
        Self::json(status, json!({"error":message}))
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EncounterRequest {
    seed: u32,
    // String preserves u64 precision in JavaScript.
    advances: String,
    options: ot2_speedrun_assistant::encounter::PredictionOptions,
    #[serde(default)]
    profile: Option<String>,
}
fn encounter_prediction(encoded: &str) -> Result<Value, SimError> {
    let mut bytes = Vec::new();
    let mut chars = encoded.bytes();
    while let Some(ch) = chars.next() {
        if ch == b'%' {
            let pair = [
                chars
                    .next()
                    .ok_or_else(|| SimError("Invalid URL escape".into()))?,
                chars
                    .next()
                    .ok_or_else(|| SimError("Invalid URL escape".into()))?,
            ];
            let hex =
                std::str::from_utf8(&pair).map_err(|_| SimError("Invalid URL escape".into()))?;
            bytes.push(
                u8::from_str_radix(hex, 16).map_err(|_| SimError("Invalid URL escape".into()))?,
            );
        } else {
            bytes.push(ch);
        }
    }
    let request: EncounterRequest = serde_json::from_slice(&bytes)?;
    if let Some(profile) = &request.profile {
        if profile != "throne_after_opening" {
            return Err(SimError("Unknown route profile".into()));
        }
        ot2_speedrun_assistant::route::validate_throne_field_options(&request.options)?;
    }
    let advances = request
        .advances
        .parse::<u64>()
        .map_err(|_| SimError("Invalid advance count".into()))?;
    let mut rng = ot2_battle_engine::CrtRng::new(request.seed);
    rng.advance(advances);
    let start = rng.state;
    let reset_reach = if request.profile.is_some() {
        Some(ot2_speedrun_assistant::route::throne_reset_reach(
            request
                .options
                .current_counter
                .expect("validated route counter"),
        )?)
    } else {
        None
    };
    if let Some(reach) = &reset_reach {
        let movement = if request.options.dash {
            &reach.running
        } else {
            &reach.walking
        };
        if movement.reaches_reset {
            // This path reaches the event before an encounter. Do not perform
            // the hypothetical encounter's reset, lottery or party timing draws.
            let gap = request
                .options
                .intervening_advances
                .parse::<u64>()
                .map_err(|_| SimError("Invalid intervening CRT advance count".into()))?;
            let mut end = CrtRng::new(start);
            end.advance(gap);
            let total = (advances as u128 + gap as u128).to_string();
            return Ok(json!({
                "kind":"reset_reached", "reset_reach":reach,
                "profile":ot2_speedrun_assistant::route::throne_field_profile(),
                "setup":{"seed":request.seed,"advances":request.advances,"starting_state":start,"options":request.options},
                "counter_before_reset":movement.counter_before_reset,
                "final_state":end.state,"draws":gap.to_string(),"total_advances":total,
                "endpoint":"At the next cutscene trigger, before its event or counter-reset callbacks",
                "checkpoints":[
                    {"label":"Post-battle field frame","state":start,"draws":"0","total_advances":request.advances},
                    {"label":"Next cutscene trigger, before reset","state":end.state,"draws":gap.to_string(),"total_advances":total}
                ]
            }));
        }
    }
    // Relative prediction counts do not wrap the absolute seed position.
    let mut local = ot2_battle_engine::CrtRng::new(start);
    let prediction =
        ot2_speedrun_assistant::encounter::catalog().predict(&mut local, &request.options)?;
    let mut value = serde_json::to_value(&prediction)?;
    for c in value["checkpoints"]
        .as_array_mut()
        .expect("checkpoint array")
    {
        c["total_advances"] =
            json!((advances as u128 + c["draws"].as_u64().expect("draws") as u128).to_string());
        c["draws"] = json!(c["draws"].as_u64().expect("draws").to_string());
    }
    value["total_advances"] = json!((advances as u128 + prediction.draws as u128).to_string());
    value["draws"] = json!(prediction.draws.to_string());
    value["setup"] = json!({"seed":request.seed,"advances":request.advances,"starting_state":start,"options":request.options});
    if request.profile.is_some() {
        value["profile"] =
            serde_json::to_value(ot2_speedrun_assistant::route::throne_field_profile())?;
        value["reset_reach"] = serde_json::to_value(reset_reach)?;
    }
    value["kind"] = json!("encounter_prediction");
    Ok(value)
}

fn route(request: &str, address: &str) -> Response {
    let mut lines = request.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    let (Some(method), Some(path), Some(version)) = (first.next(), first.next(), first.next())
    else {
        return Response::error(400, "Invalid HTTP request.");
    };
    if first.next().is_some() || !matches!(version, "HTTP/1.0" | "HTTP/1.1") {
        return Response::error(400, "Invalid HTTP request.");
    }
    let mut host = None;
    let mut origin = None;
    let mut viewer_header = false;
    let mut cross_site = false;
    for line in lines.take_while(|line| !line.is_empty()) {
        let Some((key, value)) = line.split_once(':') else {
            return Response::error(400, "Invalid header.");
        };
        let value = value.trim();
        if key.eq_ignore_ascii_case("host") {
            host = Some(value);
        }
        if key.eq_ignore_ascii_case("origin") {
            origin = Some(value);
        }
        if key.eq_ignore_ascii_case("x-ot2-viewer") {
            viewer_header = value == "1";
        }
        if key.eq_ignore_ascii_case("sec-fetch-site") {
            cross_site = value == "cross-site";
        }
    }
    let expected_origin = format!("http://{address}");
    if host != Some(address) || origin.is_some_and(|v| v != expected_origin) || cross_site {
        return Response::error(403, "Open this viewer at its local address.");
    }
    match (method, path) {
        ("GET", "/") => Response::new(200, "text/html; charset=utf-8", HTML),
        ("GET", "/viewer.css") => Response::new(200, "text/css; charset=utf-8", CSS),
        ("GET", "/viewer.js") => Response::new(200, "text/javascript; charset=utf-8", JS),
        ("GET", "/encounters.js") => Response::new(
            200,
            "text/javascript; charset=utf-8",
            include_str!("../../viewer/encounters.js"),
        ),
        ("GET", "/api/encounters/catalog") => {
            Response::json(200, ot2_speedrun_assistant::encounter::catalog().menu())
        }
        ("GET", path) if path.starts_with("/api/encounters/predict?request=") => {
            match encounter_prediction(&path["/api/encounters/predict?request=".len()..]) {
                Ok(value) => Response::json(200, value),
                Err(e) => Response::error(422, &e.to_string()),
            }
        }
        ("GET", "/api/health") => Response::json(200, json!({"app":"OT2 Battle Viewer"})),
        ("GET", path) if path.starts_with("/api/crt?") => {
            match CrtSetup::parse(&path["/api/crt?".len()..]) {
                Err(e) => Response::error(400, &e.to_string()),
                Ok(setup) => Response::json(200, setup.json()),
            }
        }
        ("GET", path) if path.starts_with("/api/simulate?") => {
            match CrtSetup::parse(&path["/api/simulate?".len()..]) {
                Err(e) => Response::error(400, &e.to_string()),
                Ok(setup) => match setup.simulate() {
                    Ok(value) => Response::json(200, value),
                    Err(e) => Response::error(422, &e.to_string()),
                },
            }
        }
        ("GET", path) if path.starts_with("/api/boost-search?") => {
            match CrtSetup::parse(&path["/api/boost-search?".len()..]) {
                Err(e) => Response::error(400, &e.to_string()),
                Ok(setup) => match setup.calculate() {
                    Ok(value) => Response::json(200, value),
                    Err(e) => Response::error(422, &e.to_string()),
                },
            }
        }
        ("GET", path) if path.starts_with("/api/export?") => {
            match CrtSetup::parse(&path["/api/export?".len()..]) {
                Err(e) => Response::error(400, &e.to_string()),
                Ok(setup) => match setup.simulate() {
                    Ok(value) => {
                        let mut response = Response::new(
                            200,
                            "application/json; charset=utf-8",
                            serde_json::to_string_pretty(&value).expect("serializable result"),
                        );
                        let suffix = if setup.recreate_shared_level {
                            "-recreated"
                        } else {
                            ""
                        };
                        response.filename = Some(format!(
                            "ot2-throne-seed-{}-title-advances-{}{suffix}{}.json",
                            setup.seed,
                            setup.advances,
                            if setup.boost_plan.advances > 0 {
                                format!(
                                    "-boost-menu-{}-advances-{}",
                                    setup.boost_plan.player_menu, setup.boost_plan.advances
                                )
                            } else {
                                String::new()
                            }
                        ));
                        response
                    }
                    Err(e) => Response::error(422, &e.to_string()),
                },
            }
        }
        ("POST", "/api/shutdown") if viewer_header => {
            let mut response = Response::json(200, json!({"closed":true}));
            response.stop = true;
            response
        }
        _ => Response::error(404, "Not found."),
    }
}

fn handle(mut stream: TcpStream, address: &str) -> std::io::Result<bool> {
    stream.set_read_timeout(Some(Duration::from_millis(750)))?;
    stream.set_write_timeout(Some(Duration::from_secs(3)))?;
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 2048];
    let response = loop {
        let count = stream.read(&mut chunk)?;
        if count == 0 {
            return Ok(false);
        }
        bytes.extend_from_slice(&chunk[..count]);
        if bytes.len() > 8192 {
            break Response::error(431, "Request too large.");
        }
        if bytes.windows(4).any(|v| v == b"\r\n\r\n") {
            break match std::str::from_utf8(&bytes) {
                Ok(request) => route(request, address),
                Err(_) => Response::error(400, "Invalid request encoding."),
            };
        }
    };
    let phrase = match response.status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        422 => "Unprocessable Content",
        431 => "Request Header Fields Too Large",
        _ => "Error",
    };
    let attachment = response.filename.as_ref().map_or_else(String::new, |name| {
        format!("Content-Disposition: attachment; filename=\"{name}\"\r\n")
    });
    write!(
        stream,
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n{attachment}Connection: close\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nReferrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'\r\n\r\n{}",
        response.status,
        phrase,
        response.mime,
        response.body.len(),
        response.body
    )?;
    Ok(response.stop)
}

fn open_browser(url: &str) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        Command::new("rundll32.exe")
            .args(["url.dll,FileProtocolHandler", url])
            .creation_flags(0x0800_0000)
            .spawn()?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }
    Ok(())
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut port = 0;
    let mut launch = true;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--no-open" => launch = false,
            "--port" => {
                port = args
                    .next()
                    .ok_or("--port requires a number")?
                    .parse::<u16>()?
            }
            _ => return Err("usage: ot2-gui [--no-open] [--port 0..65535]".into()),
        }
    }
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    let address = listener.local_addr()?.to_string();
    let url = format!("http://{address}/");
    println!("{url}");
    std::io::stdout().flush()?;
    if launch {
        open_browser(&url)?;
    }
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if handle(stream, &address).unwrap_or(false) {
                    break;
                }
            }
            Err(error) => eprintln!("Connection error: {error}"),
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("OT2 Battle Viewer: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn mansion_outcome_budget_and_escape_strategy_are_independent_settings() {
        use super::*;
        let p=CrtSetup::parse("seed=0&mansion_run_max_boost_repeats=0&mansion_escape_max_boost_repeats=4").unwrap();
        assert_eq!(p.mansion_settings.run_max_boost_repeats,Some(0));
        assert_eq!(p.mansion_settings.max_boost_repeats,Some(4));
        // Later default Required targets cannot rewrite this optional budget.
        assert_eq!(p.execution_settings().mansion_settings.run_max_boost_repeats,Some(0));
        assert_eq!(p.json()["search_policies"]["mansion_escape"]["mode"],"optional");
        let q=CrtSetup::parse("seed=0&mansion_run_max_boost_repeats=required&mansion_escape_max_boost_repeats=4").unwrap();
        assert!(q.mansion_settings.run_max_boost_repeats.is_none());
        assert_eq!(q.mansion_settings.max_boost_repeats,Some(4));
        assert_eq!(q.json()["search_policies"]["mansion_escape"]["mode"],"required");
        for bad in ["257","boost_only","4&mansion_run_max_boost_repeats=required"] {
            assert!(CrtSetup::parse(&format!("seed=0&mansion_run_max_boost_repeats={bad}")).is_err());
        }
    }
    #[test]
    fn required_route_continues_past_optional_mansion_escape_limit() {
        use super::*;
        let setup=CrtSetup::parse("seed=0&advances=4916&encounter_max_boost_repeats=required&pirro_victory_max_boost_repeats=required&pirro_no_grape_max_boost_repeats=required&highroad_approach_max_boost_repeats=required&highroad_flats_max_boost_repeats=required&highroad_flats_approach_max_boost_repeats=required&flats_escape_max_boost_repeats=boost_only").unwrap();
        assert_eq!(setup.mansion_settings.max_boost_repeats,Some(4));
        let result=setup.calculate().unwrap();
        let p=&result["prediction"];
        assert_eq!(p["required_outcomes"]["met"],true,"{}",p["required_outcomes"]);
        assert_eq!(p["crt_setup"]["mansion_escape_max_boost_repeats"],4);
        assert_eq!(p["effective_search_settings"]["mansion_escape_max_boost_repeats"],4);
        assert!(p["post_battle_field"]["mansion_escape"]["solution"]["counter_at_trigger"].as_u64().unwrap()>0);
        assert_eq!(p["post_battle_field"]["black_market"]["clerics_present"],true);
    }
    #[test]
    fn mug_clerics_setting_replays_route_and_keeps_upstream_rng() {
        use super::*;
        let query="seed=0&advances=4942&boost_menu=6&boost_advances=1&highroad_flats_max_boost_repeats=required&highroad_flats_approach_max_boost_repeats=required";
        let required=CrtSetup::parse(query).unwrap();
        assert!(required.mug_clerics_max_boost_repeats.is_none());
        let p=required.simulate().unwrap();
        let optional=CrtSetup::parse(&format!("{query}&mug_clerics_max_boost_repeats=0")).unwrap().simulate().unwrap();
        let f=&p["post_battle_field"];
        assert_eq!(f["black_market"]["clerics_present"],true);
        assert_eq!(f["black_market"]["search"]["extra_advances"],2);
        assert_eq!(f["black_market"]["search"]["cape_time_switches"],1);
        assert_eq!(f["black_market"]["search"]["weighted_input_cost"],18);
        assert_eq!(f["black_market"]["switched_at_village"],false);
        assert_eq!(f["cape_cold_mug"]["mug"]["survived_round_two"],false);
        for key in ["cape_cold","flats_escape","pirro_battle"] {
            assert_eq!(f[key],optional["post_battle_field"][key]);
        }
        assert_eq!(optional["post_battle_field"]["black_market"]["search"]["extra_advances"],0);
        assert_eq!(optional["post_battle_field"]["black_market"]["search"]["met"],true);
        let offset=required.battle_entry_advances();
        assert_eq!(offset+f["black_market"]["after_night"]["draws"].as_u64().unwrap(),6268);
        for bad in ["257","boost_only","4&mug_clerics_max_boost_repeats=required"] {
            assert!(CrtSetup::parse(&format!("seed=0&mug_clerics_max_boost_repeats={bad}")).is_err());
        }
    }
    #[test]
    fn osvald_prediction_propagates_flats_plan_and_independent_setting() {
        use super::*;
        let query = "seed=0&advances=4942&boost_menu=6&boost_advances=1&encounter_max_boost_repeats=required&pirro_victory_max_boost_repeats=required&pirro_no_grape_max_boost_repeats=required&highroad_approach_max_boost_repeats=required&highroad_flats_max_boost_repeats=required&highroad_flats_approach_max_boost_repeats=required&flats_escape_max_boost_repeats=boost_only&mug_clerics_max_boost_repeats=0";
        let setup = CrtSetup::parse(query).unwrap();
        let result = setup.simulate().unwrap();
        let f = &result["post_battle_field"]["flats_escape"];
        let o = &f["osvald"];
        // The lowercase _sound package must participate before counter B/A.
        // Omitting it rejected this valid one-cycle plan and chose six cycles.
        assert_eq!(f["escape"]["inputs"]["extra_repeats"], 1);
        assert_eq!(o["world_at_entry"]["encounter_counter"], 727);
        assert_eq!(o["walking"]["counter_before_reset"], 97);
        let offset = setup.battle_entry_advances();
        assert_eq!(offset + o["after_loading"]["draws"].as_u64().unwrap(), 6130);
        assert_eq!(
            offset + o["after_recruitment"]["draws"].as_u64().unwrap(),
            6132
        );
        // Keep the previously executed six-cycle route as an independent
        // reported endpoint, even though the optimizer now needs just one.
        let mut six_cycle_arrival = CrtRng::new(0);
        six_cycle_arrival.advance(6121);
        let six_cycle = ot2_speedrun_assistant::osvald::with_world_state(
            six_cycle_arrival,
            serde_json::from_value(f["village_return"]["world_after"].clone()).unwrap(),
        )
        .unwrap();
        assert_eq!(six_cycle.after_loading.draws, 6135);
        assert_eq!(six_cycle.after_loading.draws - six_cycle.arrival.draws, 14);
        let loaded: Vec<_> = six_cycle
            .trace
            .iter()
            .filter(|t| t.kind == ot2_speedrun_assistant::world::DrawKind::StreamedLevel)
            .map(|t| t.subject.as_str())
            .collect();
        assert_eq!(
            loaded,
            [
                "Fld_Snw_1_2",
                "Fld_Snw_1_2_Art",
                "Fld_Snw_1_2_Coll",
                "Fld_Snw_1_2_GD",
                "Fld_Snw_1_2_sound",
                "Btl_Fld_Snw_Land_A",
                "Btl_Fld_Snw_River_A"
            ]
        );
        assert_eq!(o["arrival"], f["village_return"]["at_snow_entrance"]);
        assert_eq!(o["walking"]["reaches_reset"], true);
        assert_eq!(o["walking"]["cost"], 630);
        assert_eq!(o["world_after"]["party"], json!([7, 6]));
        let cape = &result["post_battle_field"]["cape_cold"];
        assert_eq!(cape["arrival"], o["after_recruitment"]);
        assert_eq!(cape["at_entrance"], cape["arrival"]);
        assert_eq!(cape["reach"]["initial_counter"], 552);
        assert_eq!(cape["reach"]["walking"]["cost"], 486);
        assert_eq!(cape["reach"]["running"]["cost"], 612);
        assert_eq!(cape["reach"]["mixed"]["running_distance_percent"], 51.5);
        assert_eq!(cape["world_after"]["map"], "Twn_Snw_1_2_A");
        assert_eq!(cape["world_after"]["party"], json!([7, 6]));
        assert!(
            cape["world_after"]["visited_maps"]
                .as_array()
                .unwrap()
                .contains(&json!("Twn_Snw_1_2_A"))
        );
        let mut last = cape["arrival"].clone();
        for t in cape["trace"].as_array().unwrap() {
            assert_eq!(t["before"], last);
            last = t["after"].clone();
        }
        assert_eq!(last, cape["after_loading"]);
        // Mug inherits the selected route's exact RNG and world state. Its
        // reports already include the route-relative offset; add only title
        // and opening draws to display counts from the initial seed.
        let mug = &result["post_battle_field"]["cape_cold_mug"];
        assert_eq!(mug["before"], cape["after_loading"]);
        assert_eq!(offset + mug["after_night"]["draws"].as_u64().unwrap(), 6163);
        assert_eq!(
            offset + mug["before_battle"]["draws"].as_u64().unwrap(),
            6165
        );
        assert_eq!(
            mug["mug"]["menus"]
                .as_array()
                .unwrap()
                .iter()
                .map(|m| offset + m["rng"]["draws"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [6192, 6218, 6289]
        );
        let damage: Vec<_> = mug["mug"]["steps"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|s| s["events"].as_array().unwrap())
            .filter(|e| e["kind"] == "damage")
            .map(|e| e["amount"].as_u64().unwrap())
            .collect();
        assert_eq!(damage, [53, 226, 11]);
        assert_eq!(mug["mug"]["survived_round_two"], true);
        assert_eq!(
            offset + mug["mug"]["after"]["draws"].as_u64().unwrap(),
            6297
        );
        assert_eq!(offset + mug["after"]["draws"].as_u64().unwrap(), 6299);
        assert_eq!(mug["after"]["state"], 0xdc86178d_u32);
        assert_eq!(mug["world_after"]["time"], 4);
        // Invalid movement cannot fabricate a town endpoint or consume draws.
        let mut blocked: ot2_speedrun_assistant::world::WorldState =
            serde_json::from_value(o["world_after"].clone()).unwrap();
        blocked.encounter_counter = 486;
        let no_entry = ot2_speedrun_assistant::cape_cold::with_world_state(
            serde_json::from_value(o["after_recruitment"].clone()).unwrap(),
            blocked,
        )
        .unwrap();
        assert!(no_entry.after_loading.is_none());
        assert!(no_entry.world_after.is_none());
        assert!(no_entry.trace.is_empty());
        assert!(
            ot2_speedrun_assistant::cape_cold_mug::after_cape_cold(&no_entry)
                .unwrap()
                .is_none()
        );
        println!(
            "Cape Cold: {}",
            json!({"entry":offset+cape["after_loading"]["draws"].as_u64().unwrap(),"hex":format!("0x{:08X}",cape["after_loading"]["state"].as_u64().unwrap()),"trace":cape["trace"]})
        );
        assert_eq!(result["required_outcomes"]["met"], true);
        assert_eq!(
            result["crt_setup"]["search_policies"]["osvald_walk"]["mode"],
            "required"
        );
        let address = "127.0.0.1:9999";
        let response = route(
            &format!("GET /api/export?{query} HTTP/1.1\r\nHost: {address}\r\n\r\n"),
            address,
        );
        assert_eq!(response.status, 200);
        assert_eq!(
            serde_json::from_str::<Value>(&response.body).unwrap(),
            result
        );
        let optional =
            CrtSetup::parse(&format!("{query}&flats_osvald_max_boost_repeats=0")).unwrap();
        assert_eq!(
            optional.flats_escape_settings.osvald_max_boost_repeats,
            Some(0)
        );
        assert!(
            optional
                .flats_escape_settings
                .snow_max_boost_repeats
                .is_none()
        );
        let previous = optional.simulate().unwrap();
        for key in ["highroad", "harbor_return", "pirro_battle"] {
            assert_eq!(
                result["post_battle_field"][key],
                previous["post_battle_field"][key]
            );
        }
        for bad in [
            "257",
            "boost_only",
            "4&flats_osvald_max_boost_repeats=required",
        ] {
            assert!(
                CrtSetup::parse(&format!("seed=0&flats_osvald_max_boost_repeats={bad}")).is_err()
            );
        }
        println!(
            "Osvald route: {}",
            json!({"Flats_repeats":f["escape"]["inputs"]["extra_repeats"],"village_return":offset+f["village_return"]["after"]["draws"].as_u64().unwrap(),"snow_entry":offset+o["after_loading"]["draws"].as_u64().unwrap(),"counter":o["world_at_entry"]["encounter_counter"],"walk_remaining":o["walking"]["counter_before_reset"],"after_recruitment":offset+o["after_recruitment"]["draws"].as_u64().unwrap(),"hex":format!("0x{:08X}",o["after_recruitment"]["state"].as_u64().unwrap()),"trace":o["trace"]})
        );
    }
    #[test]
    fn second_boost_search_propagates_without_changing_the_opening_plan() {
        let base =
            super::CrtSetup::parse("seed=0&advances=4904&mansion_approach_max_boost_repeats=0")
                .unwrap()
                .calculate()
                .unwrap();
        let searched =
            super::CrtSetup::parse("seed=0&advances=4904&mansion_approach_max_boost_repeats=1")
                .unwrap()
                .calculate()
                .unwrap();
        let baseline = &base["prediction"];
        let p = &searched["prediction"];
        let old_second = &baseline["post_battle_field"]["second_battle"];
        let second = &p["post_battle_field"]["second_battle"];
        let search = &second["mansion_approach_search"];
        assert_eq!(
            old_second["mansion_approach_search"]["situation"],
            "Surprise"
        );
        assert_eq!(old_second["mansion_approach_search"]["limit_reached"], true);
        assert_eq!(search["situation"], "Normal");
        assert_eq!(search["target_met"], true);
        assert_eq!(search["plan"]["advances"], 1);
        assert_eq!(search["plan"]["player_menu"], 6);
        assert_eq!(second["manipulation"]["inputs"]["extra_input_count"], 2);
        assert_eq!(second["manipulation"]["inputs"]["extra_repeats"], 1);
        // Zero extra presses fail; returning to the same selected BP requires
        // an even number of presses, so two is the global minimum.
        for key in ["state", "steps", "manipulation"] {
            assert_eq!(p[key], baseline[key]);
        }
        assert_eq!(second["entry"], old_second["entry"]);
        assert_eq!(second["initial_state"], old_second["initial_state"]);
        assert_eq!(old_second["assessment"]["only_one_enemy_action"], true);
        assert_eq!(second["assessment"]["only_one_enemy_action"], true);
        assert_ne!(second["after"], old_second["after"]);
        assert_ne!(
            second["mansion"]["after_entry"],
            old_second["mansion"]["after_entry"]
        );
        assert_eq!(
            p["post_battle_field"]["mansion_escape"]["first_encounter"]["situation"],
            "Normal"
        );
        let c = &p["crt_setup"];
        let query = format!(
            "seed=0&advances=4904&boost_menu={}&boost_advances={}{}&mansion_approach_max_boost_repeats=1&second_boost_menu=6&second_boost_advances=1&second_boost_repeat_cap=1",
            c["boost_menu"],
            c["boost_advances"],
            c["boost_repeat_cap"]
                .as_u64()
                .map(|n| format!("&boost_repeat_cap={n}"))
                .unwrap_or_default()
        );
        let replay = super::CrtSetup::parse(&query).unwrap().simulate().unwrap();
        for key in ["state", "steps", "manipulation", "mansion"] {
            assert_eq!(
                replay["post_battle_field"]["second_battle"][key],
                second[key]
            );
        }
        assert_eq!(
            replay["post_battle_field"]["mansion_escape"],
            p["post_battle_field"]["mansion_escape"]
        );
        assert_eq!(replay["crt_setup"], p["crt_setup"]);
        let unlimited = super::CrtSetup::parse(
            "seed=0&advances=4904&mansion_approach_max_boost_repeats=unlimited",
        )
        .unwrap()
        .calculate()
        .unwrap();
        let u = &unlimited["prediction"]["post_battle_field"]["second_battle"];
        assert!(u["mansion_approach_search"]["max_boost_repeats"].is_null());
        assert_eq!(u["state"], second["state"]);
        for query in [
            "mansion_approach_max_boost_repeats=257",
            "mansion_approach_max_boost_repeats=1&mansion_approach_max_boost_repeats=unlimited",
            "second_boost_menu=7",
            "second_boost_advances=1",
        ] {
            assert!(super::CrtSetup::parse(&format!("seed=0&{query}")).is_err());
        }
    }

    #[test]
    fn independent_pirro_limits_round_trip_through_api_and_export() {
        for (victory, grape) in [("0", "unlimited"), ("unlimited", "0"), ("1", "3")] {
            let query = format!(
                "seed=0&advances=4942&pirro_victory_max_boost_repeats={victory}&pirro_no_grape_max_boost_repeats={grape}"
            );
            let data = super::CrtSetup::parse(&query).unwrap().calculate().unwrap();
            let p = &data["prediction"];
            let boss = &p["post_battle_field"]["pirro_battle"];
            let cap = |value: &str| {
                if value == "unlimited" {
                    serde_json::Value::Null
                } else {
                    serde_json::json!(value.parse::<u32>().unwrap())
                }
            };
            assert_eq!(
                p["crt_setup"]["pirro_victory_max_boost_repeats"],
                cap(victory)
            );
            assert_eq!(
                p["crt_setup"]["pirro_no_grape_max_boost_repeats"],
                cap(grape)
            );
            assert_eq!(
                boss["boost_search"]["settings"]["victory_max_boost_repeats"],
                p["effective_search_settings"]["pirro_victory_max_boost_repeats"]
            );
            assert_eq!(
                boss["boost_search"]["settings"]["no_grape_max_boost_repeats"],
                cap(grape)
            );
            // This route already wins without a grape with the corrected
            // item timer gate, including with zero search allowance.
            assert_eq!(boss["boost_search"]["baseline_heal"], false);
            assert_eq!(boss["boost_search"]["target_met"], true);
            let setup = &p["crt_setup"];
            let inputs = setup["pirro_boost_inputs"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap().to_string())
                .collect::<Vec<_>>()
                .join(",");
            let replay_query = format!(
                "{query}&boost_menu={}&boost_advances={}&boost_repeat_cap={}&second_boost_menu={}&second_boost_advances={}&pirro_boost_inputs={inputs}",
                setup["boost_menu"],
                setup["boost_advances"],
                setup["boost_repeat_cap"],
                setup["second_boost_menu"],
                setup["second_boost_advances"]
            );
            let request =
                format!("GET /api/export?{replay_query} HTTP/1.1\r\nHost: 127.0.0.1:12345\r\n\r\n");
            let exported = super::route(&request, "127.0.0.1:12345");
            assert_eq!(exported.status, 200, "{}", exported.body);
            let exported: serde_json::Value = serde_json::from_str(&exported.body).unwrap();
            assert_eq!(
                exported["post_battle_field"]["pirro_battle"]["state"],
                boss["state"]
            );
            assert_eq!(
                exported["crt_setup"]["pirro_victory_max_boost_repeats"],
                cap(victory)
            );
            assert_eq!(
                exported["crt_setup"]["pirro_no_grape_max_boost_repeats"],
                cap(grape)
            );
        }
    }
    #[test]
    fn post_pirro_export_propagates_victory_through_ending_without_rebasing_twice() {
        let request = "GET /api/export?seed=0&advances=4942&boost_menu=6&boost_advances=1 HTTP/1.1\r\nHost: 127.0.0.1:12345\r\n\r\n";
        let response = super::route(request, "127.0.0.1:12345");
        assert_eq!(response.status, 200, "{}", response.body);
        let route: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        let field = &route["post_battle_field"];
        let ending = &field["pirro_aftermath"];
        assert_eq!(ending["arrival"], field["pirro_battle"]["after_victory"]);
        let base: u64 = route["crt_setup"]["battle_entry_advances"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        assert_eq!(base + ending["arrival"]["draws"].as_u64().unwrap(), 5900);
        assert_eq!(base + ending["after"]["draws"].as_u64().unwrap(), 5962);
        assert_eq!(ending["after"]["state"], 0x6e0026c2_u32);
        assert_eq!(ending["world_after"]["map"], "Fld_Cty_1_3");
        assert_eq!(ending["world_after"]["encounter_counter"], 862);
        let mut previous = &ending["arrival"];
        for span in ending["trace"].as_array().unwrap() {
            assert_eq!(&span["before"], previous);
            previous = &span["after"];
        }
        assert_eq!(previous, &ending["after"]);
    }
    #[test]
    fn harbor_tag_return_reloads_both_maps_and_preserves_progression() {
        use ot2_speedrun_assistant::{
            CrtRng, harbor,
            world::{DrawKind, WorldState},
        };
        // Keep the historical five-cycle fixture, with the new target disabled.
        let setup = CrtSetup::parse("seed=0&advances=4942&boost_menu=6&boost_advances=1&highroad_flats_approach_max_boost_repeats=0").unwrap();
        let p = setup.simulate().unwrap();
        let flats = &p["post_battle_field"]["flats"];
        let h = &p["post_battle_field"]["harbor_return"];
        assert!(h["error"].is_null(), "{h}");
        assert_eq!(h["arrival"], flats["after"]);
        let origin = setup.battle_entry_advances();
        assert_eq!(origin + h["at_harbor"]["draws"].as_u64().unwrap(), 6039);
        assert_eq!(origin + h["after"]["draws"].as_u64().unwrap(), 6053);
        assert_eq!(h["after"]["state"], 0x04a0b097_u32);
        assert_eq!(h["world_after"]["encounter_counter"], 515);
        assert_eq!(h["world_after"]["first_encounter"], true);
        assert_eq!(h["world_after"]["map"], "Fld_Cty_1_2");
        assert!(
            h["world_after"]["visited_maps"]
                .as_array()
                .unwrap()
                .contains(&json!("Fld_Cty_1_1"))
        );
        let trace = h["trace"].as_array().unwrap();
        let count = |kind| trace.iter().filter(|t| t["kind"] == kind).count();
        assert_eq!(
            (
                count("streamed_level"),
                count("environment_initialization"),
                count("npc_frame"),
                count("counter")
            ),
            (11, 2, 6, 8)
        );
        assert_eq!(
            trace
                .iter()
                .filter(|t| t["kind"] == "streamed_level" && t["subject"] == "Btl_Fld_Cty_Land_A")
                .count(),
            2
        );
        assert!(!trace.iter().any(|t| t["subject"] == "Btl_SinExpose"));
        let mut previous = &h["arrival"];
        for t in trace {
            assert_eq!(&t["before"], previous);
            previous = &t["after"];
        }
        assert_eq!(previous, &h["after"]);

        let arrival: CrtRng = serde_json::from_value(flats["after"].clone()).unwrap();
        let state: WorldState = serde_json::from_value(flats["world_after"].clone()).unwrap();
        // Already tagging the harbor changes visit metadata, not load calls.
        let mut tagged = state.clone();
        tagged.visited_maps.insert("Fld_Cty_1_1".into());
        let repeated = harbor::with_world_state(arrival, tagged).unwrap();
        assert_eq!(serde_json::to_value(repeated.after).unwrap(), h["after"]);
        // Actual story gate 6717 enables nine additional daytime harbor NPCs.
        let mut progressed = state.clone();
        progressed.set_flag(6717, true);
        let later = harbor::with_world_state(arrival, progressed).unwrap();
        assert_eq!(
            later
                .trace
                .iter()
                .filter(|t| t.kind == DrawKind::NpcFrame)
                .count(),
            15
        );
        assert_eq!(later.after.draws, repeated.after.draws + 9);
        // Changing map/progression must be evaluated, not replaced with +27.
        assert_ne!(later.after.state, repeated.after.state);
        let mut unsafe_walk = state;
        unsafe_walk.encounter_counter = 549;
        assert!(harbor::with_world_state(arrival, unsafe_walk).is_err());
    }

    #[test]
    fn flats_repeat_setting_is_independent_and_round_trips_unlimited() {
        let defaults = CrtSetup::parse("seed=0&advances=4942").unwrap();
        assert_eq!(defaults.highroad_settings.flats_max_boost_repeats, Some(4));
        let setup = CrtSetup::parse("seed=0&advances=4942&boost_menu=6&boost_advances=1&highroad_escape_max_boost_repeats=0&highroad_flats_max_boost_repeats=unlimited").unwrap();
        assert_eq!(setup.highroad_settings.escape_max_boost_repeats, Some(0));
        assert_eq!(setup.highroad_settings.flats_max_boost_repeats, None);
        let p = setup.simulate().unwrap();
        assert!(p["crt_setup"]["highroad_flats_max_boost_repeats"].is_null());
        assert_eq!(p["crt_setup"]["highroad_escape_max_boost_repeats"], 0);
        assert!(
            p["post_battle_field"]["highroad"]["escape"]["flats_search"]["max_extra_boost_repeats"]
                .is_null()
        );
        assert_eq!(
            p["post_battle_field"]["highroad"]["escape"]["flats_search"]["target_met"],
            true
        );
    }

    #[test]
    fn flats_approach_propagates_highroad_plan_and_escape_replays_to_village() {
        use ot2_speedrun_assistant::{flats, flats_escape, harbor, highroad, world::WorldState};
        let query = "seed=0&advances=4942&boost_menu=6&boost_advances=1";
        let mut setup = CrtSetup::parse(query).unwrap();
        setup.highroad_settings.flats_approach_max_boost_repeats = Some(0);
        let baseline = setup.simulate().unwrap();
        assert_eq!(
            baseline["post_battle_field"]["flats_escape"]["encounter"]["situation"],
            "Surprise"
        );
        assert!(baseline["post_battle_field"]["flats_escape"]["escape"].is_null());
        let old = &baseline["post_battle_field"]["highroad"];
        let old_repeats = old["escape"]["inputs"]["extra_repeats"].as_u64().unwrap();
        setup.highroad_settings.flats_approach_max_boost_repeats = None;
        let selected = setup.simulate().unwrap();
        let high = &selected["post_battle_field"]["highroad"];
        let search = &high["escape"]["flats_approach_search"];
        assert_eq!(search["target_met"], true);
        let n = high["escape"]["inputs"]["extra_repeats"].as_u64().unwrap();
        assert_eq!(search["prior_repeats"], n);
        assert!(n > old_repeats);
        assert_eq!(search["added_repeats"], 0);
        assert_eq!(selected["steps"], baseline["steps"]);
        assert_eq!(
            selected["post_battle_field"]["pirro_battle"],
            baseline["post_battle_field"]["pirro_battle"]
        );
        let world: WorldState = serde_json::from_value(high["world_after_al"].clone()).unwrap();
        let entry: CrtRng =
            serde_json::from_value(high["before_battle_initialization"].clone()).unwrap();
        let enemies: Vec<_> = high["encounter"]["enemies"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["enemy"]["id"].as_str().unwrap())
            .collect();
        let menu = ot2_battle_engine::field_escape::FieldEscapeMenu::new(
            entry,
            ot2_battle_engine::escape::Situation::Normal,
            5,
            &enemies,
        )
        .unwrap();
        for repeats in 0..=n {
            let inputs = (0..repeats).flat_map(|_| [1, 0]).collect::<Vec<_>>();
            let a = menu.clone().attempt(&inputs, 5).unwrap();
            if !a.success {
                assert_ne!(repeats, n);
                continue;
            }
            let f = flats::after_highroad(&highroad::return_to_field(&world, &a).unwrap()).unwrap();
            let Some(h) = harbor::after_flats(&f).unwrap() else {
                assert_ne!(repeats, n);
                continue;
            };
            let (_, enc) = flats_escape::first_encounter(h.after, &h.world_after, 5).unwrap();
            assert_eq!(enc.situation.as_deref() != Some("Surprise"), repeats == n);
        }
        let e = &selected["post_battle_field"]["flats_escape"];
        assert!(e["error"].is_null(), "{e}");
        assert_eq!(
            e["arrival"],
            selected["post_battle_field"]["harbor_return"]["after"]
        );
        assert_eq!(e["run_updates"], 170);
        assert_eq!(e["escape"]["attempt"]["success"], true);
        assert_eq!(e["remaining_run"]["reaches_reset"], true);
        assert_eq!(e["at_village_entrance"], e["field_return"]["after"]);
        let village = &e["village_return"];
        assert_eq!(village["arrival"], e["at_village_entrance"]);
        assert_eq!(village["world_at_village"]["map"], "Twn_Cty_1_2_A");
        assert_eq!(village["world_after"]["map"], "Fld_Cty_1_2");
        assert_eq!(village["world_after"]["first_encounter"], true);
        assert_eq!(village["snow_reach"]["walking"]["cost"], 648);
        assert_eq!(village["snow_reach"]["running"]["cost"], 816);
        assert_eq!(village["snow_reach"]["walking"]["reaches_reset"], true);
        assert_eq!(village["at_snow_entrance"], village["after"]);
        assert!(selected["crt_setup"]["flats_snow_max_boost_repeats"].is_null());
        assert_eq!(
            selected["crt_setup"]["search_policies"]["village_snow_walk"]["mode"],
            "required"
        );
        let after: CrtRng =
            serde_json::from_value(selected["post_battle_field"]["harbor_return"]["after"].clone())
                .unwrap();
        let world: WorldState = serde_json::from_value(
            selected["post_battle_field"]["harbor_return"]["world_after"].clone(),
        )
        .unwrap();
        let replay = flats_escape::predict(after, &world, 5, setup.flats_escape_settings).unwrap();
        assert_eq!(serde_json::to_value(replay).unwrap(), *e);
        for (cap, met) in [(n - old_repeats - 1, false), (n - old_repeats, true)] {
            setup.highroad_settings.flats_approach_max_boost_repeats = Some(cap as u32);
            let p = setup.simulate().unwrap();
            assert_eq!(
                p["post_battle_field"]["highroad"]["escape"]["flats_approach_search"]["target_met"],
                met
            );
            if !met {
                assert_eq!(
                    p["post_battle_field"]["highroad"]["escape"]["inputs"],
                    old["escape"]["inputs"]
                );
            }
        }
        let limits=CrtSetup::parse(&format!("{query}&highroad_flats_approach_max_boost_repeats=unlimited&flats_escape_max_boost_repeats=unlimited")).unwrap();
        assert!(limits.json()["highroad_flats_approach_max_boost_repeats"].is_null());
        assert!(limits.json()["flats_escape_max_boost_repeats"].is_null());
        assert_eq!(limits.highroad_settings.flats_max_boost_repeats, Some(4));
        for bad in [
            "flats_escape_max_boost_repeats=257",
            "highroad_flats_approach_max_boost_repeats=-1",
            "flats_escape_max_boost_repeats=0&flats_escape_max_boost_repeats=unlimited",
            "flats_snow_max_boost_repeats=257",
            "flats_snow_max_boost_repeats=boost_only",
            "flats_snow_max_boost_repeats=0&flats_snow_max_boost_repeats=required",
        ] {
            assert!(CrtSetup::parse(&format!("{query}&{bad}")).is_err());
        }
        println!(
            "{}",
            json!({"highroad_repeats":n,"harbor_return":selected["post_battle_field"]["harbor_return"]["after"],"entry_counter":world.encounter_counter,"flats_method":e["escape"]["method"],"flats_inputs":e["escape"]["inputs"],"flats_menu":e["escape"]["menu"]["rng"],"field_return":e["field_return"]["after"],"remaining":e["remaining_run"],"origin":setup.battle_entry_advances()})
        );
    }

    #[test]
    fn highroad_stage_preserves_al_and_first_escape_boundaries_and_independent_caps() {
        let mut setup = CrtSetup::parse("seed=0&advances=4942&boost_menu=6&boost_advances=1&highroad_escape_max_boost_repeats=4&highroad_flats_approach_max_boost_repeats=0").unwrap();
        let capped = setup.simulate().unwrap();
        let h = &capped["post_battle_field"]["highroad"];
        assert!(h["error"].is_null(), "{h}");
        let base = setup.battle_entry_advances();
        assert_eq!(base + h["after_al"]["draws"].as_u64().unwrap(), 5965);
        assert_eq!(h["after_al"]["state"], 0xd12b9cdf_u32);
        assert_eq!(h["level"], 5);
        assert_eq!(h["encounter"]["situation"], "Normal");
        assert_eq!(h["encounter"]["group"], "ENG_FLD_CTY_1_3_LND_DAY_150");
        assert_eq!(
            base + h["escape"]["menu"]["rng"]["draws"].as_u64().unwrap(),
            6002
        );
        assert_eq!(h["escape"]["baseline"]["chance_percent"], 40);
        assert_eq!(h["escape"]["baseline"]["success"], false);
        assert_eq!(h["escape"]["method"], "unlimited_fallback");
        assert_eq!(
            h["escape"]["inputs"]["levels"],
            json!([1, 0, 1, 0, 1, 0, 1, 0, 1, 0])
        );
        assert!(h["escape"]["break_action"].is_null());
        let field = &h["field_return"];
        assert_eq!(field["arrival"], h["escape"]["attempt"]["after"]);
        assert_eq!(base + field["after"]["draws"].as_u64().unwrap(), 6012);
        assert_eq!(field["after"]["state"], 0x2e4773fc_u32);
        assert_eq!(field["world_after"]["first_encounter"], false);
        assert_eq!(field["world_after"]["encounter_counter"], 1448);
        let flats = &capped["post_battle_field"]["flats"];
        assert!(flats["error"].is_null(), "{flats}");
        assert_eq!(flats["arrival"], field["after"]);
        assert_eq!(base + flats["after"]["draws"].as_u64().unwrap(), 6026);
        assert_eq!(flats["after"]["state"], 0xf0f1e302_u32);
        assert_eq!(flats["world_after"]["map"], "Fld_Cty_1_2");
        assert_eq!(flats["world_after"]["first_encounter"], true);
        assert_eq!(flats["world_after"]["encounter_counter"], 717);
        assert_eq!(h["escape"]["flats_search"]["target_met"], true);
        assert_eq!(h["escape"]["flats_search"]["added_repeats"], 0);
        assert_eq!(
            flats["harbor_reach"]["mixed"]["running_distance_percent"],
            100.0
        );
        let live: Value = serde_json::from_str(include_str!(
            "../../data/source/live-highroad-escape-6002.json"
        ))
        .unwrap();
        let mut observed = live["records"].as_array().unwrap().iter();
        for span in h["escape"]["attempt"]["trace"]
            .as_array()
            .unwrap()
            .iter()
            .chain(field["trace"].as_array().unwrap())
        {
            let mut rng: CrtRng = serde_json::from_value(span["before"].clone()).unwrap();
            let after: CrtRng = serde_json::from_value(span["after"].clone()).unwrap();
            while rng.draws < after.draws {
                if base + rng.draws >= 6002 {
                    let row = observed.next().expect("extra simulated CRT draw");
                    assert_eq!(row["position"], base + rng.draws);
                    assert_eq!(row["before"], rng.state);
                    let position = row["position"].as_u64().unwrap();
                    let reason = span["reason"].as_str().unwrap();
                    match position {
                        6002..=6006 => assert_eq!(reason, "boost_voice"),
                        6007 => assert_eq!(reason, "escape_check"),
                        6008..=6009 => {
                            assert!(reason.ends_with("Canoe initial bobbing time"));
                            assert!(row["object"].as_str().unwrap().ends_with(
                                span["subject"].as_str().unwrap().split_once('.').unwrap().1
                            ));
                        }
                        6010 => assert!(reason.ends_with("counter B")),
                        6011 => assert!(reason.ends_with("counter A")),
                        _ => panic!("unexpected captured draw"),
                    }
                }
                rng.next_u15();
            }
            assert_eq!(rng, after);
        }
        assert!(observed.next().is_none(), "missing simulated CRT draws");
        let mut previous = &field["arrival"];
        for span in field["trace"]
            .as_array()
            .unwrap()
            .iter()
            .chain(flats["trace"].as_array().unwrap())
        {
            assert_eq!(&span["before"], previous);
            previous = &span["after"];
        }
        assert_eq!(previous, &flats["after"]);
        setup.highroad_settings.escape_max_boost_repeats = None;
        let unlimited = setup.simulate().unwrap();
        let u = &unlimited["post_battle_field"]["highroad"];
        assert_eq!(u["escape"]["method"], "within_limit");
        assert_eq!(u["field_return"], h["field_return"]);
        assert_eq!(unlimited["post_battle_field"]["flats"], *flats);
        assert_eq!(
            u["escape"]["inputs"]["levels"],
            json!([1, 0, 1, 0, 1, 0, 1, 0, 1, 0])
        );
        assert_eq!(u["escape"]["attempt"]["success"], true);
        assert_eq!(
            base + u["escape"]["attempt"]["after"]["draws"].as_u64().unwrap(),
            6008
        );
        assert_eq!(
            capped["post_battle_field"]["pirro_battle"]["state"],
            unlimited["post_battle_field"]["pirro_battle"]["state"]
        );
    }
    #[test]
    fn r8_search_freezes_upstream_inputs_and_replays_exact_sword_path() {
        let query = "seed=0&advances=4972&highroad_approach_max_boost_repeats=1&highroad_escape_max_boost_repeats=unlimited";
        let searched = CrtSetup::parse(query).unwrap().calculate().unwrap();
        let selected = &searched["prediction"];
        let setup = &selected["crt_setup"];
        let boss = &selected["post_battle_field"]["pirro_battle"];
        assert_eq!(boss["state"]["outcome"], "victory");
        assert_eq!(boss["r8_plan"], json!([1, 0, 1, 2, 3]));
        assert_eq!(boss["r8_manipulation"]["inputs"]["extra_input_count"], 2);
        assert_eq!(
            selected["post_battle_field"]["highroad"]["approach_search"]["target_met"],
            true
        );
        let mut replay_setup = CrtSetup::parse(query).unwrap();
        replay_setup.boost_plan = BoostPlan {
            player_menu: setup["boost_menu"].as_u64().unwrap() as u32,
            advances: setup["boost_advances"].as_u64().unwrap() as u32,
            repeat_cap: setup["boost_repeat_cap"].as_u64().map(|n| n as u32),
        };
        replay_setup.second_plan = Some(BoostPlan {
            player_menu: setup["second_boost_menu"].as_u64().unwrap() as u32,
            advances: setup["second_boost_advances"].as_u64().unwrap() as u32,
            repeat_cap: setup["second_boost_repeat_cap"].as_u64().map(|n| n as u32),
        });
        replay_setup.pirro_plan =
            serde_json::from_value(setup["pirro_boost_inputs"].clone()).unwrap();
        replay_setup.pirro_r8_plan =
            serde_json::from_value(setup["pirro_r8_boost_inputs"].clone()).unwrap();
        let replay = replay_setup.simulate().unwrap();
        assert_eq!(
            replay["post_battle_field"]["pirro_battle"]["state"],
            boss["state"]
        );
        assert_eq!(
            replay["post_battle_field"]["highroad"]["escape"],
            selected["post_battle_field"]["highroad"]["escape"]
        );
        replay_setup.pirro_r8_plan = None;
        assert_eq!(
            replay["post_battle_field"]["flats"],
            selected["post_battle_field"]["flats"]
        );
        replay_setup.highroad_settings.approach_max_boost_repeats = Some(0);
        let baseline = replay_setup.simulate().unwrap();
        assert_eq!(baseline["state"], selected["state"]);
        assert_eq!(
            baseline["post_battle_field"]["second_battle"]["state"],
            selected["post_battle_field"]["second_battle"]["state"]
        );
        assert_eq!(
            baseline["post_battle_field"]["mansion_escape"],
            selected["post_battle_field"]["mansion_escape"]
        );
        assert_eq!(
            baseline["post_battle_field"]["pirro_battle"]["boost_search"]["plan"],
            boss["boost_search"]["plan"]
        );
        let prefix = |p: &Value| {
            p["steps"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|s| s["round"].as_u64().unwrap() < 8)
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            prefix(&baseline["post_battle_field"]["pirro_battle"]),
            prefix(boss)
        );
        assert_eq!(
            baseline["post_battle_field"]["highroad"]["encounter"]["situation"],
            "Surprise"
        );
        assert!(baseline["post_battle_field"]["highroad"]["escape"].is_null());
        assert!(baseline["post_battle_field"]["highroad"]["field_return"].is_null());
        assert!(baseline["post_battle_field"]["flats"].is_null());
        assert!(baseline["post_battle_field"]["harbor_return"].is_null());
        replay_setup.highroad_settings.approach_max_boost_repeats = None;
        let unlimited = replay_setup.simulate().unwrap();
        assert_eq!(
            unlimited["post_battle_field"]["pirro_battle"]["r8_plan"],
            boss["r8_plan"]
        );
        for invalid in [
            "pirro_r8_boost_inputs=1,3",
            "pirro_r8_boost_inputs=1,2",
            "highroad_approach_max_boost_repeats=257",
            "highroad_escape_max_boost_repeats=4&highroad_escape_max_boost_repeats=5",
            "highroad_flats_max_boost_repeats=257",
            "highroad_flats_max_boost_repeats=4&highroad_flats_max_boost_repeats=unlimited",
        ] {
            assert!(CrtSetup::parse(&format!("seed=0&{invalid}")).is_err());
        }
    }
    #[test]
    fn pirro_search_and_export_preserve_exact_inputs_and_upstream_route() {
        let query = "seed=0&advances=4942&pirro_max_boost_repeats=unlimited";
        let searched = super::CrtSetup::parse(query).unwrap().calculate().unwrap();
        let p = &searched["prediction"];
        let boss = &p["post_battle_field"]["pirro_battle"];
        assert_eq!(boss["boost_search"]["target_met"], true);
        assert_eq!(boss["routine"]["heal_before_round_eight"], false);
        assert_eq!(boss["state"]["outcome"], "victory");
        assert_eq!(
            p["crt_setup"]["pirro_boost_inputs"],
            boss["boost_search"]["plan"]
        );
        let mut setup = super::CrtSetup::parse(query).unwrap();
        setup.boost_plan = ot2_speedrun_assistant::manipulation::BoostPlan {
            advances: p["crt_setup"]["boost_advances"].as_u64().unwrap() as u32,
            player_menu: p["crt_setup"]["boost_menu"].as_u64().unwrap() as u32,
            repeat_cap: p["crt_setup"]["boost_repeat_cap"]
                .as_u64()
                .map(|n| n as u32),
        };
        setup.second_plan = Some(ot2_speedrun_assistant::manipulation::BoostPlan {
            advances: p["crt_setup"]["second_boost_advances"].as_u64().unwrap() as u32,
            player_menu: p["crt_setup"]["second_boost_menu"].as_u64().unwrap() as u32,
            repeat_cap: p["crt_setup"]["second_boost_repeat_cap"]
                .as_u64()
                .map(|n| n as u32),
        });
        setup.pirro_plan =
            serde_json::from_value(p["crt_setup"]["pirro_boost_inputs"].clone()).unwrap();
        let replay = setup.simulate().unwrap();
        assert_eq!(
            replay["post_battle_field"]["pirro_aftermath"],
            p["post_battle_field"]["pirro_aftermath"]
        );
        assert_eq!(
            replay["post_battle_field"]["pirro_battle"]["state"],
            boss["state"]
        );
        assert_eq!(
            replay["post_battle_field"]["pirro_battle"]["steps"],
            boss["steps"]
        );
        setup.pirro_plan = None;
        setup.pirro_settings.victory_max_boost_repeats = Some(0);
        setup.pirro_settings.no_grape_max_boost_repeats = Some(0);
        let baseline = setup.simulate().unwrap();
        assert_eq!(baseline["state"], p["state"]);
        assert_eq!(
            baseline["post_battle_field"]["second_battle"]["state"],
            p["post_battle_field"]["second_battle"]["state"]
        );
        assert_eq!(
            baseline["post_battle_field"]["mansion_escape"],
            p["post_battle_field"]["mansion_escape"]
        );
        let parsed = super::CrtSetup::parse(
            "seed=0&pirro_boost_inputs=1,2,3,2,3&pirro_max_boost_repeats=unlimited",
        )
        .unwrap();
        assert_eq!(
            parsed.json()["pirro_boost_inputs"],
            serde_json::json!([1, 2, 3, 2, 3])
        );
        assert!(parsed.json()["pirro_no_grape_max_boost_repeats"].is_null());
        assert_eq!(parsed.json()["pirro_victory_max_boost_repeats"], 4);
        for invalid in [
            "pirro_boost_inputs=1,3",
            "pirro_boost_inputs=1,2",
            "pirro_boost_inputs=",
            "pirro_max_boost_repeats=257",
            "pirro_max_boost_repeats=1&pirro_max_boost_repeats=2",
            "pirro_victory_max_boost_repeats=257",
            "pirro_no_grape_max_boost_repeats=257",
            "pirro_victory_max_boost_repeats=1&pirro_victory_max_boost_repeats=2",
            "pirro_no_grape_max_boost_repeats=1&pirro_max_boost_repeats=2",
        ] {
            assert!(super::CrtSetup::parse(&format!("seed=0&{invalid}")).is_err());
        }
    }
    #[test]
    fn mansion_route_uses_measurement_and_replays_escape_plan() {
        let p = super::CrtSetup::parse(
            "seed=0&advances=4942&mansion_escape_max_boost_repeats=unlimited",
        )
        .unwrap()
        .calculate()
        .unwrap();
        let m = &p["prediction"]["post_battle_field"]["mansion_escape"];
        assert!(m["error"].is_null(), "{m}");
        assert_eq!(m["decrement"], 12);
        assert_eq!(m["run_updates"], 161);
        assert_eq!(m["first_encounter"]["group"], "ENG_DNG_CTY_1_1_LND_DAY_110");
        assert_eq!(m["first_encounter"]["first_override"], true);
        assert_eq!(
            m["first_encounter_update"].as_u64().unwrap()
                + m["remaining_updates"].as_u64().unwrap(),
            161
        );
        assert_eq!(
            m["required_counter"].as_u64().unwrap(),
            m["remaining_updates"].as_u64().unwrap() * 12 + 1
        );
        assert_eq!(m["solution"]["attempt"]["success"], true, "{m}");
        assert!(m["solution"]["counter_at_trigger"].as_u64().unwrap() > 0);
        assert_eq!(m["solution"]["world_at_trigger"]["first_encounter"], false);
        let boss = &p["prediction"]["post_battle_field"]["pirro_battle"];
        assert!(boss["error"].is_null());
        assert_eq!(boss["entry"], m["solution"]["after_field_return"]);
        assert_eq!(
            boss["initial_state"]["actors"][0]["resources"]["hp"],
            m["solution"]["attempt"]["state"]["actors"][0]["resources"]["hp"]
        );
        assert_eq!(boss["state"]["outcome"], "victory");
        assert!(boss["routine_error"].is_null());
        assert_eq!(boss["after_victory"], boss["after"]);
        // The four stolen stones enter inventory and are each used once.
        assert_eq!(
            &boss["initial_state"]["inventory"].as_array().unwrap()[10..],
            &[
                serde_json::json!(1),
                serde_json::json!(1),
                serde_json::json!(1),
                serde_json::json!(1)
            ]
        );
        assert_eq!(
            &boss["state"]["inventory"].as_array().unwrap()[10..],
            &[
                serde_json::json!(0),
                serde_json::json!(0),
                serde_json::json!(0),
                serde_json::json!(0)
            ]
        );
        println!(
            "mansion prediction: {}",
            serde_json::json!({"entry":m["before_battle_initialization"],"menu":m["at_command_menu"],"after":m["solution"]["after_field_return"],"inputs":m["solution"]["inputs"]["levels"],"counter":m["solution"]["counter"],"remaining":m["remaining_updates"],"status":m["status"]})
        );
        let setup =
            super::CrtSetup::parse("seed=0&advances=4942&mansion_escape_max_boost_repeats=0")
                .unwrap();
        assert!(setup.json().get("mansion_run_updates").is_none());
        assert_eq!(setup.json()["mansion_escape_max_boost_repeats"], 0);
        for query in [
            "mansion_run_updates=161",
            "mansion_run_updates=162",
            "mansion_run_updates=0",
            "mansion_run_updates=1.5",
            "mansion_escape_max_boost_repeats=257",
            "mansion_run_updates=161&mansion_run_updates=162",
        ] {
            assert!(super::CrtSetup::parse(&format!("seed=0&{query}")).is_err());
        }
    }
    #[test]
    fn unlimited_event_settings_round_trip_through_search_and_export() {
        use super::*;
        let query = "seed=0&advances=4916&encounter_max_boost_repeats=1&second_fight_max_boost_repeats=unlimited&flats_snow_max_boost_repeats=0&flats_osvald_max_boost_repeats=0&mug_clerics_max_boost_repeats=0";
        let setup = CrtSetup::parse(query).unwrap();
        assert_eq!(
            setup.event_caps,
            EventRepeatCaps {
                encounter: Some(1),
                second_fight: None
            }
        );
        let result = setup.calculate().unwrap();
        let prediction = &result["prediction"];
        assert!(prediction["crt_setup"]["second_fight_max_boost_repeats"].is_null());
        assert!(prediction["crt_setup"]["boost_repeat_cap"].is_null());
        assert_eq!(prediction["required_outcomes"]["met"], true);
        assert_eq!(
            prediction["post_battle_field"]["second_battle"]["assessment"]["only_one_enemy_action"],
            true
        );
        assert_eq!(prediction["manipulation"]["inputs"]["input_count"], 14);
        let replay_query = format!(
            "{query}&boost_menu={}&boost_advances={}",
            prediction["crt_setup"]["boost_menu"], prediction["crt_setup"]["boost_advances"]
        );
        let address = "127.0.0.1:9999";
        let request = format!("GET /api/export?{replay_query} HTTP/1.1\r\nHost: {address}\r\n\r\n");
        let export = route(&request, address);
        assert_eq!(export.status, 200);
        assert_eq!(
            serde_json::from_str::<Value>(&export.body).unwrap(),
            *prediction
        );
        let both = CrtSetup::parse(
            "seed=0&encounter_max_boost_repeats=unlimited&second_fight_max_boost_repeats=unlimited",
        )
        .unwrap();
        assert_eq!(
            both.event_caps,
            EventRepeatCaps {
                encounter: None,
                second_fight: None
            }
        );
        for duplicate in [
            "encounter_max_boost_repeats=unlimited&encounter_max_boost_repeats=4",
            "second_fight_max_boost_repeats=unlimited&second_fight_max_boost_repeats=4",
            "max_boost_repeats=unlimited&second_fight_max_boost_repeats=4",
        ] {
            assert!(CrtSetup::parse(&format!("seed=0&{duplicate}")).is_err());
        }
        // Large explicit plans must remain replayable after an uncapped search.
        let large = CrtSetup::parse("seed=0&advances=4916&boost_menu=6&boost_advances=300")
            .unwrap()
            .simulate()
            .unwrap();
        assert_eq!(large["manipulation"]["plan"]["advances"], 300);
    }

    #[test]
    fn crackridge_route_settings_and_clean_escape_propagate_selected_market() {
        let setup=CrtSetup::parse("seed=0&advances=4916&encounter_max_boost_repeats=required&pirro_victory_max_boost_repeats=required&pirro_no_grape_max_boost_repeats=required&highroad_approach_max_boost_repeats=required&highroad_flats_max_boost_repeats=required&highroad_flats_approach_max_boost_repeats=required&flats_escape_max_boost_repeats=boost_only").unwrap();
        assert!(setup.crackridge_approach_max_boost_repeats.is_none());
        let p=setup.calculate().unwrap()["prediction"].clone();
        let f=&p["post_battle_field"];
        assert_eq!(f["black_market"]["clerics_present"],true);
        assert_eq!(f["crackridge_escape"]["attempt"]["success"],true);
        assert!(matches!(f["crackridge_escape"]["encounter"]["situation"].as_str(),Some("Normal"|"Preemptive")));
        assert_eq!(f["crackridge_voyage"]["arrival"],f["black_market"]["after_night"]);
        assert_eq!(f["crackridge_escape"]["menu"]["party_levels"],json!([5,1]));
        assert_eq!(f["crackridge_escape"]["field_return"]["world_after"]["party"],json!([7,6]));
        let capped=CrtSetup::parse("seed=0&crackridge_approach_max_boost_repeats=0").unwrap();
        assert_eq!(capped.crackridge_approach_max_boost_repeats,Some(0));
        assert_eq!(capped.json()["crackridge_approach_max_boost_repeats"],0);
        assert!(CrtSetup::parse("seed=0&crackridge_approach_max_boost_repeats=4&crackridge_approach_max_boost_repeats=5").is_err());
        println!("switch={} arrival={} menu={} boosts={} field={}",f["crackridge_voyage"]["day_switch_location"],f["crackridge_voyage"]["after"]["draws"],f["crackridge_escape"]["menu"]["rng"]["draws"],f["crackridge_escape"]["inputs"]["extra_repeats"],f["crackridge_escape"]["field_return"]["after"]["draws"]);
    }
    #[test]
    fn required_policy_migrates_legacy_settings_and_reports_unreachable_targets() {
        use super::*;
        let query = "seed=0&advances=4942&boost_menu=6&boost_advances=1&highroad_flats_max_boost_repeats=required&highroad_flats_approach_max_boost_repeats=required&flats_snow_max_boost_repeats=0&flats_osvald_max_boost_repeats=0&mug_clerics_max_boost_repeats=0";
        let setup = CrtSetup::parse(query).unwrap();
        let legacy = CrtSetup::parse(&query.replace("required", "unlimited")).unwrap();
        assert_eq!(setup.json(), legacy.json());
        assert_eq!(
            setup.json()["search_policies"]["harbor_walk"]["mode"],
            "required"
        );
        assert_eq!(
            setup.json()["search_policies"]["pirro_victory"]["mode"],
            "optional"
        );
        let mut result = setup.simulate().unwrap();
        assert_eq!(result["required_outcomes"]["met"], true);
        // A required downstream event that wasn't reached cannot count as success.
        result["post_battle_field"]["flats_escape"] = Value::Null;
        assert_eq!(
            setup.check_requirements(&result)["unmet"],
            json!({
                "met":false, "unmet":[{"id":"flats_approach","label":"Avoid surprise after Harbor return"}]
            })["unmet"]
        );
        let missing = setup.check_requirements(&json!({}));
        assert_eq!(missing["met"], false);
        assert_eq!(missing["unmet"].as_array().unwrap().len(), 4);
        let strategy = CrtSetup::parse("seed=0&highroad_escape_max_boost_repeats=boost_only&flats_escape_max_boost_repeats=boost_only").unwrap();
        assert_eq!(strategy.highroad_settings.escape_max_boost_repeats, None);
        assert_eq!(strategy.flats_escape_settings.max_boost_repeats, None);
        assert!(CrtSetup::parse("seed=0&highroad_flats_max_boost_repeats=boost_only").is_err());
        let all_required = [
            "encounter_max_boost_repeats",
            "second_fight_max_boost_repeats",
            "mansion_approach_max_boost_repeats",
            "mansion_escape_max_boost_repeats",
            "pirro_victory_max_boost_repeats",
            "pirro_no_grape_max_boost_repeats",
            "highroad_approach_max_boost_repeats",
            "highroad_flats_max_boost_repeats",
            "highroad_flats_approach_max_boost_repeats",
        ]
        .map(|key| format!("{key}=required"))
        .join("&");
        let all = CrtSetup::parse(&format!("seed=0&advances=4942&{all_required}"))
            .unwrap()
            .calculate()
            .unwrap();
        assert_eq!(
            all["prediction"]["required_outcomes"]["met"], true,
            "{}",
            all["prediction"]["required_outcomes"]
        );
    }

    #[test]
    fn repeat_settings_validate_default_fallback_and_replay_path() {
        use super::*;
        let setup = CrtSetup::parse("seed=0&advances=4916").unwrap();
        assert_eq!(setup.event_caps, EventRepeatCaps::default());
        assert_eq!(
            setup.calculate().unwrap()["search"]["second_fight_limit_reached"],
            false
        );
        let value = CrtSetup::parse("seed=0&advances=4916&second_fight_max_boost_repeats=2")
            .unwrap()
            .calculate()
            .unwrap();
        assert_eq!(value["search"]["second_fight_limit_reached"], true);
        assert_eq!(value["search"]["target"], "safe_checkpoint");
        assert_eq!(value["prediction"]["crt_setup"]["boost_advances"], 3);
        assert_eq!(
            value["prediction"]["post_battle_field"]["reset_reach"]["walking"]["reaches_reset"],
            true
        );
        assert_eq!(
            value["prediction"]["post_battle_field"]["second_battle"]["assessment"]["only_one_enemy_action"],
            false
        );
        let constrained = CrtSetup::parse(
            "seed=0&advances=4916&boost_menu=6&boost_advances=12&boost_repeat_cap=6",
        )
        .unwrap()
        .simulate()
        .unwrap();
        assert_eq!(constrained["manipulation"]["inputs"]["extra_repeats"], 6);
        assert_eq!(constrained["manipulation"]["inputs"]["input_count"], 14);
        assert_eq!(constrained["crt_setup"]["boost_repeat_cap"], 6);
        let joint = CrtSetup::parse(
            "seed=0&advances=4916&encounter_max_boost_repeats=2&second_fight_max_boost_repeats=4",
        )
        .unwrap()
        .calculate()
        .unwrap();
        assert_eq!(
            joint["prediction"]["crt_setup"]["encounter_max_boost_repeats"],
            2
        );
        assert_eq!(
            joint["prediction"]["crt_setup"]["second_fight_max_boost_repeats"],
            4
        );
        assert_eq!(joint["search"]["second_fight_extra_repeats"], 4);
        assert_eq!(joint["search"]["encounter_dodge_repeats"], 2);
        assert_eq!(
            joint["prediction"]["manipulation"]["inputs"],
            joint["search"]["solution"]["inputs"]
        );
        assert_eq!(joint["prediction"]["crt_setup"]["boost_repeat_cap"], 6);
        assert_eq!(
            joint["prediction"]["post_battle_field"]["second_battle"]["assessment"]["only_one_enemy_action"],
            true
        );
        for parameter in [
            "encounter_max_boost_repeats",
            "second_fight_max_boost_repeats",
            "boost_repeat_cap",
        ] {
            for value in ["-1", "4294967296", "4.5", "+4", "", "true"] {
                assert!(CrtSetup::parse(&format!("seed=0&{parameter}={value}")).is_err());
            }
            assert!(CrtSetup::parse(&format!("seed=0&{parameter}=4&{parameter}=5")).is_err());
        }
        assert!(CrtSetup::parse("seed=0&encounter_max_boost_repeats=257").is_err());
        assert!(CrtSetup::parse("seed=0&second_fight_max_boost_repeats=257").is_err());
        let legacy = CrtSetup::parse("seed=0&max_boost_repeats=9").unwrap();
        assert_eq!(
            legacy.event_caps,
            EventRepeatCaps {
                encounter: Some(4),
                second_fight: Some(9)
            }
        );
        assert!(
            CrtSetup::parse("seed=0&max_boost_repeats=4&second_fight_max_boost_repeats=5").is_err()
        );
        let blocked = CrtSetup::parse(
            "seed=0&advances=4916&encounter_max_boost_repeats=0&second_fight_max_boost_repeats=14&mug_clerics_max_boost_repeats=0&flats_osvald_max_boost_repeats=0&flats_snow_max_boost_repeats=0",
        )
        .unwrap()
        .calculate()
        .unwrap();
        assert_eq!(blocked["search"]["encounter_limit_reached"], true);
        assert_eq!(blocked["search"]["event_caps"]["encounter"], 0);
        assert_eq!(blocked["search"]["event_caps"]["second_fight"], 14);
        assert!(blocked["prediction"].is_null());
    }

    #[test]
    fn second_fight_carries_route_progress_and_crt_without_double_counting() {
        use super::*;
        // Exact first-battle input adjustment used in the 2026-09-27 capture.
        let prediction = CrtSetup::parse("seed=0&advances=4942&boost_menu=6&boost_advances=1")
            .unwrap()
            .simulate()
            .unwrap();
        let field = &prediction["post_battle_field"];
        let second = &field["second_battle"];
        assert_eq!(second["arrival"], field["ambush"]["after_rejoin"]);
        assert_eq!(second["opening_exp"], 6);
        assert_eq!(second["party_level"], 2);
        assert_eq!(second["state"]["outcome"], "victory");
        assert!(second["routine_error"].is_null());
        assert_eq!(second["assessment"]["only_one_enemy_action"], true);
        assert_eq!(second["assessment"]["enemy_actions"], 1);
        assert_eq!(second["assessment"]["damage_hits_received"], 1);
        assert_eq!(
            second["assessment"]["order_conditions"],
            serde_json::json!({
                "throne_before_right_round_one": true,
                "scaracci_before_left_round_one": true,
                "throne_before_leader_round_two": true,
            })
        );
        let mut rng: CrtRng = serde_json::from_value(second["arrival"].clone()).unwrap();
        assert_eq!(serde_json::to_value(rng).unwrap(), second["entry"]);
        assert_eq!(
            second["transition_trace"][0]["before"],
            second["transition_trace"][0]["after"]
        );
        assert_eq!(second["entry"]["state"], 0x73B6B400u32);
        assert_eq!(
            prediction["crt_setup"]["battle_entry_advances"]
                .as_str()
                .unwrap()
                .parse::<u64>()
                .unwrap()
                + rng.draws,
            5120
        );
        for (i, actor) in second["actors"]
            .as_array()
            .unwrap()
            .iter()
            .take(3)
            .enumerate()
        {
            let resources = &second["initial_state"]["actors"][i]["resources"];
            assert_eq!(resources["hp"], actor["max_hp"]);
            assert_eq!(resources["sp"], actor["max_sp"]);
        }
        let mut local = CrtRng::new(rng.state);
        for span in second["state"]["opening_crt"]["trace"].as_array().unwrap() {
            assert_eq!(serde_json::to_value(local).unwrap(), span["before"]);
            local.advance(span["after"]["draws"].as_u64().unwrap() - local.draws);
            assert_eq!(serde_json::to_value(local).unwrap(), span["after"]);
        }
        assert_eq!(serde_json::to_value(local).unwrap(), second["state"]["rng"]);
        rng.advance(local.draws);
        assert_eq!(serde_json::to_value(rng).unwrap(), second["after"]);
        assert_eq!(second["after"]["state"], 0x41C7DB10u32);
        // The city is a separate route stage. Its origin must be the route's
        // after-drops state, not the battle-local draw count or entry again.
        assert_eq!(second["city"]["arrival"], second["after"]);
        rng.advance(110);
        assert_eq!(
            serde_json::to_value(rng).unwrap(),
            second["city"]["after_popup"]
        );
        assert_eq!(second["city"]["map"], "Twn_Cty_1_1_B");
        assert_eq!(second["city"]["tutorial"], "eRADAR_MAP");
        assert_eq!(second["city"]["after_popup"]["state"], 0x6B5BBFD6u32);
        assert_eq!(rng.draws + 4954, 5374);
        assert_eq!(second["parlor"]["arrival"], second["city"]["after_popup"]);
        rng.advance(50);
        assert_eq!(
            serde_json::to_value(rng).unwrap(),
            second["parlor"]["after_return"]
        );
        assert_eq!(second["parlor"]["after_return"]["state"], 0x080E80B0u32);
        assert_eq!(rng.draws + 4954, 5424);
        assert_eq!(second["parlor"]["world_after"]["time"], 2);
        assert_eq!(second["parlor"]["map"], "Twn_Cty_1_1_B");
        assert_eq!(
            second["parlor_revisit"]["arrival"],
            second["parlor"]["after_return"]
        );
        rng.advance(43);
        assert_eq!(
            serde_json::to_value(rng).unwrap(),
            second["parlor_revisit"]["after_return"]
        );
        assert_eq!(
            second["parlor_revisit"]["after_return"]["state"],
            0xB8E6F54Du32
        );
        assert_eq!(rng.draws + 4954, 5467);
        assert_eq!(
            second["parlor_revisit"]["after_popup"],
            second["parlor_revisit"]["after_return"]
        );
        assert_eq!(
            second["mansion"]["arrival"],
            second["parlor_revisit"]["after_popup"]
        );
        rng.advance(95);
        assert_eq!(
            second["mansion"]["after_entry"],
            serde_json::to_value(rng).unwrap()
        );
        assert_eq!(second["mansion"]["after_entry"]["state"], 0xDDCC3032u32);
        assert_eq!(rng.draws + 4954, 5562);
        assert_eq!(second["mansion"]["world_after"]["map"], "Dng_Cty_1_1");
        let ambush = &second["mansion"]["lookout_ambush"];
        assert_eq!(ambush["before_menu"], second["mansion"]["after_entry"]);
        rng.advance(2);
        assert_eq!(ambush["after"], serde_json::to_value(rng).unwrap());
        assert_eq!(rng.draws + 4954, 5564);
        assert_eq!(ambush["after"]["state"], 0x5121903Cu32);
        assert_eq!(ambush["counter_reset"], false);
        assert_eq!(
            ambush["world_after"]["encounter_counter"],
            second["mansion"]["world_after"]["encounter_counter"]
        );
        let unsafe_route = CrtSetup::parse("seed=0&advances=4916")
            .unwrap()
            .simulate()
            .unwrap();
        assert!(unsafe_route["post_battle_field"]["second_battle"].is_null());
    }
    #[test]
    fn boost_search_replays_exports_and_keeps_menu_checkpoint_before_inputs() {
        use super::*;
        let address = "127.0.0.1:54345";
        let request = |path: &str| format!("GET {path} HTTP/1.1\r\nHost: {address}\r\n\r\n");
        let response = route(
            &request(
                "/api/boost-search?seed=0&advances=4916&encounter_max_boost_repeats=2&second_fight_max_boost_repeats=14",
            ),
            address,
        );
        assert_eq!(response.status, 200);
        let value: Value = serde_json::from_str(&response.body).unwrap();
        let prediction = &value["prediction"];
        let plan = &value["search"]["solution"]["plan"];
        assert_eq!(
            value["search"]["target"],
            "safe_checkpoint_and_one_enemy_action"
        );
        let menu = plan["player_menu"].as_u64().unwrap();
        let cycles = plan["advances"].as_u64().unwrap();
        assert!(cycles > 0);
        assert_eq!(
            prediction["state"]["rng"],
            value["search"]["solution"]["final_rng"]
        );
        assert_eq!(
            prediction["manipulation"]["rng_after_inputs"]["draws"]
                .as_u64()
                .unwrap()
                - prediction["manipulation"]["rng_before"]["draws"]
                    .as_u64()
                    .unwrap(),
            cycles + {
                let bp = prediction["manipulation"]["inputs"]["normal_bp"]
                    .as_u64()
                    .unwrap();
                bp * (bp + 1) / 2
            }
        );
        let player_steps: Vec<_> = prediction["steps"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|s| s["report"]["actor"].as_u64().unwrap() < 3)
            .collect();
        assert_eq!(
            player_steps[menu as usize - 1]["report"]["rng_before"],
            prediction["manipulation"]["rng_before"]
        );
        assert_eq!(
            prediction["post_battle_field"]["reset_reach"]["walking"]["reaches_reset"],
            true
        );
        let repeat_cap = plan["repeat_cap"].as_u64().unwrap();
        let query = format!(
            "seed=0&advances=4916&boost_menu={menu}&boost_advances={cycles}&encounter_max_boost_repeats=2&second_fight_max_boost_repeats=14&boost_repeat_cap={repeat_cap}"
        );
        let second = &prediction["post_battle_field"]["second_battle"];
        let searched_second = &value["search"]["solution"]["second_fight"];
        assert_eq!(second["state"]["outcome"], "victory");
        assert!(second["routine_error"].is_null());
        assert_eq!(second["assessment"]["only_one_enemy_action"], true);
        assert_eq!(second["assessment"]["order_conditions_met"], true);
        for field in ["entry", "after", "party_level", "assessment"] {
            assert_eq!(second[field], searched_second[field], "{field}");
        }
        let export = route(&request(&format!("/api/export?{query}")), address);
        assert_eq!(export.status, 200);
        assert_eq!(
            serde_json::from_str::<Value>(&export.body).unwrap(),
            *prediction
        );
        assert!(
            export
                .filename
                .unwrap()
                .contains(&format!("-boost-menu-{menu}-advances-{cycles}"))
        );
        let response = route(&request(&format!("/api/simulate?{query}")), address);
        assert_eq!(
            serde_json::from_str::<Value>(&response.body).unwrap(),
            *prediction
        );
        for query in [
            "seed=0&boost_menu=0&boost_advances=1",
            "seed=0&boost_menu=7&boost_advances=1",
            "seed=0&boost_menu=1&boost_advances=4294967296",
            "seed=0&boost_menu=1&boost_menu=2&boost_advances=1",
        ] {
            assert_eq!(
                route(&request(&format!("/api/boost-search?{query}")), address).status,
                400
            );
        }
    }
    #[test]
    fn no_enemy_turn_prediction_checks_actual_break_target() {
        // Two leading players, one leading player breaking the next enemy,
        // and one leading player hitting the wrong enemy for that order.
        for (seed, order, enemy_turns) in [
            (0, vec![0, 1, 3, 2, 4], 0),
            (1, vec![1, 3, 0, 2, 4], 0),
            (25, vec![0, 4, 1, 3, 2], 1),
        ] {
            let r = super::simulate(seed).unwrap();
            assert_eq!(r["initial_state"]["queue"], serde_json::json!(order));
            assert_eq!(r["battle_assessment"]["no_enemy_actions"], enemy_turns == 0);
            assert_eq!(
                r["battle_assessment"]["enemy_turns"]
                    .as_array()
                    .unwrap()
                    .len(),
                enemy_turns
            );
            assert_eq!(r["state"]["outcome"], "victory");
        }
    }

    #[test]
    fn reaching_cutscene_does_not_consume_hypothetical_encounter_rng() {
        let battle = super::CrtSetup::parse("seed=0&advances=4917")
            .unwrap()
            .simulate()
            .unwrap();
        let frame = &battle["post_battle_field"];
        assert_eq!(frame["checkpoint"]["arrival"], battle["state"]["rng"]);
        assert_eq!(
            frame["checkpoint"]["after_popup"]["draws"]
                .as_u64()
                .unwrap(),
            battle["state"]["rng"]["draws"].as_u64().unwrap() + 4
        );
        assert_eq!(frame["options"]["current_counter"], 831);
        let ambush = &frame["ambush"];
        assert_eq!(ambush["before_menu"], frame["checkpoint"]["after_popup"]);
        let before = ambush["before_menu"]["draws"].as_u64().unwrap();
        for (stage, extra) in [("at_menu", 1), ("after_ambush", 3), ("after_rejoin", 5)] {
            assert_eq!(ambush[stage]["draws"].as_u64().unwrap(), before + extra);
        }
        assert_eq!(ambush["encounters_enabled"], true);
        assert_eq!(ambush["known_flags"]["25304"], false);
        assert_eq!(frame["reset_reach"]["recommendation"], "walk");
        let mut request = serde_json::json!({"seed":frame["seed"],"advances":frame["advances"],"options":frame["options"],"profile":frame["profile"]["id"]});
        let p = super::encounter_prediction(&request.to_string()).unwrap();
        assert_eq!(p["kind"], "reset_reached");
        assert_eq!(p["counter_before_reset"], 147);
        assert_eq!(p["draws"], "0");
        assert_eq!(p["final_state"], frame["starting_state"]);
        assert_eq!(p["total_advances"], frame["advances"]);
        request["options"]["intervening_advances"] = serde_json::json!("7");
        let gap = super::encounter_prediction(&request.to_string()).unwrap();
        let mut rng = super::CrtRng::new(frame["starting_state"].as_u64().unwrap() as u32);
        rng.advance(7);
        assert_eq!(gap["final_state"], rng.state);
        assert_eq!(gap["draws"], "7");
        request["options"]["intervening_advances"] = serde_json::json!("0");
        request["options"]["dash"] = serde_json::json!(true);
        let run = super::encounter_prediction(&request.to_string()).unwrap();
        assert_eq!(run["kind"], "encounter_prediction");
        assert_eq!(run["draws"], "6");
        assert_eq!(run["counter_updates"], 52);
    }

    #[test]
    fn battle_frame_feeds_field_api_without_an_extra_reset() {
        let result = super::CrtSetup::parse("seed=0&advances=4918")
            .unwrap()
            .simulate()
            .unwrap();
        let frame = &result["post_battle_field"];
        assert_eq!(frame["seed"], 0);
        assert_eq!(frame["advances"], "5083");
        assert_eq!(frame["starting_state"], result["state"]["rng"]["state"]);
        assert_eq!(frame["options"]["current_counter"], 549);
        assert!(frame["checkpoint"].is_null()); // Cannot reach it before an encounter.
        assert!(frame["ambush"].is_null());
        let mut request = serde_json::json!({"seed":frame["seed"],"advances":frame["advances"],"options":frame["options"],"profile":frame["profile"]["id"]});
        let p = super::encounter_prediction(&request.to_string()).unwrap();
        assert_eq!(p["total_advances"], "5089");
        assert_eq!(p["checkpoints"][0]["total_advances"], "5083");
        assert_eq!(p["checkpoints"][0]["state"], frame["starting_state"]);
        assert_eq!(p["counter"]["supplied"], true);
        assert_eq!(p["profile"]["event"], "MS_TOU_10_0300");
        request["options"]["progress"] = serde_json::json!(1);
        assert!(super::encounter_prediction(&request.to_string()).is_err());
        request["profile"] = serde_json::Value::Null;
        assert!(super::encounter_prediction(&request.to_string()).is_ok());
    }
    #[test]
    fn encounter_routes_preserve_large_seed_positions_and_reject_bad_requests() {
        use serde_json::json;
        let address = "127.0.0.1:54345";
        let request =
            json!({"seed":0,"advances":"9007199254740993","options":{"boundary":"reset"}})
                .to_string();
        let encoded = request
            .bytes()
            .map(|b| format!("%{b:02X}"))
            .collect::<String>();
        let response = super::route(
            &format!(
                "GET /api/encounters/predict?request={encoded} HTTP/1.1\r\nHost: {address}\r\n\r\n"
            ),
            address,
        );
        assert_eq!(response.status, 200);
        let result: serde_json::Value = serde_json::from_str(&response.body).unwrap();
        assert_eq!(result["total_advances"], "9007199254740995");
        assert_eq!(
            result["checkpoints"][0]["total_advances"],
            "9007199254740993"
        );
        assert!(super::encounter_prediction("%ZZ").is_err());
        assert!(super::encounter_prediction("%7B%7D").is_err());
        let response = super::route(
            &format!(
                "GET /api/encounters/catalog HTTP/1.1\r\nHost: {address}\r\nOrigin: https://example.com\r\n\r\n"
            ),
            address,
        );
        assert_eq!(response.status, 403);
    }
    use super::*;

    #[test]
    fn seed_boundaries_and_invalid_inputs() {
        for (text, expected) in [
            ("0", 0),
            ("123", 123),
            ("0x7b", 123),
            ("0XFFFFFFFF", u32::MAX),
            ("4294967295", u32::MAX),
        ] {
            assert_eq!(parse_seed(text).unwrap(), expected);
        }
        for invalid in [
            "",
            "0x",
            "-1",
            "+1",
            "1.2",
            "1e3",
            "4294967296",
            "0x100000000",
            "123&x=1",
        ] {
            assert!(parse_seed(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn response_preserves_core_checkpoints_and_per_action_resources() {
        let value = simulate(123).unwrap();
        assert_eq!(
            value["state"]["rng"],
            json!({"state":3217448444u32,"draws":163})
        );
        assert_eq!(
            value["state"]["opening_crt"]["after_battle"]["encounter_steps"],
            639
        );
        let steps = value["steps"].as_array().unwrap();
        assert_eq!(steps.len(), 7);
        assert_eq!(steps[0]["round"], 1);
        assert_eq!(steps[6]["round"], 2);
        assert_eq!(steps[6]["actors_after"], value["state"]["actors"]);
        assert_eq!(value["state"]["outcome"], "victory");
        assert_eq!(value["routine_id"], "first_actor_boosts_enemy_one");
        assert_eq!(value["actors"][3]["name"], "Enemy 1 (right) · Pursuer A");
        assert_eq!(value["actors"][4]["name"], "Enemy 2 (left) · Pursuer B");
    }

    #[test]
    fn title_advances_include_transition_without_changing_battle_draw_counts() {
        let setup = CrtSetup::parse("seed=0x7b&advances=2").unwrap();
        assert_eq!(setup.battle_seed(), 2_400_679_945);
        let configured = setup.simulate().unwrap();
        let direct = simulate(2_400_679_945).unwrap();
        assert_eq!(configured["state"], direct["state"]);
        assert_eq!(configured["steps"], direct["steps"]);
        assert_eq!(configured["initial_seed"], 2_400_679_945u32);
        let s = &configured["crt_setup"];
        assert_eq!(s["initial_seed"], 123);
        assert_eq!(s["advances"], "2");
        assert_eq!(s["input_boundary"], "title_screen");
        assert_eq!(s["title_state"], 3_396_176_589u32);
        assert_eq!(s["transition_draws"], 12);
        assert_eq!(s["battle_entry_advances"], "14");
        assert_eq!(s["transition_checkpoints"][0]["total_advances"], "2");
        assert_eq!(s["transition_checkpoints"][7]["total_advances"], "14");
        let at_zero = CrtSetup::parse("seed=123&advances=0").unwrap();
        let mut rng = CrtRng::new(123);
        rng.advance(12);
        assert_eq!(
            at_zero.simulate().unwrap()["state"],
            simulate(rng.state).unwrap()["state"]
        );
    }

    #[test]
    fn shared_level_recreation_adds_one_prebattle_draw_and_preserves_export() {
        let address = "127.0.0.1:12345";
        let query = "seed=0&advances=4916&recreate_shared_level=true";
        let request = |endpoint: &str| {
            format!("GET /api/{endpoint}?{query} HTTP/1.1\r\nHost: {address}\r\n\r\n")
        };
        let preview = route(&request("crt"), address);
        assert_eq!(preview.status, 200);
        let mut setup: Value = serde_json::from_str(&preview.body).unwrap();
        assert_eq!(setup["battle_seed"], 0x0EA7_9703u32);
        assert_eq!(setup["advances"], "4916");
        assert_eq!(setup["title_state"], 0x137EA674u32);
        assert_eq!(setup["transition_draws"], 13);
        assert_eq!(setup["battle_entry_advances"], "4929");
        assert_eq!(setup["shared_level_draws"], 1);
        let response = route(&request("simulate"), address);
        assert_eq!(response.status, 200);
        let value: Value = serde_json::from_str(&response.body).unwrap();
        let direct = CrtSetup::parse("seed=0&advances=4917")
            .unwrap()
            .simulate()
            .unwrap();
        // The preview has no battle plan yet; simulation adds its exact replay inputs.
        assert!(setup["pirro_boost_inputs"].is_null());
        assert_eq!(
            value["crt_setup"]["pirro_boost_inputs"],
            value["post_battle_field"]["pirro_battle"]["boost_search"]["plan"]
        );
        setup["pirro_boost_inputs"] = value["crt_setup"]["pirro_boost_inputs"].clone();
        setup["pirro_r8_boost_inputs"] = value["crt_setup"]["pirro_r8_boost_inputs"].clone();
        assert_eq!(value["crt_setup"], setup);
        assert_eq!(value["state"], direct["state"]);
        assert_eq!(value["steps"], direct["steps"]);
        assert_eq!(value["initial_state"]["rng"]["state"], 0xF485D421u32);
        assert_eq!(value["initial_state"]["rng"]["draws"], 30);
        assert_eq!(
            value["state"]["rng"],
            json!({"state":2079755834u32,"draws":129})
        );
        let export = route(&request("export"), address);
        assert_eq!(export.status, 200);
        assert_eq!(
            export.filename.as_deref(),
            Some("ot2-throne-seed-0-title-advances-4916-recreated.json")
        );
        assert_eq!(serde_json::from_str::<Value>(&export.body).unwrap(), value);
        let retained = CrtSetup::parse("seed=0&advances=4916&recreate_shared_level=false").unwrap();
        assert_eq!(
            retained.battle_seed(),
            CrtSetup::parse("seed=0&advances=4916")
                .unwrap()
                .battle_seed()
        );
        assert_eq!(retained.battle_entry_advances(), 4928);
        assert_ne!(
            retained.battle_seed(),
            setup["battle_seed"].as_u64().unwrap() as u32
        );
    }

    #[test]
    fn advance_input_preview_and_export_preserve_configuration() {
        for invalid in [
            "seed=123&advances=-1",
            "seed=123&advances=1.5",
            "seed=123&advances=",
            "seed=123&advances=18446744073709551616",
            "seed=1&seed=2",
            "seed=1&advances=0&advances=1",
            "seed=1&recreate_shared_level=1",
            "seed=1&recreate_shared_level=true&recreate_shared_level=false",
            "seed=1&advances=18446744073709551615&recreate_shared_level=true",
            "seed=1&advances=18446744073709551604",
            "seed=1&advances=18446744073709551603&recreate_shared_level=true",
        ] {
            assert!(CrtSetup::parse(invalid).is_err(), "{invalid}");
        }
        let largest = CrtSetup::parse("seed=123&advances=18446744073709551603").unwrap();
        assert_eq!(largest.json()["advances"], "18446744073709551603");
        let last =
            CrtSetup::parse("seed=123&advances=18446744073709551602&recreate_shared_level=true")
                .unwrap();
        assert_eq!(last.battle_seed(), largest.battle_seed());
        assert_eq!(last.json()["battle_entry_advances"], "18446744073709551615");
        let address = "127.0.0.1:12345";
        let request = |path: &str| format!("GET {path} HTTP/1.1\r\nHost: {address}\r\n\r\n");
        let preview = route(&request("/api/crt?seed=123&advances=2"), address);
        assert_eq!(preview.status, 200);
        assert_eq!(
            serde_json::from_str::<Value>(&preview.body).unwrap()["battle_seed"],
            2_400_679_945u32
        );
        let export = route(&request("/api/export?seed=123&advances=2"), address);
        assert_eq!(
            export.filename.as_deref(),
            Some("ot2-throne-seed-123-title-advances-2.json")
        );
        let value = serde_json::from_str::<Value>(&export.body).unwrap();
        assert_eq!(value["crt_setup"]["initial_seed"], 123);
        assert_eq!(value["crt_setup"]["advances"], "2");
        assert_eq!(value["initial_seed"], 2_400_679_945u32);
    }

    #[test]
    fn local_routes_validate_seeds_and_reject_cross_origin_access() {
        let address = "127.0.0.1:12345";
        let request = |path: &str| format!("GET {path} HTTP/1.1\r\nHost: {address}\r\n\r\n");
        assert_eq!(
            route(&request("/api/simulate?seed=0x7b"), address).status,
            200
        );
        assert_eq!(
            route(&request("/api/simulate?seed=4294967296"), address).status,
            400
        );
        assert_eq!(route(&request("/not-found"), address).status, 404);
        assert_eq!(
            route("GET / HTTP/1.1\r\nHost: example.com\r\n\r\n", address).status,
            403
        );
        let foreign = format!(
            "GET /api/health HTTP/1.1\r\nHost: {address}\r\nOrigin: https://example.com\r\n\r\n"
        );
        assert_eq!(route(&foreign, address).status, 403);
        assert!(!route(&request("/api/shutdown"), address).stop);
        let export = route(&request("/api/export?seed=0x7b"), address);
        assert_eq!(
            export.filename.as_deref(),
            Some("ot2-throne-seed-123-title-advances-0.json")
        );
        let mut rng = CrtRng::new(123);
        rng.advance(12);
        assert_eq!(
            serde_json::from_str::<Value>(&export.body).unwrap()["initial_seed"],
            rng.state
        );
    }
}
