//! CRT-relevant field callbacks. Data contains candidates and conditions, never
//! precomputed draw totals. The route supplies the world state and event path.
use crate::{CrtRng, Result, SimError, ensure};
use ot2_battle_engine::opening_crt::EncounterCounter;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;

#[derive(Debug, Deserialize)]
pub struct Catalog {
    fresh_throne: InitialState,
    pub maps: BTreeMap<String, Map>,
    pub placements: BTreeMap<String, Placement>,
    pub flag_rules: Vec<FlagRule>,
    pub events: BTreeMap<String, Event>,
    pub tutorials: BTreeMap<u32, Tutorial>,
}
#[derive(Debug, Deserialize)]
struct InitialState {
    map: String,
    time: u8,
    party: Vec<u32>,
    bench: Vec<u32>,
    story_types: BTreeSet<i32>,
    active_scenarios: BTreeSet<u32>,
    prologues: BTreeSet<String>,
    shared_levels: BTreeSet<String>,
}
#[derive(Debug, Deserialize)]
pub struct Map {
    pub id: u32,
    pub streams: Vec<String>,
    pub field_streams: Vec<String>,
    pub environment_draws: Vec<EnvironmentDraw>,
    pub placements: Vec<String>,
    pub counter: EncounterCounter,
    pub bar_event: String,
}
#[derive(Debug, Deserialize)]
pub struct EnvironmentDraw {
    level: String,
    actor: String,
    kind: EnvironmentDrawKind,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EnvironmentDrawKind {
    CanoeBobbingTime,
}
#[derive(Debug, Deserialize)]
pub struct Placement {
    kind: u8,
    resource: String,
    action: Option<String>,
    begin: u32,
    end: u32,
    time: String,
    start_delay: i32,
    end_delay: i32,
    story_type: i32,
    scenario: u32,
    not_prologue: bool,
    prologue: String,
    not_final: bool,
    excludes: Vec<String>,
}
#[derive(Debug, Deserialize)]
pub struct FlagRule {
    pub id: String,
    trigger: String,
    begin: u32,
    end: u32,
    apply: bool,
    candidates: Vec<Candidate>,
}
#[derive(Debug, Deserialize)]
struct Candidate {
    flag: u32,
    weight: u32,
}
#[derive(Debug, Deserialize)]
pub struct Event {
    pub map_id: u32,
    pub reset_counter: bool,
    #[serde(default)]
    pub completion: Option<StoryCompletion>,
    pub commands: Vec<Command>,
}

#[derive(Debug, Deserialize)]
pub struct StoryCompletion {
    pub scenario: u32,
    pub character: String,
}
#[derive(Debug, Deserialize)]
pub struct Command {
    index: usize,
    pub(crate) cmd: u32,
    target: i32,
    opt: Vec<String>,
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct Tutorial {
    label: String,
    list_flag: u32,
    opened_flag: u32,
}

/// Explicit dialog selections keyed by event name and command index.
pub type EventChoices = BTreeMap<(String, usize), String>;

pub fn catalog() -> &'static Catalog {
    static DATA: OnceLock<Catalog> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(include_str!("../data/town-loading.json"))
            .expect("imported town rules")
    })
}

/// Persistent state at a field boundary. Fresh route values come from event
/// execution; callers extending the route retain the resulting state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldState {
    pub map: String,
    /// Ordinary field transitions observed by this route (save visit/icon bits).
    /// Older exports did not track these; no complete prior visit history is inferred.
    #[serde(default)]
    pub visited_maps: BTreeSet<String>,
    pub time: u8,
    pub flags: BTreeSet<u32>,
    /// Native event integer variables (7410 increments; 7420 compares).
    #[serde(default)]
    pub event_variables: BTreeMap<i32, i32>,
    pub party: Vec<u32>,
    pub bench: Vec<u32>,
    pub story_types: BTreeSet<i32>,
    pub active_scenarios: BTreeSet<u32>,
    pub prologues: BTreeSet<String>,
    pub resident_levels: BTreeSet<String>,
    pub shared_levels: BTreeSet<String>,
    pub existing_npcs: BTreeSet<String>,
    /// Native saved spawn-delay timers: -1/missing initializes, zero expires.
    pub npc_start_delays: BTreeMap<String, i32>,
    pub npc_end_delays: BTreeMap<String, i32>,
    pub first_encounter: bool,
    pub encounter_counter: u32,
    /// Native callback suppression state, false on the supported fresh route.
    pub suppress_flag_callbacks: bool,
    pub suppress_level_lottery: bool,
    pub suppress_time_callbacks: bool,
}
impl WorldState {
    pub fn flag(&self, flag: u32) -> bool {
        self.flags.contains(&flag)
    }
    pub fn set_flag(&mut self, flag: u32, value: bool) {
        if flag != 0 {
            if value {
                self.flags.insert(flag);
            } else {
                self.flags.remove(&flag);
            }
        }
    }
    pub fn final_chapter(&self) -> bool {
        self.flag(2008) && !self.flag(2009)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DrawKind {
    Counter,
    StreamedLevel,
    EnvironmentInitialization,
    NpcFrame,
    FlagLottery,
    PathActionMenu,
    StealCheck,
    DialogueLottery,
    Boundary,
}

#[derive(Debug, Serialize)]
pub struct Trace {
    pub reason: String,
    pub kind: DrawKind,
    pub subject: String,
    pub value: Option<u32>,
    pub before: CrtRng,
    pub after: CrtRng,
}
#[derive(Debug, Serialize)]
pub struct Decision {
    pub callback: String,
    pub subject: String,
    pub result: String,
    pub rng: CrtRng,
}

pub struct Loader<'a> {
    pub data: &'a Catalog,
    pub state: WorldState,
    pub rng: CrtRng,
    pub trace: Vec<Trace>,
    pub decisions: Vec<Decision>,
}
impl<'a> Loader<'a> {
    pub fn new(data: &'a Catalog, state: WorldState, rng: CrtRng) -> Result<Self> {
        ensure(
            data.maps.contains_key(&state.map),
            format!("No town data for {}", state.map),
        )?;
        ensure((1..=4).contains(&state.time), "Invalid world time phase")?;
        Ok(Self {
            data,
            state,
            rng,
            trace: Vec::new(),
            decisions: Vec::new(),
        })
    }
    fn note(&mut self, callback: &str, subject: &str, result: impl Into<String>) {
        self.decisions.push(Decision {
            callback: callback.into(),
            subject: subject.into(),
            result: result.into(),
            rng: self.rng,
        });
    }
    pub fn boundary(&mut self, reason: impl Into<String>) {
        self.trace.push(Trace {
            reason: reason.into(),
            kind: DrawKind::Boundary,
            subject: self.state.map.clone(),
            value: None,
            before: self.rng,
            after: self.rng,
        });
    }
    pub(crate) fn draw(
        &mut self,
        kind: DrawKind,
        subject: &str,
        reason: impl Into<String>,
        range: Option<u32>,
    ) -> u32 {
        let before = self.rng;
        let value = match range {
            Some(n) => self.rng.index(n),
            None => u32::from(self.rng.next_u15()),
        };
        self.trace.push(Trace {
            reason: reason.into(),
            kind,
            subject: subject.into(),
            value: Some(value),
            before,
            after: self.rng,
        });
        value
    }
    pub fn reset_counter(&mut self, callback: &str) -> Result<()> {
        let map = self
            .data
            .maps
            .get(&self.state.map)
            .ok_or_else(|| SimError("Missing current map".into()))?;
        let (min, b, a, ratio) = (
            map.counter.min,
            map.counter.range_b,
            map.counter.range_a,
            if self.state.first_encounter {
                map.counter.first_ratio
            } else {
                1.0
            },
        );
        let subject = self.state.map.clone();
        let b = self.draw(
            DrawKind::Counter,
            &subject,
            format!("{callback}: counter B"),
            Some(b),
        );
        let a = self.draw(
            DrawKind::Counter,
            &subject,
            format!("{callback}: counter A"),
            Some(a),
        );
        self.state.encounter_counter = ((min + b + a) as f32 * ratio) as u32;
        self.note(
            callback,
            &subject,
            format!("counter = {}", self.state.encounter_counter),
        );
        Ok(())
    }
    /// All rows, including single-candidate rules, execute in cooked table order.
    /// Their writes take effect before testing the next row.
    pub fn flag_lotteries(&mut self, time_change: bool) -> Result<()> {
        let trigger = if time_change {
            "eTimeZoneChange"
        } else {
            "eLevelChange"
        };
        if self.state.suppress_flag_callbacks || (!time_change && self.state.suppress_level_lottery)
        {
            self.note(trigger, "FlagRandomControlDB", "callback suppressed");
            return Ok(());
        }
        for row in &self.data.flag_rules {
            if row.trigger != trigger {
                continue;
            }
            if (row.begin != 0 && !self.state.flag(row.begin))
                || (row.end != 0 && self.state.flag(row.end))
            {
                self.note(trigger, &row.id, "flag gate closed");
                continue;
            }
            let sum = row
                .candidates
                .iter()
                .try_fold(0u32, |a, c| a.checked_add(c.weight))
                .ok_or_else(|| SimError("Flag lottery weight overflow".into()))?;
            ensure(sum < i32::MAX as u32, "Flag lottery range overflow")?;
            for c in &row.candidates {
                self.state.set_flag(c.flag, !row.apply);
            }
            // Native inclusive 0..sum, with <= comparison against cumulative weight.
            let roll = self.draw(
                DrawKind::FlagLottery,
                &row.id,
                format!("{trigger}: {}", row.id),
                Some(sum + 1),
            );
            let mut cumulative = 0;
            for c in &row.candidates {
                cumulative += c.weight;
                if roll <= cumulative && c.flag != 0 {
                    self.state.set_flag(c.flag, row.apply);
                    self.note(
                        trigger,
                        &row.id,
                        format!("roll {roll}; flag {} = {}", c.flag, row.apply),
                    );
                    break;
                }
            }
        }
        Ok(())
    }
    /// Stable Day/Night and scripted Day -> Dusk / Night -> Dawn (f6eb70).
    pub fn set_time(&mut self, time: u8, callback: &str) -> Result<()> {
        ensure(
            time == 2
                || time == 4
                || (time == 3 && self.state.time == 2)
                || (time == 1 && self.state.time == 4),
            "Only stable Day/Night and scripted Day-to-Dusk / Night-to-Dawn are imported",
        )?;
        // Native ed58b0 refreshes using the OLD phase before f6aa40 writes it.
        if time == 1 || time == 3 {
            self.refresh_bar()?;
            self.refresh_npcs(if time == 1 {
                "SetTimeZoneType pre-dawn placement refresh"
            } else {
                "SetTimeZoneType pre-dusk placement refresh"
            })?;
        }
        let old = match self.state.time {
            1 => 4,
            3 => 2,
            t => t,
        };
        if old != time && !self.state.suppress_time_callbacks {
            let elapsed = (time + 4 - old) % 4;
            for timer in self
                .state
                .npc_start_delays
                .values_mut()
                .chain(self.state.npc_end_delays.values_mut())
            {
                if *timer > 0 {
                    *timer = (*timer - i32::from(elapsed)).max(0);
                }
            }
            if elapsed >= 2 {
                self.state.set_flag(0x4edd, false);
                self.flag_lotteries(true)?;
            }
        }
        self.state.time = time;
        self.note(callback, "time", format!("{old} -> {time}"));
        self.boundary(format!("{callback}: time {old} -> {time}"));
        Ok(())
    }
    fn placement_enabled(
        &mut self,
        label: &str,
        labels: &BTreeSet<String>,
        stack: &mut Vec<String>,
    ) -> Result<bool> {
        ensure(
            !stack.iter().any(|x| x == label),
            format!("Placement exclusion cycle at {label}"),
        )?;
        let p = self
            .data
            .placements
            .get(label)
            .ok_or_else(|| SimError(format!("Missing placement {label}")))?;
        let mut enabled =
            (p.begin == 0 || self.state.flag(p.begin)) && (p.end == 0 || !self.state.flag(p.end));
        if p.kind == 1 {
            if enabled && p.start_delay > 0 {
                let timer = self
                    .state
                    .npc_start_delays
                    .entry(label.into())
                    .or_insert(-1);
                if *timer == -1 {
                    *timer = p.start_delay;
                }
                enabled = *timer == 0;
            }
            if p.end != 0 && self.state.flag(p.end) && p.end_delay > 0 {
                let timer = self.state.npc_end_delays.entry(label.into()).or_insert(-1);
                if *timer == -1 {
                    *timer = p.end_delay;
                }
                enabled = *timer != 0;
            }
        }
        if !enabled {
            return Ok(false);
        }
        if p.not_prologue && self.state.prologues.contains(&p.prologue) {
            return Ok(false);
        }
        if p.not_final && self.state.final_chapter() {
            return Ok(false);
        }
        let time_matches = match p.time.as_str() {
            "None" => false,
            "eALL" => true,
            "eDAYTIME" => matches!(self.state.time, 2 | 3),
            "eNIGHT" => matches!(self.state.time, 1 | 4),
            other => {
                return Err(SimError(format!(
                    "Unsupported placement time {other}: {label}"
                )));
            }
        };
        if !time_matches {
            return Ok(false);
        }
        if p.story_type < 0 {
            if !self.state.story_types.is_empty() {
                return Ok(false);
            }
        } else if p.story_type != 0 && !self.state.story_types.contains(&p.story_type) {
            return Ok(false);
        }
        if p.scenario != 0 && !self.state.active_scenarios.contains(&p.scenario) {
            return Ok(false);
        }
        stack.push(label.into());
        for excluded in &p.excludes {
            if labels.contains(excluded) && self.placement_enabled(excluded, labels, stack)? {
                stack.pop();
                return Ok(false);
            }
        }
        stack.pop();
        Ok(true)
    }
    pub fn refresh_npcs(&mut self, callback: &str) -> Result<()> {
        let map = &self.data.maps[&self.state.map];
        let labels: BTreeSet<_> = map.placements.iter().cloned().collect();
        // Native uses PlacementList order, not the map's sorted key order.
        for label in &map.placements {
            let p = &self.data.placements[label];
            if p.kind != 1 {
                continue;
            }
            if !self.placement_enabled(label, &labels, &mut Vec::new())? {
                self.state.existing_npcs.remove(label);
                self.note(callback, label, "placement gate closed");
            } else if self.state.existing_npcs.insert(label.clone()) {
                ensure(
                    p.action.is_some(),
                    format!("Missing NPC resource {}", p.resource),
                )?;
                self.draw(
                    DrawKind::NpcFrame,
                    label,
                    format!("{callback}: NPC frame {label}"),
                    Some(16),
                );
            } else {
                self.note(callback, label, "existing actor refreshed; no spawn draw");
            }
        }
        Ok(())
    }
    pub(crate) fn refresh_bar(&mut self) -> Result<()> {
        let map = &self.data.maps[&self.state.map];
        if self.state.final_chapter() || map.bar_event == "None" {
            self.note(
                "RefreshBarCharacter",
                &self.state.map.clone(),
                if self.state.final_chapter() {
                    "final chapter; no lottery"
                } else {
                    "BarEventLabel=None; no lottery"
                },
            );
            return Ok(());
        }
        Err(SimError(format!(
            "Tavern callback data is not imported for {}",
            self.state.map
        )))
    }
    fn initialize_environment(&mut self, map: &Map, levels: &BTreeSet<String>, callback: &str) {
        for init in &map.environment_draws {
            if levels.contains(&init.level) {
                let reason = match init.kind {
                    EnvironmentDrawKind::CanoeBobbingTime => "Canoe initial bobbing time",
                };
                self.draw(
                    DrawKind::EnvironmentInitialization,
                    &format!("{}.{}", init.level, init.actor),
                    if callback.is_empty() {
                        reason.into()
                    } else {
                        format!("{callback}: {reason}")
                    },
                    None,
                );
            }
        }
    }

    /// Re-show the resident field after battle. The live Escape trace confirms
    /// its canoe BeginPlay callbacks run again before the B/A reset. Residency
    /// avoids constructor draws, not these actor callbacks. Battle levels stay out.
    pub fn restore_field_after_battle(&mut self, callback: &str) -> Result<()> {
        let map = &self.data.maps[&self.state.map];
        let field_levels = map.field_streams.iter().cloned().collect::<BTreeSet<_>>();
        ensure(
            field_levels.is_subset(&self.state.resident_levels),
            "Cannot restore a field whose levels are not resident",
        )?;
        self.initialize_environment(map, &field_levels, callback);
        self.reset_counter(callback)
    }

    /// Ordinary on-foot transition. Scripted event loads have different visit
    /// gates and continue using load_map directly. SaveVisitedMap_Impl updates
    /// visit/icon/mask data without a CRT lottery; it does not reuse loaded actors.
    pub fn visit_map(&mut self, destination: &str) -> Result<()> {
        ensure(
            self.data.maps.contains_key(destination),
            format!("No town-loading data for {destination}"),
        )?;
        let first = self.state.visited_maps.insert(destination.into());
        self.note(
            "SaveVisitedMap_Impl",
            destination,
            if first {
                "visited/icon bits set; no CRT draw"
            } else {
                "already visited; no CRT draw"
            },
        );
        self.load_map(destination)
    }

    /// Caller must evaluate the popup's eligibility; opening/dismissal only
    /// updates tutorial flags and does not reset the encounter counter.
    pub fn dismiss_tutorial(&mut self, id: u32) -> Result<()> {
        let tutorial = self
            .data
            .tutorials
            .get(&id)
            .ok_or_else(|| SimError(format!("Missing tutorial {id}")))?;
        self.state.set_flag(tutorial.list_flag, true);
        self.state.set_flag(tutorial.opened_flag, true);
        self.boundary(format!("Tutorial {id} ({}) dismissed", tutorial.label));
        Ok(())
    }

    pub fn load_map(&mut self, destination: &str) -> Result<()> {
        let map = self
            .data
            .maps
            .get(destination)
            .ok_or_else(|| SimError(format!("No town-loading data for {destination}")))?;
        if self.state.map == destination {
            self.note(
                "RequestLoadFieldLevel",
                destination,
                "already current; no transition",
            );
            return Ok(());
        }
        // ResetEncountVolume restores FirstEncount before rolling, even in town.
        self.state.first_encounter = true;
        self.reset_counter("RequestLoadFieldLevel outgoing")?;
        self.flag_lotteries(false)?;
        self.state.existing_npcs.clear();
        // ed9bc0 unloads the outgoing field AND its BattleMapNameList before
        // ed98a0 loads the destination. Equal battle-map names do not imply
        // reuse. Btl_SinExpose is retained separately, outside either map list.
        for level in &self.data.maps[&self.state.map].streams {
            if self.state.resident_levels.remove(level) {
                self.note("Unload outgoing map", level, "field-owned level unloaded");
            }
        }
        let requested: BTreeSet<_> = map
            .streams
            .iter()
            .chain(self.state.shared_levels.iter())
            .cloned()
            .collect();
        self.state.resident_levels.retain(|l| requested.contains(l));
        let mut created_levels = BTreeSet::new();
        for level in map
            .streams
            .iter()
            .chain(self.state.shared_levels.clone().iter())
        {
            if self.state.resident_levels.insert(level.clone()) {
                created_levels.insert(level.clone());
                self.draw(
                    DrawKind::StreamedLevel,
                    level,
                    format!("Create streamed level {level}"),
                    None,
                );
            } else {
                self.note(
                    "Create streamed level",
                    level,
                    "resident; no constructor draw",
                );
            }
        }
        // On first load, actors begin play after level construction and before
        // NPC placement. Battle return has its own re-show callback above.
        self.initialize_environment(map, &created_levels, "");
        self.state.map = destination.into();
        self.refresh_npcs("OnFieldLevelShown")?;
        self.refresh_bar()?;
        self.reset_counter("OnFieldLevelShown incoming")?;
        Ok(())
    }
    pub fn map_name(&self, id: u32) -> Result<String> {
        self.data
            .maps
            .iter()
            .find(|(_, m)| m.id == id)
            .map(|(n, _)| n.clone())
            .ok_or_else(|| SimError(format!("Missing map id {id}")))
    }
    /// Execute audited skipped-event commands, including branch/jump semantics.
    /// During bootstrap only state mutations run; earlier RNG belongs to the
    /// preceding route stages and is never consumed a second time.
    fn run_event(
        &mut self,
        name: &str,
        bootstrap: bool,
        depth: usize,
        choices: &EventChoices,
        stop_before: Option<&str>,
    ) -> Result<bool> {
        ensure(depth < 16, "Event recursion limit")?;
        let event = self
            .data
            .events
            .get(name)
            .ok_or_else(|| SimError(format!("Missing event {name}")))?;
        if !bootstrap {
            if let Some(completion) = &event.completion {
                ensure(
                    completion.scenario == 700
                        && self.state.party == [7]
                        && self.state.bench.is_empty(),
                    "Only fresh solo Throné chapter completion is audited",
                )?;
                // PlayEvent -> UpdateMainMissionProgress -> f55560 -> f4ec50:
                // final normal task sets status 5 before EventLoadLevel.
                self.state.active_scenarios.remove(&completion.scenario);
                self.state.story_types.remove(&7);
                self.state.prologues.remove(&completion.character);
                self.boundary(format!("{name}: chapter completed before map load"));
            }
            let destination = self.map_name(event.map_id)?;
            if self.state.map != destination {
                self.load_map(&destination)?;
            }
            if stop_before == Some(name) {
                self.boundary(format!("{name}: arrived before event commands"));
                return Ok(true);
            }
        }
        let mut pc = 0;
        let mut steps = 0;
        // Native flag/time commands mark EventManager +0x810. FinishPre
        // (e8e740) dispatches ed58b0 before its encounter-counter callback.
        let mut placements_dirty = false;
        while let Some(c) = event.commands.get(pc) {
            ensure(steps < 1024, format!("Event command loop in {name}"))?;
            steps += 1;
            ensure(c.index == pc, "Invalid event command index")?;
            pc += 1;
            match c.cmd {
                70 | 71 => {
                    let v = c.opt.first().map(String::as_str);
                    ensure(matches!(v, Some("0" | "1")), "Unsupported flag assignment")?;
                    self.state.set_flag(c.target as u32, v == Some("1"));
                    placements_dirty = true;
                    if !bootstrap {
                        self.note(
                            name,
                            &c.target.to_string(),
                            format!("flag = {}", v.unwrap()),
                        );
                    }
                }
                4000 => {
                    let branch = usize::from(!self.state.flag(c.target as u32));
                    pc = c.opt[branch]
                        .parse::<usize>()
                        .map_err(|_| SimError("Invalid flag branch".into()))?
                        + 1;
                }
                5 => {
                    pc = usize::try_from(c.target).map_err(|_| SimError("Invalid jump".into()))? + 1
                }
                5010 => {
                    let selected = choices.get(&(name.into(), c.index)).ok_or_else(|| {
                        SimError(format!("Missing dialog choice for {name}:{}", c.index))
                    })?;
                    let pair = c
                        .opt
                        .chunks_exact(2)
                        .find(|pair| &pair[0] == selected)
                        .ok_or_else(|| {
                            SimError(format!(
                                "Invalid dialog choice {selected} for {name}:{}",
                                c.index
                            ))
                        })?;
                    pc = pair[1]
                        .parse::<usize>()
                        .map_err(|_| SimError("Invalid dialog branch".into()))?
                        + 1;
                    if !bootstrap {
                        self.boundary(format!("{name}: select {selected}"));
                    }
                }
                3000 => {
                    if !self.state.party.contains(&(c.target as u32)) {
                        self.state.party.push(c.target as u32);
                    }
                    if name == "MS_GAK_00_0100" {
                        self.boundary("Osvald joined the party; fixed join dialog dismissed");
                    }
                }
                3010 | 3120 if name == "MS_GAK_00_0100" && c.target == 6 => {
                    let selected = choices.get(&(name.into(), c.index)).ok_or_else(|| {
                        SimError(format!("Missing recruitment choice for {name}:{}", c.index))
                    })?;
                    let branch = match selected.as_str() {
                        "eDIALOG_YES" => 0,
                        "eDIALOG_NO" => 1,
                        _ => return Err(SimError("Invalid recruitment choice".into())),
                    };
                    // 3010 accepts the invitation; 3120 asks to play the prologue.
                    ensure(
                        c.cmd != 3120 || branch == 1,
                        "Playing Osvald's prologue is not supported by this route",
                    )?;
                    pc = c.opt[branch]
                        .parse::<usize>()
                        .map_err(|_| SimError("Invalid recruitment branch".into()))?
                        + 1;
                    self.boundary(format!("{name}:{} select {selected}", c.index));
                }
                3005 if name == "MS_GAK_00_0100" && c.target == 6 => {
                    ensure(
                        self.state.party.contains(&6) && !self.state.flag(2900),
                        "Only immediate Osvald recruitment without a prologue is audited",
                    )?;
                    // StartAddMemberEnd finds him already in the active party;
                    // EndPrologueReminiscence returns early without flag 0xb54.
                    self.boundary("Osvald retained in active party; no party-change dialog");
                }
                7410 if name == "MS_GAK_00_0100" => {
                    let amount = c.opt[0]
                        .parse::<i32>()
                        .map_err(|_| SimError("Invalid event variable increment".into()))?;
                    *self.state.event_variables.entry(c.target).or_default() += amount;
                }
                7420 if name == "MS_GAK_00_0100" => {
                    let expected = c.opt[0]
                        .parse::<i32>()
                        .map_err(|_| SimError("Invalid event variable comparison".into()))?;
                    let equal = self
                        .state
                        .event_variables
                        .get(&c.target)
                        .copied()
                        .unwrap_or(0)
                        == expected;
                    pc = c.opt[if equal { 1 } else { 2 }]
                        .parse::<usize>()
                        .map_err(|_| SimError("Invalid event variable branch".into()))?
                        + 1;
                }
                3020 => self.state.party.retain(|id| *id != c.target as u32),
                1550 => {
                    let time = c.opt[0]
                        .parse::<u8>()
                        .map_err(|_| SimError("Invalid time command".into()))?
                        + 1;
                    if bootstrap {
                        self.state.time = time;
                    } else {
                        self.set_time(time, name)?;
                    }
                    placements_dirty = true;
                }
                500 => {
                    if !bootstrap {
                        let map = self.map_name(c.target as u32)?;
                        self.load_map(&map)?;
                    }
                }
                9990 => {
                    if !bootstrap {
                        if placements_dirty {
                            self.refresh_bar()?;
                            self.refresh_npcs(&format!("{name} FinishPre placement refresh"))?;
                            placements_dirty = false;
                        }
                        if event.reset_counter {
                            self.reset_counter(&format!("{name} FinishPre"))?;
                        }
                    }
                }
                1000 => {
                    if !bootstrap {
                        if self.run_event(&c.opt[0], false, depth + 1, choices, stop_before)? {
                            return Ok(true);
                        }
                    }
                }
                7100 | 7101 => {
                    ensure(
                        c.cmd == 7100
                            || (name == "SC_FC13_Tutorial_0100" && c.target == 2 && c.opt == ["0"]),
                        "Unaudited conditional tutorial",
                    )?;
                    let tutorial = self
                        .data
                        .tutorials
                        .get(&(c.target as u32))
                        .ok_or_else(|| SimError(format!("Missing tutorial {}", c.target)))?;
                    self.state.set_flag(tutorial.list_flag, true);
                    self.state.set_flag(tutorial.opened_flag, true);
                    if !bootstrap {
                        self.boundary(format!(
                            "Tutorial {} ({}) dismissed",
                            c.target, tutorial.label
                        ));
                    }
                }
                // Audited fixed dialogue/actor activation for the Steal tutorial.
                // Keep other dialogue paths unsupported: general/random talk
                // and newly spawned actors must not be treated as zero-draw.
                300 if name == "MS_TOU_10_1300" && c.target == 0 && c.opt == ["1", "1"] => {
                    if !bootstrap {
                        self.boundary("1300: existing party actors enabled");
                    }
                }
                40 if name == "MS_TOU_10_1300"
                    && c.target == 0
                    && c.text == "TX_MS_TOU_10_1300_0320"
                    && c.opt == ["1", "0", "0", "0"] =>
                {
                    if !bootstrap {
                        self.boundary("1300: fixed dialogue completed");
                    }
                }
                40 if name == "MS_TOU_10_1400"
                    && c.target == 0
                    && matches!(
                        c.text.as_str(),
                        "TX_MS_TOU_10_1301_0010" | "TX_MS_TOU_10_1301_0020"
                    )
                    && c.opt == ["1", "0", "0", "0"] =>
                {
                    // Item-specific clothes dialogue; audited fixed lines,
                    // bypassing the generic Steal success-line lottery.
                    if !bootstrap {
                        self.boundary(format!("1400: fixed dialogue {} completed", c.text));
                    }
                }
                // Al's fixed skipped introduction: event-only actors do not run
                // the ordinary field NPC spawn/timer path. No randomized talk.
                40 if name == "SC_FC13_Tutorial_0100"
                    && c.target == 10
                    && c.text.starts_with("TX_SS_SP_Tutorial_0100_")
                    && c.opt == ["1", "0", "0", "0"] => {}
                4 | 10 | 20 | 30 | 100 | 106 | 203 | 220 | 240 | 300 | 7400 | 8010
                    if name == "SC_FC13_Tutorial_0100" => {}
                1300 if name == "SC_FC13_Tutorial_0100" && c.opt == ["SS_TUT_003"] => {}
                40 if name == "MS_GAK_00_0100"
                    && c.text.starts_with("TX_PCJ_GAK_0000_")
                    && c.opt == ["1", "0", "0", "0"] => {}
                // Event-only actor and fixed presentation commands. The NPC
                // frame lottery belongs to ordinary map placement, not these.
                10 | 20 | 30 | 60 | 200 | 240 | 300 | 8010 | 9100 if name == "MS_GAK_00_0100" => {}
                10000 => break,
                // Audited route-specific commands; unknown commands are an error.
                // 230 positions the existing player; 2310 heals party resources.
                1 | 2 | 42 | 52 | 62 | 230 | 2310 | 310 | 910 | 1250 | 1251 | 1500 | 1501
                | 9980 | 8030 | 8900 => {}
                other => {
                    return Err(SimError(format!(
                        "Unaudited command {other} in {name}:{}",
                        c.index
                    )));
                }
            }
        }
        Ok(false)
    }
    pub fn skipped_event(&mut self, name: &str) -> Result<()> {
        self.skipped_event_with_choices(name, &EventChoices::new())
    }
    pub fn skipped_event_with_choices(&mut self, name: &str, choices: &EventChoices) -> Result<()> {
        self.run_event(name, false, 0, choices, None).map(|_| ())
    }
    /// Stop after a serial event's destination loads, before its commands run.
    pub fn skipped_event_until_arrival(
        &mut self,
        name: &str,
        destination_event: &str,
    ) -> Result<()> {
        let arrived = self.run_event(
            name,
            false,
            0,
            &EventChoices::new(),
            Some(destination_event),
        )?;
        ensure(
            arrived,
            format!("Event {name} did not reach {destination_event}"),
        )
    }
}

/// Known route constants plus commands actually executed before the second
/// victory. This constructor doesn't advance the supplied post-battle RNG.
pub fn throne_after_second_state() -> Result<WorldState> {
    let data = catalog();
    let p = &data.fresh_throne;
    let state = WorldState {
        map: p.map.clone(),
        visited_maps: BTreeSet::new(),
        time: p.time,
        flags: BTreeSet::new(),
        event_variables: BTreeMap::new(),
        party: p.party.clone(),
        bench: p.bench.clone(),
        story_types: p.story_types.clone(),
        active_scenarios: p.active_scenarios.clone(),
        prologues: p.prologues.clone(),
        resident_levels: data.maps[&p.map]
            .streams
            .iter()
            .chain(p.shared_levels.iter())
            .cloned()
            .collect(),
        shared_levels: p.shared_levels.clone(),
        existing_npcs: BTreeSet::new(),
        npc_start_delays: BTreeMap::new(),
        npc_end_delays: BTreeMap::new(),
        first_encounter: true,
        encounter_counter: 0,
        suppress_flag_callbacks: false,
        suppress_level_lottery: false,
        suppress_time_callbacks: false,
    };
    let mut loader = Loader::new(data, state, CrtRng::new(0))?;
    for suffix in ["0100", "0200", "0300", "0400", "0410", "0500", "0600"] {
        loader.run_event(
            &format!("MS_TOU_10_{suffix}"),
            true,
            0,
            &EventChoices::new(),
            None,
        )?;
    }
    // Actors already existed before this boundary. Resolve their identities,
    // not their frame rolls; those draws are in the previous route stage.
    let labels: BTreeSet<_> = data.maps[&p.map].placements.iter().cloned().collect();
    for label in &data.maps[&p.map].placements {
        if data.placements[label].kind == 1
            && loader.placement_enabled(label, &labels, &mut Vec::new())?
        {
            loader.state.existing_npcs.insert(label.clone());
        }
    }
    ensure(loader.rng.draws == 0, "World bootstrap advanced RNG")?;
    Ok(loader.state)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn loader() -> Loader<'static> {
        Loader::new(
            catalog(),
            throne_after_second_state().unwrap(),
            CrtRng::new(0),
        )
        .unwrap()
    }

    #[test]
    fn black_market_is_time_triggered_and_preserves_one_selected_flag() {
        let mut l = loader();
        l.flag_lotteries(false).unwrap();
        l.set_time(4, "same Night").unwrap();
        assert_eq!(l.rng.draws, 0);
        for time in [2, 4, 2, 4] {
            let before = l.rng.draws;
            l.set_time(time, "day/night switch").unwrap();
            assert_eq!(l.rng.draws - before, 1);
            assert_eq!((25500..=25503).filter(|f| l.state.flag(*f)).count(), 1);
        }
        assert!(
            l.trace
                .iter()
                .filter(|t| t.kind == DrawKind::FlagLottery)
                .all(|t| t.subject == "GMK_BLACKMARKET_CONTROL")
        );
    }

    #[test]
    fn guaranteed_progression_lottery_draws_once_then_closes_its_own_gate() {
        let mut l = loader();
        l.state.set_flag(20313, true);
        l.flag_lotteries(false).unwrap();
        assert_eq!(l.rng.draws, 1);
        assert_eq!(l.trace[0].subject, "SUB_TSe21_0100_GRAPE");
        assert!(l.state.flag(20314));
        l.flag_lotteries(false).unwrap();
        assert_eq!(l.rng.draws, 1);
        l.state.set_flag(20314, false);
        l.state.suppress_level_lottery = true;
        l.flag_lotteries(false).unwrap();
        assert_eq!(l.rng.draws, 1);
    }

    #[test]
    fn time_lottery_observes_row_order_and_prior_flag_mutation() {
        let mut l = loader();
        l.state.set_flag(9255, true);
        // MS_COP_ST2_SINSO_0020 occurs before _0010 in the cooked table.
        // _0010 opens the earlier row's begin gate; it runs only next callback.
        l.set_time(2, "first switch").unwrap();
        assert_eq!(l.rng.draws, 2); // Black Market + _0010.
        assert!(l.state.flag(25492));
        assert!(!l.state.flag(25526));
        l.set_time(4, "second switch").unwrap();
        assert_eq!(l.rng.draws, 4); // Black Market + _0020; _0010 now gated off.
        assert!(l.state.flag(25526));
    }

    #[test]
    fn npc_refresh_does_not_repeat_spawn_rolls_and_gates_are_re_evaluated() {
        let mut l = loader();
        l.load_map("Twn_Cty_1_1_B").unwrap();
        let before = l.rng;
        l.refresh_npcs("repeat refresh").unwrap();
        assert_eq!(l.rng, before);
        l.state.set_flag(2206, false);
        l.refresh_npcs("remove marker").unwrap();
        assert_eq!(l.rng, before);
        assert!(!l.state.existing_npcs.contains("Epilogue_PLAYER_IS_Throne"));
        l.state.set_flag(2206, true);
        l.refresh_npcs("new marker").unwrap();
        assert_eq!(l.rng.draws, before.draws + 1);
        assert_eq!(l.trace.last().unwrap().subject, "Epilogue_PLAYER_IS_Throne");
    }

    #[test]
    fn next_new_delsta_zone_uses_persisted_world_and_keeps_shared_level() {
        let p = crate::city::after_second_victory(CrtRng::new(0)).unwrap();
        let mut l = Loader::new(catalog(), p.world_after, p.after_popup).unwrap();
        l.load_map("Twn_Cty_1_1_A").unwrap();
        assert!(l.state.flag(8007));
        assert_eq!(l.state.party, vec![7]);
        assert!(!l.trace.iter().any(|t| t.subject == "Btl_SinExpose"));
        assert_eq!(
            l.trace
                .iter()
                .filter(|t| t.kind == DrawKind::Counter)
                .count(),
            4
        );
        let before = l.rng;
        l.load_map("Twn_Cty_1_1_A").unwrap();
        assert_eq!(l.rng, before);
        assert!(l.load_map("missing map").is_err());
        assert!(l.set_time(0, "invalid phase").is_err());
    }

    #[test]
    fn mansion_finish_refreshes_flag_enabled_actors_before_rolling_counter() {
        let city = crate::city::after_second_victory(CrtRng::new(0)).unwrap();
        let first = crate::parlor::after_city(&city).unwrap();
        let revisit = crate::parlor_revisit::after_first_visit(&first).unwrap();
        let mut l = Loader::new(catalog(), revisit.world_after, CrtRng::new(0)).unwrap();
        l.state.set_flag(8013, true); // Clothes acquired; 1500 trigger enabled.
        l.state.set_flag(34049, false);
        l.load_map("Twn_Cty_1_1_A").unwrap();
        l.rng = CrtRng {
            state: 0x94b3ad61,
            draws: 5535,
        };
        l.trace.clear();
        l.skipped_event("MS_TOU_10_1500").unwrap();
        assert_eq!(
            l.rng,
            CrtRng {
                state: 0xddcc3032,
                draws: 5562
            }
        );
        assert_eq!(l.state.map, "Dng_Cty_1_1");
        assert_eq!(l.state.time, 4);
        assert!(l.state.flag(8015) && l.state.flag(25305));
        assert_eq!(l.trace[0].subject, "MS_TOU_10_16001800");
        let finish: Vec<_> = l
            .trace
            .iter()
            .filter(|t| t.reason.starts_with("MS_TOU_10_1600 FinishPre"))
            .map(|t| t.kind)
            .collect();
        assert_eq!(
            finish,
            [
                DrawKind::NpcFrame,
                DrawKind::NpcFrame,
                DrawKind::NpcFrame,
                DrawKind::NpcFrame,
                DrawKind::NpcFrame,
                DrawKind::Counter,
                DrawKind::Counter
            ]
        );
        // Re-evaluating unchanged placements must retain the actors. Replaying
        // just this scene at Night also must not repeat the time lottery.
        let before = l.rng.draws;
        l.skipped_event("MS_TOU_10_1600").unwrap();
        assert_eq!(l.rng.draws - before, 2);
    }
}
