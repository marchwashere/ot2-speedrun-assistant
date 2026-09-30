//! Route adapter for the second Chapter 1 sewer battle. Combat stays in the
//! standalone battle crate; this module supplies progression and CRT boundaries.
use crate::manipulation::{BoostInputs, BoostPlan};
use crate::{CrtRng, Result, ensure};
use ot2_battle_engine::{
    ActorId, Battle, BattleConfig, BattleState, Event, ItemCatalog, Outcome, Side, StepReport,
    content::Resource, opening_crt::RngSpan, routine::ThroneSecondRoutine,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Manipulation {
    pub plan: BoostPlan,
    pub inputs: BoostInputs,
    pub actor: ActorId,
    pub round: u32,
    pub round_menu: u32,
    pub rng_before: CrtRng,
    pub rng_after_inputs: CrtRng,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct OrderConditions {
    pub throne_before_right_round_one: bool,
    pub scaracci_before_left_round_one: bool,
    pub throne_before_leader_round_two: bool,
}
impl OrderConditions {
    pub fn all_met(self) -> bool {
        self.throne_before_right_round_one
            && self.scaracci_before_left_round_one
            && self.throne_before_leader_round_two
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Assessment {
    pub order_conditions: OrderConditions,
    pub order_conditions_met: bool,
    pub enemy_actions: u32,
    pub damage_hits_received: u32,
    pub damage_received: u32,
    pub only_one_enemy_action: bool,
}
impl Assessment {
    /// The fixed routine never defends or changes speed. Round two therefore
    /// uses the cached next queue, with broken/dead actors removed on execution.
    /// Compare the requested pairs before removals; count actual actions too.
    pub fn new(battle: &Battle) -> Result<Self> {
        ensure(
            battle.state().round == 1,
            "assess second-fight order at initialization",
        )?;
        let id = |name| {
            battle
                .actor_id(name)
                .ok_or_else(|| crate::SimError(format!("second-fight actor missing: {name}")))
        };
        let throne = id("throne")?;
        let scaracci = id("scaracci")?;
        let right = id("grunt_right")?;
        let left = id("grunt_left")?;
        let leader = id("leader")?;
        let before = |order: &[ActorId], player, enemy| -> Result<bool> {
            let p = order.iter().position(|a| *a == player);
            let e = order.iter().position(|a| *a == enemy);
            ensure(
                p.is_some() && e.is_some(),
                "second-fight order is incomplete",
            )?;
            Ok(p.unwrap() < e.unwrap())
        };
        let first = battle.state().turn_order();
        let second = &battle.state().next_queue;
        let order_conditions = OrderConditions {
            throne_before_right_round_one: before(first, throne, right)?,
            scaracci_before_left_round_one: before(first, scaracci, left)?,
            throne_before_leader_round_two: before(second, throne, leader)?,
        };
        Ok(Self {
            order_conditions,
            order_conditions_met: order_conditions.all_met(),
            enemy_actions: 0,
            damage_hits_received: 0,
            damage_received: 0,
            only_one_enemy_action: false,
        })
    }

    pub fn record(&mut self, battle: &Battle, report: &StepReport) {
        // A missed attack still costs an enemy action. Double Thrust can land
        // two hits in one action, so neither count substitutes for the other.
        if battle.side(report.actor) == Some(Side::Enemy) {
            self.enemy_actions += 1;
        }
        for event in &report.events {
            if let Event::Damage { target, amount, .. } = *event {
                if battle.side(target) == Some(Side::Player) && amount > 0 {
                    self.damage_hits_received += 1;
                    self.damage_received += amount;
                }
            }
        }
    }

    pub fn finish(&mut self, outcome: Outcome) {
        self.only_one_enemy_action = outcome == Outcome::Victory && self.enemy_actions == 1;
    }
}

#[derive(Debug, Serialize)]
pub struct Actor {
    pub name: String,
    pub side: Side,
    pub max_hp: u32,
    pub max_sp: u32,
}
#[derive(Debug, Serialize)]
pub struct Step {
    pub round: u32,
    pub report: StepReport,
    pub actors_after: Vec<ot2_battle_engine::Combatant>,
}
#[derive(Debug, Serialize)]
pub struct Prediction {
    pub manipulation: Option<Manipulation>,
    pub mansion_approach_search: Option<crate::second_manipulation::Search>,
    pub group: &'static str,
    pub routine_id: &'static str,
    pub arrival: CrtRng,
    pub entry: CrtRng,
    pub after: CrtRng,
    pub city: Option<crate::city::Prediction>,
    pub parlor: Option<crate::parlor::Prediction>,
    pub parlor_revisit: Option<crate::parlor_revisit::Prediction>,
    pub mansion: Option<crate::mansion::Prediction>,
    pub opening_exp: u32,
    pub party_level: u8,
    pub actors: Vec<Actor>,
    pub initial_state: BattleState,
    pub state: BattleState,
    pub steps: Vec<Step>,
    pub assessment: Assessment,
    pub transition_trace: Vec<RngSpan>,
    pub routine_error: Option<String>,
    pub verification: &'static str,
}

/// CalcBonusReward + SaveBattleResult: break/overkill each add max(trunc(4*.1),1).
/// SetLastEnemyOverKill ORs qualifying kills across the entire battle.
pub fn opening_experience(reports: &[StepReport]) -> u32 {
    let mut broken = false;
    let mut overkill = false;
    for report in reports {
        let mut damage = std::collections::BTreeMap::new();
        for event in &report.events {
            match *event {
                Event::Broken { .. } => broken = true,
                Event::Damage { target, amount, .. } => {
                    damage.insert(target, amount);
                }
                Event::ResourceChanged {
                    actor,
                    resource: Resource::Hp,
                    before,
                    after: 0,
                } if actor.0 >= 3 && before > 0 => {
                    overkill |= damage
                        .get(&actor)
                        .is_some_and(|amount| u64::from(*amount) >= 2 * u64::from(before));
                }
                _ => (),
            }
        }
    }
    4 + u32::from(broken) + u32::from(overkill)
}

pub fn after_rejoin(
    arrival: CrtRng,
    opening: &Battle,
    reports: &[StepReport],
) -> Result<Prediction> {
    after_rejoin_with_plan(arrival, opening, reports, BoostPlan::default())
}

pub fn after_rejoin_with_plan(
    arrival: CrtRng,
    opening: &Battle,
    reports: &[StepReport],
    plan: BoostPlan,
) -> Result<Prediction> {
    plan.validate()?;
    ensure(
        opening.state().outcome == Outcome::Victory,
        "opening must be won before the second fight",
    )?;
    let exp = opening_experience(reports);
    let level = if exp >= 6 { 2 } else { 1 };
    // 0600 starts battle inside its rich sequence. StartEventBattle suspends
    // that event; the skip handler also stops at this handoff (+0 CRT).
    // EncountReset=true applies to FinishPre, which is not run at this boundary.
    // Same loaded map, no intervening encounter (user's short-route assumption).
    let entry = arrival;
    let transition_trace = vec![RngSpan {
        reason: "second_fight_serial_event_start",
        actor: None,
        before: arrival,
        after: entry,
    }];
    let mut config = BattleConfig::throne_second_at_level(entry.state, level)?;
    // A level-up fully restores HP/SP. Without one, retain prior combat damage
    // and spent SP; each new battle starts with its own default BP.
    if level == 1 {
        for (i, player) in config.players.iter_mut().enumerate() {
            let r = player.resources.as_mut().unwrap();
            r.hp = opening.state().actors[i].resources.hp;
            r.sp = opening.state().actors[i].resources.sp;
        }
    }
    for item in ItemCatalog::builtin()?.items {
        config
            .inventory
            .insert(item.id.clone(), opening.item_count(&item.id)?);
    }
    let actors = config
        .players
        .iter()
        .map(|p| Actor {
            name: p.name.clone(),
            side: Side::Player,
            max_hp: p.stats.max_hp,
            max_sp: p.stats.max_sp,
        })
        .chain(
            config
                .enemies
                .iter()
                .zip(["Right grunt", "Leader (middle)", "Left grunt"])
                .map(|(e, name)| Actor {
                    name: name.into(),
                    side: Side::Enemy,
                    max_hp: e.stats.max_hp,
                    max_sp: e.stats.max_sp,
                }),
        )
        .collect();
    let mut battle = Battle::new_throne_second(config, ItemCatalog::builtin()?, true)?;
    let initial_state = battle.state().clone();
    let mut assessment = Assessment::new(&battle)?;
    let mut routine = ThroneSecondRoutine;
    let mut steps = Vec::new();
    let mut routine_error = None;
    let mut manipulation = None;
    let (mut player_menu, mut round_menu, mut menu_round) = (0, 0, 0);
    while battle.state().outcome == Outcome::Ongoing {
        ensure(steps.len() < 18, "second fight exceeded action budget")?;
        let round = battle.state().round;
        let actor = battle.current_actor().expect("ongoing actor");
        if battle.side(actor) == Some(Side::Player) {
            player_menu += 1;
            if round != menu_round {
                round_menu = 0;
                menu_round = round;
            }
            round_menu += 1;
        }
        let before = battle.state().rng;
        let step = if battle.side(actor) == Some(Side::Player)
            && plan.advances > 0
            && plan.player_menu == player_menu
        {
            let command = routine.command(&battle)?;
            let normal_bp = match &command {
                ot2_battle_engine::Command::Boost { bp, .. } => *bp,
                _ => 0,
            };
            let inputs = plan.inputs(battle.state().actors[actor.0].resources.bp, normal_bp)?;
            let report = battle.step_player_with_boost_inputs(command, &inputs.levels);
            if report.is_ok() {
                let mut rng_after_inputs = before;
                rng_after_inputs.advance(
                    u64::from(plan.advances) + u64::from(normal_bp) * u64::from(normal_bp + 1) / 2,
                );
                manipulation = Some(Manipulation {
                    plan,
                    inputs,
                    actor,
                    round,
                    round_menu,
                    rng_before: before,
                    rng_after_inputs,
                });
            }
            report
        } else {
            routine.step(&mut battle)
        };
        match step {
            Ok(mut report) => {
                report.rng_before = before;
                assessment.record(&battle, &report);
                steps.push(Step {
                    round,
                    report,
                    actors_after: battle.state().actors.clone(),
                });
            }
            Err(error) => {
                routine_error = Some(error.to_string());
                break;
            }
        }
    }
    ensure(
        plan.advances == 0 || manipulation.is_some(),
        "Second-fight boost menu was not reached",
    )?;
    let mut after = entry;
    assessment.finish(battle.state().outcome);
    after.advance(battle.state().rng.draws);
    ensure(
        after.state == battle.state().rng.state,
        "second battle CRT continuity failed",
    )?;
    let city = (battle.state().outcome == Outcome::Victory && routine_error.is_none())
        .then(|| crate::city::after_second_victory(after))
        .transpose()?;
    let parlor = city.as_ref().map(crate::parlor::after_city).transpose()?;
    let parlor_revisit = parlor
        .as_ref()
        .map(crate::parlor_revisit::after_first_visit)
        .transpose()?;
    let mansion = parlor_revisit
        .as_ref()
        .map(crate::mansion::after_parlor_revisit)
        .transpose()?;
    Ok(Prediction {
        manipulation,
        mansion_approach_search: None,
        group: "ENG_EVE_THI_C01_020",
        routine_id: ThroneSecondRoutine::ID,
        arrival,
        entry,
        after,
        city,
        parlor,
        parlor_revisit,
        mansion,
        opening_exp: exp,
        party_level: level,
        actors,
        initial_state,
        state: battle.state().clone(),
        steps,
        assessment,
        transition_trace,
        routine_error,
        verification: "English/2x routine matched all 144 CRT calls and all six command menus in the 2026-09-27 capture (seed 0, entry 5120, drops 5264). Short encounter-free walk, skipped 0600, level-two party. Ends after drops, before returning to the field; other starting states remain source-derived predictions.",
    })
}
