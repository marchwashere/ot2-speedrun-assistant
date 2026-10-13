//! CRT-relevant field callbacks. Data contains candidates and conditions, never
//! precomputed draw totals. The route supplies the world state and event path.
use crate::{CrtRng, Result, SimError, ensure};
use ot2_battle_engine::opening_crt::EncounterCounter;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::OnceLock;
use std::cell::Cell;

thread_local! {
    static QUIET_DEPTH: Cell<u32> = const { Cell::new(0) };
}
pub(crate) fn diagnostics_enabled() -> bool { QUIET_DEPTH.with(|depth| depth.get() == 0) }
/// Search-only diagnostics suppression. RNG callbacks and world mutations still
/// execute normally; callers replay the selected plan with diagnostics enabled.
pub(crate) fn without_diagnostics<T>(f: impl FnOnce() -> T) -> T {
    struct Restore;
    impl Drop for Restore { fn drop(&mut self) { QUIET_DEPTH.with(|d| d.set(d.get()-1)); } }
    QUIET_DEPTH.with(|d| d.set(d.get()+1));
    let _restore=Restore;
    f()
}


#[derive(Debug, Deserialize)]
pub struct Catalog {
    fresh_throne: InitialState,
    pub maps: BTreeMap<String, Map>,
    pub placements: BTreeMap<String, Placement>,
    pub flag_rules: Vec<FlagRule>,
    pub events: BTreeMap<String, Event>,
    pub tutorials: BTreeMap<u32, Tutorial>,
    pub prologue_completions: BTreeMap<u32, Vec<StoryCompletion>>,
    #[serde(default)]
    pub story_starts: BTreeMap<String, StoryStart>,
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
    #[serde(default)]
    pub bar_allowed: Vec<bool>,
    pub environment_draws: Vec<EnvironmentDraw>,
    pub placements: Vec<String>,
    pub counter: EncounterCounter,
    pub bar_event: String,
}
#[derive(Deserialize)]
struct TavernRules {
    no_appearance_rate:u32,
    one_character_rate:u32,
    gift_rate:u32,
    maps:BTreeMap<String,TavernMap>,
}
#[derive(Deserialize)]
struct TavernMap {slots:u32,allowed:Vec<bool>}
fn tavern_rules()-> &'static TavernRules {
    static DATA:OnceLock<TavernRules>=OnceLock::new();
    DATA.get_or_init(||serde_json::from_str(include_str!("../data/taverns.json")).expect("Imported tavern rules"))
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
    GondolaBobbingTime,
    FlagAnimationSpeed,
    LampSwingTime,
}
#[derive(Debug, Deserialize)]
pub struct Placement {
    kind: u8,
    pub(crate) resource: String,
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
#[derive(Deserialize)]
struct StoryPlacementScope { character:i32, maps:Vec<String>, switches:Vec<StoryMapSwitch> }
#[derive(Deserialize)]
struct StoryMapSwitch { begin:u32, end:u32, maps:Vec<String> }
fn story_placement_scopes()-> &'static BTreeMap<u32,StoryPlacementScope> {
    static DATA:OnceLock<BTreeMap<u32,StoryPlacementScope>>=OnceLock::new();
    DATA.get_or_init(||serde_json::from_str(include_str!("../data/story-placement-scopes.json")).expect("Imported story placement scopes"))
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
    pub points: u32,
    #[serde(default)]
    pub on_flag_ranges:Vec<[u32;2]>,
    #[serde(default)]
    pub off_flag_ranges:Vec<[u32;2]>,
}
#[derive(Debug,Deserialize)]
pub struct StoryStart {
    pub scenario:u32,
    pub character_type:i32,
    pub opening_event:String,
    pub release_flags:Vec<u32>,
    pub view_event_flag:u32,
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
    pub(crate) opened_flag: u32,
}

/// Explicit dialog selections keyed by event name and command index.
pub type EventChoices = BTreeMap<(String, usize), String>;

#[derive(Debug,Clone,Serialize)]
pub struct ScriptedBattle {
    pub event:String,
    pub command_index:usize,
    pub volume:String,
    pub victory_event:String,
}
#[derive(Clone,Copy)]
enum EventStop<'a> { Arrival(&'a str), Battle }
enum EventExit { Complete, Arrival, Battle(ScriptedBattle) }

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
    #[serde(default,skip_serializing_if="BTreeSet::is_empty")]
    pub armsmaster_abilities:BTreeSet<String>,
    #[serde(default,skip_serializing_if="BTreeSet::is_empty")]
    pub swooned_route_npcs:BTreeSet<String>,
    #[serde(default,skip_serializing_if="BTreeSet::is_empty")]
    pub opened_route_chests:BTreeSet<String>,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub hired_merchant:Option<HiredMerchant>,
    #[serde(default,skip_serializing_if="BTreeMap::is_empty")]
    pub route_items:BTreeMap<String,u32>,
    #[serde(default)]
    pub galdera_route: bool,
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
    /// Save-game MainProgressPoint: story ClearDifficultyPoint, including skipped prologues.
    #[serde(default)]
    pub main_progress_points: u32,
    #[serde(default)]
    pub completed_scenarios: BTreeSet<u32>,
    /// Save-game confirmation for a selected arrival chapter.
    #[serde(default)]
    pub confirmed_scenarios: BTreeSet<u32>,
    pub prologues: BTreeSet<String>,
    pub resident_levels: BTreeSet<String>,
    pub shared_levels: BTreeSet<String>,
    pub existing_npcs: BTreeSet<String>,
    /// Native saved spawn-delay timers: -1/missing initializes, zero expires.
    pub npc_start_delays: BTreeMap<String, i32>,
    pub npc_end_delays: BTreeMap<String, i32>,
    pub first_encounter: bool,
    pub encounter_counter: u32,
    #[serde(default)]
    pub evasive_maneuvers: bool,
    #[serde(default)]
    pub osvald_extra_skills: u8,
    #[serde(default)]
    pub party_experience: Vec<u32>,
    /// Per-character ledger survives party changes and map loads.
    #[serde(default)]
    pub character_progression: BTreeMap<u32, CharacterProgression>,
    /// Acquired job licenses remain owned while a character uses a subjob.
    #[serde(default,skip_serializing_if="BTreeMap::is_empty")]
    pub job_licenses:BTreeMap<u32,u8>,
    #[serde(default)]
    pub inventor_abilities:BTreeSet<String>,
    #[serde(default)]
    pub enemy_knowledge: ot2_battle_engine::knowledge::EnemyHistory,
    /// Native callback suppression state, false on the supported fresh route.
    pub suppress_flag_callbacks: bool,
    pub suppress_level_lottery: bool,
    pub suppress_time_callbacks: bool,
}
#[derive(Debug,Clone,Serialize,Deserialize,PartialEq,Eq,PartialOrd,Ord,Hash)]
pub struct HiredMerchant {pub npc:String,pub unique_id:u32,pub supporter:String,pub talent:String,pub price:u32}
#[derive(Debug,Clone,Serialize,Deserialize)]
pub struct CharacterProgression {
    pub experience:u32,
    pub jp_earned:u32,
    pub jp_spent:u32,
    pub level:u32,
    pub growth:ot2_battle_engine::progression::Growth,
    pub character:ot2_battle_engine::PlayerCharacter,
    #[serde(default,skip_serializing_if="CharacterJobs::is_empty")]
    pub jobs:CharacterJobs,
}
#[derive(Debug,Clone,Default,PartialEq,Eq,PartialOrd,Ord,Hash,Serialize,Deserialize)]
pub struct CharacterJobs {
    #[serde(default,skip_serializing_if="BTreeSet::is_empty")]
    pub ex_abilities:BTreeSet<String>,
    pub secondary_job:Option<u32>,
    /// Source AbilitySet IDs, including the two automatically granted defaults.
    pub learned_abilities:BTreeMap<u32,BTreeSet<String>>,
    pub unlocked_support:BTreeSet<String>,
    pub equipped_support:BTreeSet<String>,
}
impl CharacterJobs {
    pub fn is_empty(&self)->bool {self.ex_abilities.is_empty() && self.secondary_job.is_none() && self.learned_abilities.is_empty()
        && self.unlocked_support.is_empty() && self.equipped_support.is_empty()}
}
impl WorldState {
    fn complete_story(&mut self, story: &StoryCompletion) {
        if self.completed_scenarios.insert(story.scenario) {
            self.main_progress_points += story.points;
        }
    }
    pub fn sync_progression(&mut self)->Result<()> {
        let _profile=crate::profile::span("sync stats");
        for (i,&id) in self.party.iter().enumerate() {
            let experience=self.party_experience.get(i).copied().unwrap_or(0);
            let growth=ot2_battle_engine::progression::growth(id,experience)?.clone();
            // Starter construction is needed only when a member joins the ledger.
            if let std::collections::btree_map::Entry::Vacant(slot)=self.character_progression.entry(id) {
                let character=ot2_battle_engine::field_escape::route_player(id,experience)?;
                slot.insert(CharacterProgression {experience,jp_earned:0,jp_spent:0,level:growth.level,growth:growth.clone(),character,jobs:Default::default()});
            }
            let p=self.character_progression.get_mut(&id).unwrap();
            ot2_battle_engine::jobs::apply_growth(&mut p.character,id,p.level,growth.level,p.jobs.secondary_job)?;
            p.experience=experience;p.level=growth.level;p.growth=growth;
        }
        Ok(())
    }
    pub fn refresh_support_skills(&mut self) {
        for p in self.character_progression.values_mut() {p.character.boost_start=p.jobs.equipped_support.contains("ABI_MER_SUP_02");}
        self.evasive_maneuvers=self.party.iter().any(|id|self.character_progression.get(id)
            .is_some_and(|p|p.jobs.equipped_support.contains("ABI_SCH_SUP_01")));
    }
    pub fn award_victory(&mut self,reward:ot2_battle_engine::progression::Reward)->Result<()> {
        self.sync_progression()?;
        for &id in &self.party {
            let p=self.character_progression.get_mut(&id).unwrap();
            p.experience+=reward.exp;p.jp_earned+=reward.jp;
            let old_level=p.level;
            p.growth=ot2_battle_engine::progression::growth(id,p.experience)?.clone();p.level=p.growth.level;
            ot2_battle_engine::jobs::apply_growth(&mut p.character,id,old_level,p.level,p.jobs.secondary_job)?;
            if p.level>old_level {let resources=p.character.resources.as_mut().unwrap();resources.hp=p.character.stats.max_hp;resources.sp=p.character.stats.max_sp;}
        }
        self.party_experience=self.party.iter().map(|id|self.character_progression[id].experience).collect();
        Ok(())
    }
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

#[cfg(test)]
mod progression_tests {
    use super::*;
    use ot2_battle_engine::progression::{enemy_reward,Reward};
    #[test]
    fn accompanying_enemies_change_levels_and_stats_and_survive_map_changes() {
        let mut lone=throne_after_second_state().unwrap();
        lone.party=vec![7,6];lone.party_experience=vec![60,0];
        lone.sync_progression().unwrap();
        let mut pack=lone.clone();
        let cait=enemy_reward("ENE_NML_OTR_140").unwrap();
        let marmot=enemy_reward("ENE_NML_FLD_SNW_110").unwrap();
        let yak=enemy_reward("ENE_NML_FLD_SNW_150").unwrap();
        let add_bonus=|r:Reward|Reward {exp:r.exp+r.exp/10,jp:r.jp+r.jp/10,money:r.money};
        lone.award_victory(add_bonus(cait)).unwrap();
        pack.award_victory(add_bonus(Reward {exp:cait.exp+marmot.exp+yak.exp,jp:cait.jp+marmot.jp+yak.jp,money:0})).unwrap();
        assert_eq!(lone.character_progression[&6].level,14);
        assert_eq!(pack.character_progression[&6].level,15);
        assert!(pack.character_progression[&6].character.stats.max_hp>lone.character_progression[&6].character.stats.max_hp);
        let serialized=serde_json::to_string(&pack).unwrap();
        let mut restored:WorldState=serde_json::from_str(&serialized).unwrap();
        restored.map="Fld_Snw_2_2".into();restored.sync_progression().unwrap();
        assert_eq!(restored.character_progression[&6].experience,pack.character_progression[&6].experience);
        assert_eq!(restored.character_progression[&6].jp_earned,pack.character_progression[&6].jp_earned);
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
    MerchantTalent,
    DialogueLottery,
    TavernLottery,
    BattleTiming,
    Boundary,
}

#[derive(Debug, Clone, Serialize)]
pub struct Trace {
    pub reason: String,
    pub kind: DrawKind,
    pub subject: String,
    pub value: Option<u32>,
    pub before: CrtRng,
    pub after: CrtRng,
}
#[derive(Debug, Clone, Serialize)]
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
        if !diagnostics_enabled() { return; }
        self.decisions.push(Decision {
            callback: callback.into(),
            subject: subject.into(),
            result: result.into(),
            rng: self.rng,
        });
    }
    pub fn boundary(&mut self, reason: impl Into<String>) {
        if !diagnostics_enabled() { return; }
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
        if diagnostics_enabled() { self.trace.push(Trace {
            reason: reason.into(),
            kind,
            subject: subject.into(),
            value: Some(value),
            before,
            after: self.rng,
        }); }
        value
    }
    fn draw_lazy(&mut self, kind: DrawKind, subject: &str, reason: impl FnOnce()->String, range: Option<u32>) -> u32 {
        if diagnostics_enabled() { self.draw(kind,subject,reason(),range) }
        else { match range {Some(n)=>self.rng.index(n),None=>u32::from(self.rng.next_u15())} }
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
        let b = self.draw_lazy(
            DrawKind::Counter,
            &subject,
            || format!("{callback}: counter B"),
            Some(b),
        );
        let a = self.draw_lazy(
            DrawKind::Counter,
            &subject,
            || format!("{callback}: counter A"),
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
            let roll = self.draw_lazy(
                DrawKind::FlagLottery,
                &row.id,
                || format!("{trigger}: {}", row.id),
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
    /// Complete the ordinary player Day/Night switch, including its placement
    /// and tavern callbacks. Scripted phase changes still use `set_time` alone.
    pub fn manual_time_switch(&mut self,time:u8,callback:&str)->Result<()> {
        ensure(matches!(time,2|4) && matches!(self.state.time,2|4) && time!=self.state.time,
            "Manual time switch requires changing between stable Day and Night")?;
        self.set_time(time,callback)?;
        // Manual f5cb60 -> ed58b0 refreshes NPC placements, not the tavern.
        // Confirmed by the Winterbloom retail capture of 2026-10-09.
        self.refresh_npcs(&format!("{callback}: time-switch placements"))
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
            // FUN_140ec4900 placement trigger cases4/5 match dawn/dusk only.
            "eMORNING" => self.state.time == 1,
            "eEVENING" => self.state.time == 3,
            other => {
                return Err(SimError(format!(
                    "Unsupported placement time {other}: {label}"
                )));
            }
        };
        if !time_matches {
            return Ok(false);
        }
        if p.story_type != 0 {
            // ec4900 -> f4c660 -> f49fa0 queries the current map's active
            // stories. f2b480 selects the first flag-enabled map override;
            // an empty override falls back to the base RelationLevel list.
            let mut any=false;
            let mut matching=false;
            for scenario in &self.state.active_scenarios {
                let scope=story_placement_scopes().get(scenario)
                    .ok_or_else(||SimError(format!("Missing placement scope for story {scenario}")))?;
                let selected=scope.switches.iter().find(|s|(s.begin>0||s.end>0)
                    && (s.begin==0||self.state.flag(s.begin)) && (s.end==0||!self.state.flag(s.end)));
                let maps=selected.filter(|s|!s.maps.is_empty()).map_or(&scope.maps,|s|&s.maps);
                if maps.contains(&self.state.map) {
                    any=true;matching|=scope.character==p.story_type;
                }
            }
            if (p.story_type<0 && any)||(p.story_type>0 && !matching) {return Ok(false);}
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
            if self.state.hired_merchant.as_ref().is_some_and(|h|h.npc==p.resource) {
                self.state.existing_npcs.remove(label);
                self.note(callback,label,"hired NPC follows Partitio; original placement suppressed");
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
                self.draw_lazy(
                    DrawKind::NpcFrame,
                    label,
                    || format!("{callback}: NPC frame {label}"),
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
        let rules=tavern_rules();
        let name=self.state.map.clone();
        let config=rules.maps.get(&name).ok_or_else(||SimError(format!("Missing tavern rules for {name}")))?;
        let roll=self.draw(DrawKind::TavernLottery,&name,"RefreshBarCharacter: appearance lottery",Some(100));
        if roll<=rules.no_appearance_rate {return Ok(());}
        let slots=if roll<rules.no_appearance_rate+rules.one_character_rate {1}else{2}.min(config.slots);
        let mut eligible:Vec<_>=self.state.bench.iter().copied().filter(|id|
            id.checked_sub(1).and_then(|i|config.allowed.get(i as usize)).copied().unwrap_or(false)).collect();
        for slot in 0..slots {
            let id=if eligible.is_empty() {0}else {
                let index=self.draw(DrawKind::TavernLottery,&name,
                    format!("Tavern slot {}: select benched character",slot+1),Some(eligible.len() as u32));
                eligible.remove(index as usize)
            };
            // ID zero still spawns a placeholder and runs SetupCharacter ->
            // CalcItemData before SetCharacterEnable(false). No visual simulation.
            let gift=self.draw(DrawKind::TavernLottery,&name,
                format!("Tavern slot {}: character {id} item-gift roll (stored as 0..99)",slot+1),Some(100))+1;
            self.note("Tavern setup",&name,format!("slot {}, character {id}, gift eligible = {}",slot+1,gift<=rules.gift_rate));
        }
        Ok(())
    }
    fn initialize_environment(&mut self, map: &Map, levels: &BTreeSet<String>, callback: &str) {
        for init in &map.environment_draws {
            if levels.contains(&init.level) {
                let reason = match init.kind {
                    EnvironmentDrawKind::CanoeBobbingTime => "Canoe initial bobbing time",
                    EnvironmentDrawKind::GondolaBobbingTime => "Gondola initial bobbing time",
                    EnvironmentDrawKind::FlagAnimationSpeed => "Flag initial animation speed",
                    EnvironmentDrawKind::LampSwingTime => "Lamp initial swing time",
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
        self.load_map_requested(destination, false)
    }

    fn load_map_requested(&mut self, destination: &str, force: bool) -> Result<()> {
        let _profile=crate::profile::span("map load");
        let map = self
            .data
            .maps
            .get(destination)
            .ok_or_else(|| SimError(format!("No town-loading data for {destination}")))?;
        if self.state.map == destination && force {
            // PlayEventForce sets EventManager+0x59d. EventLoadLevel therefore
            // requests even the current map. Existing streams/NPCs survive,
            // but both level lifecycle ResetEncountVolume callbacks still run.
            self.state.first_encounter = true;
            self.reset_counter("Proc_OnFinishLoadFieldLevel: forced same-map transition")?;
            self.refresh_npcs("OnFieldLevelShown: forced same-map transition")?;
            self.refresh_bar()?;
            self.state.first_encounter = true;
            self.reset_counter("ResetEncountOnChangeLevel: forced same-map transition")?;
            return Ok(());
        }
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
                self.draw_lazy(
                    DrawKind::StreamedLevel,
                    level,
                    || format!("Create streamed level {level}"),
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
        // ed98a0 queues load+show for field streams but only load for battle
        // streams. A constructed hidden battle level has not run BeginPlay.
        let shown_levels=map.field_streams.iter().filter(|level|created_levels.contains(*level))
            .cloned().collect::<BTreeSet<_>>();
        self.initialize_environment(map, &shown_levels, "");
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
        stop_before: Option<EventStop<'_>>,
    ) -> Result<EventExit> {
        self.run_event_requested(name, bootstrap, depth, choices, stop_before, false)
    }

    fn run_event_requested(
        &mut self, name: &str, bootstrap: bool, depth: usize,
        choices: &EventChoices, stop_before: Option<EventStop<'_>>, force: bool,
    ) -> Result<EventExit> {
        ensure(depth < 16, "Event recursion limit")?;
        let event = self
            .data
            .events
            .get(name)
            .ok_or_else(|| SimError(format!("Missing event {name}")))?;
        if !bootstrap {
            if let Some(story)=self.data.story_starts.values().find(|s|s.opening_event==name) {
                ensure(self.state.confirmed_scenarios.contains(&story.scenario),"Chapter opening requires a confirmed story selection")?;
                // PlayEvent -> UpdateMainStoryProgressOnPlayEvent -> f55a00 /
                // f55560 commits the active first task before EventLoadLevel.
                self.state.active_scenarios.insert(story.scenario);
                self.state.story_types.insert(story.character_type);
                self.state.set_flag(story.view_event_flag,true);
                self.boundary(format!("{name}: chapter started before opening event/map load"));
            }
            if let Some(completion) = &event.completion {
                ensure(
                    completion.scenario == 700
                        && self.state.party == [7]
                        && self.state.bench.is_empty(),
                    "Only fresh solo Throné chapter completion is audited",
                )?;
                // PlayEvent -> UpdateMainMissionProgress -> f55560 -> f4ec50:
                // final normal task sets status 5 before EventLoadLevel.
                self.state.complete_story(completion);
                self.state.active_scenarios.remove(&completion.scenario);
                self.state.story_types.remove(&7);
                self.state.prologues.remove(&completion.character);
                self.boundary(format!("{name}: chapter completed before map load"));
            }
            // MapID=None is an in-place system event, not a load of map zero.
            if event.map_id!=0 {
                let destination = self.map_name(event.map_id)?;
                self.load_map_requested(&destination, force)?;
            }
            if matches!(stop_before,Some(EventStop::Arrival(target)) if target==name) {
                self.boundary(format!("{name}: arrived before event commands"));
                return Ok(EventExit::Arrival);
            }
        }
        let mut pc = 0;
        let mut weapon_master=crate::galdera_gravell::Flow::default();
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
                5100 if name=="MS_SHO_EX1_0110" => {
                    // The two restored weapons sold for250000 before this visit.
                    ensure(c.opt[0]=="100000","Unexpected shipbuilder money threshold")?;
                    // Sufficient money falls through to the next command;
                    // opt1 is the comparison, opt2 the failure jump label.
                }
                1300 | 1310 | 7260 | 7261 if name.starts_with("MS_SHO_EX1_") => {},
                8031 if name.starts_with("MS_SHO_EX1_") && c.target==34037 => {},
                // Fixed ship visuals/UI; default hull/sail are retained. Native
                // placement refreshes, flags, map loads and resets run below.
                4 | 10 | 40 | 55 | 65 | 100 | 106 | 200 | 203 | 210 | 220 | 240 | 300 | 710 | 711 | 8010 | 9100 | 1550 | 1510 | 1511 | 8200 | 8210 | 8220 | 8250 | 8251 if name.starts_with("MS_SHO_EX1_") => {},
                2000 if name=="MS_SHO_EX1_0700" => {
                    let amount=c.opt[1].parse::<i32>().map_err(|_|SimError("Invalid boat item count".into()))?;
                    if amount<0 {self.state.route_items.remove(&c.opt[0]);}else{*self.state.route_items.entry(c.opt[0].clone()).or_default()+=amount as u32;}
                }
                8910 if name=="SHRINE_SHO_0100" && c.target==4 && c.opt==["1"] => {
                    ensure(self.state.party.contains(&4),"Negotiate Schedule requires Partitio in the party")?;
                    self.state.character_progression.get_mut(&4).ok_or_else(||SimError("Missing Partitio progression".into()))?.jobs.ex_abilities.insert("ABI_SET_MER_090".into());
                    self.boundary("Partitio learned Negotiate Schedule (EX); no JP or CRT draw");
                }
                1100 | 200 | 300 | 700 | 10 | 5001 | 220 if name=="SHRINE_SHO_0100" => {}
                8601 if name=="SYS_WPM_GUILD_0010" => weapon_master.start(),
                8602 if name=="SYS_WPM_GUILD_0010" => {
                    let branch=weapon_master.branch(self)?;
                    pc=c.opt.get(branch).ok_or_else(||SimError("Missing Armsmaster branch".into()))?.parse::<usize>().map_err(|_|SimError("Invalid Armsmaster branch".into()))?+1;
                },
                8603 if name=="SYS_WPM_GUILD_0010" => weapon_master.apply(self)?,
                8604 if name=="SYS_WPM_GUILD_0010" => {},
                1300 if name=="SYS_WPM_GUILD_0010" && c.opt==["SS_WLD3_0300"] => {
                    self.boundary("In Search of the Divine Weapons accepted; two of six weapons restored");
                },
                2500 if name=="SYS_WPM_GUILD_0010" => {
                    ensure(c.opt==["NPC_Twn_Wld_3_1_A_TALK_1600_D000"],"Unexpected Armsmaster swoon target")?;
                    self.state.swooned_route_npcs.remove(&c.opt[0]);
                    self.boundary("Armsmaster event cancels Debt Collector swoon");
                },
                40 | 4 | 10 | 55 | 65 | 203 | 210 | 240 | 300 | 710 | 711 | 8010 | 9100 | 7240 if name=="SYS_WPM_GUILD_0010" => {},
                40 if name=="SC_SS_TSnow21_0200_02A0" => {},
                4220 if name=="SC_SS_TSnow21_0200_02A0" && c.index==6 => {
                    let count=self.state.party.len()+usize::from(self.state.hired_merchant.is_some());
                    let needed=c.opt[0].parse::<usize>().map_err(|_|SimError("Invalid Rusty Sword party threshold".into()))?;
                    pc=c.opt[if count>=needed {1}else{2}].parse::<usize>().map_err(|_|SimError("Invalid Rusty Sword party branch".into()))?+1;
                }
                5500 if name=="SC_SS_TSnow21_0200_02A0" && c.index==8 => {
                    ensure(choices.get(&(name.into(),c.index)).map(String::as_str)==Some("eDIALOG_YES"),"Rusty Sword quest requires confirming Pickaxe delivery")?;
                    let item="ITM_TRE_SS_TSn21_0200_0010";
                    let count=self.state.route_items.get_mut(item).ok_or_else(||SimError("Sturdy Pickaxe is missing".into()))?;
                    ensure(*count>0,"Sturdy Pickaxe is missing")?;*count-=1;
                    if *count==0 {self.state.route_items.remove(item);}
                    pc=c.opt[0].parse::<usize>().map_err(|_|SimError("Invalid Rusty Sword delivery branch".into()))?+1;
                    self.boundary("Rusty Sword quest: confirmed Yes; SubDeliveryItem consumes Sturdy Pickaxe");
                }
                1310 if name=="SC_SS_TSnow21_0200_02A0" && c.opt==["SS_SNW2_0200","1"] => {
                    for item in ["ITM_TRE_WPM_01","ITM_CSM_0110"] {*self.state.route_items.entry(item.into()).or_default()+=1;}
                    self.boundary("The Sword in the Stone completed: Rusty Sword, Revitalizing Jam, 7,000 leaves");
                }
                8031 if name=="SC_SS_TSnow21_0200_02A0" && c.target==34155 && c.opt==["1"] => {},
                4|10|55|65|100|106|200|240|300|8010|9100 if name=="SC_SS_TSnow21_0200_02A0" => {},
                600 if name=="SC_SP_Tutorial_0200" && c.index==7 => {
                    ensure(matches!(stop_before,Some(EventStop::Battle)),"Traveler's Bag requires an explicit battle boundary")?;
                    return Ok(EventExit::Battle(ScriptedBattle {event:name.into(),command_index:c.index,volume:c.opt[0].clone(),victory_event:c.opt[1].clone()}));
                }
                5500 if name=="SC_FC13_Tutorial_0400" && c.index==4 => {
                    ensure(self.state.flag(20510),"Al's bag must be recovered before delivery")?;
                    ensure(choices.get(&(name.into(),4)).map(String::as_str)==Some("eDIALOG_YES"),"Al bag delivery requires confirming Yes")?;
                    let item="ITM_TRE_SS_TD11_0100_0010";
                    let count=self.state.route_items.get_mut(item).ok_or_else(||SimError("Al's recovered bag is missing".into()))?;
                    ensure(*count>0,"Al's recovered bag is missing")?;*count-=1;
                    if *count==0 {self.state.route_items.remove(item);}
                    self.boundary("Al: confirmed bag delivery; SubDeliveryItem consumes recovered bag");
                    pc=5;
                }
                2000 if name=="SC_FC13_Tutorial_0310" && c.index==29 && c.opt==["ITM_TRE_SS_TD11_0100_0010","1"] => {
                    *self.state.route_items.entry(c.opt[0].clone()).or_default()+=1;
                    self.boundary("Recovered Al's bag added to inventory");
                }
                1310 if name=="SC_FC13_Tutorial_0400" && c.opt==["SS_TUT_003","1"] => {
                    *self.state.route_items.entry("ITM_CSM_0680".into()).or_default()+=1;
                    *self.state.route_items.entry("ITM_CSM_0010".into()).or_default()+=3;
                    self.boundary("The Traveler's Bag completed: 1,500 leaves, Slippery Nut, Healing Grape x3");
                }
                4|10|20|30|40|55|60|62|65|100|106|200|203|210|220|240|300|310|710|711|8010|9100|2000|5005 if matches!(name,"SC_SP_Tutorial_0200"|"SC_FC13_Tutorial_0310"|"SC_FC13_Tutorial_0400") => {}
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
                    if name.starts_with("SYS_INV_GUILD_") && c.opt[branch].is_empty() {continue;}
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
                    if name=="MS_SIN_00_0300" && c.target==5 {
                        ensure(self.state.party==[7,6,8,4]&&!self.state.bench.contains(&5),"Temenos requires original Galdera party")?;self.state.bench.push(5);
                    } else if name=="MS_KUS_00_0100" && c.target==3 {
                        ensure(self.state.party.len()==4 && !self.state.party.contains(&3) && !self.state.bench.contains(&3),"Castti requires the carried full party")?;
                        self.state.bench.push(3);
                        self.boundary("Castti joined the bench; active party unchanged");
                    } else if name=="MS_KEN_00_0300" && c.target==1 {
                        ensure(self.state.party.len()==4 && !self.state.party.contains(&1)
                            && !self.state.bench.contains(&1),"Hikari join requires the audited full party")?;
                        self.state.bench.push(1);
                        self.boundary("Hikari joined the bench before the full-party replacement dialog");
                    } else if !self.state.party.contains(&(c.target as u32)) {
                        self.state.party.push(c.target as u32);
                    }
                    if name == "MS_GAK_00_0100" {
                        self.boundary("Osvald joined the party; fixed join dialog dismissed");
                    }
                }
                3010 | 3120 if (name == "MS_GAK_00_0100" && c.target == 6) || (name == "MS_ODO_00_0200" && c.target == 8) || (name == "MS_SHO_00_0100" && c.target == 4) || (name=="MS_KEN_00_0300" && c.target==1) || (name=="MS_KUS_00_0100" && c.target==3) || (name=="MS_SIN_00_0300" && c.target==5) => {
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
                        "Playing a recruited character's prologue is not supported by this route",
                    )?;
                    if c.cmd == 3120 && branch == 1 {
                        // AddReminiscenceDialogFinishCallback -> SkipPrologueReminiscence
                        // f15190 -> f54e00 -> f4ec50 -> AddMainProgressPoint.
                        // Completing skipped prologues changes encounter tiers, without RNG.
                        for story in &self.data.prologue_completions[&(c.target as u32)] {
                            // SkipPrologueReminiscence -> f4ec50 -> f537f0 applies
                            // the chapter's saved event flags, not only progress.
                            for &[begin,end] in &story.on_flag_ranges {
                                for flag in begin..=end {self.state.set_flag(flag,true);}
                            }
                            for &[begin,end] in &story.off_flag_ranges {
                                for flag in begin..=end {self.state.set_flag(flag,false);}
                            }
                            self.state.complete_story(story);
                            self.state.active_scenarios.remove(&story.scenario);
                        }
                        self.state.story_types.remove(&c.target);
                        placements_dirty=true;
                        self.boundary(format!("{name}: prologue skipped; main progress {}", self.state.main_progress_points));
                    }
                    pc = c.opt[branch]
                        .parse::<usize>()
                        .map_err(|_| SimError("Invalid recruitment branch".into()))?
                        + 1;
                    self.boundary(format!("{name}:{} select {selected}", c.index));
                }
                3005 if (name == "MS_GAK_00_0100" && c.target == 6) || (name == "MS_ODO_00_0200" && c.target == 8) || (name == "MS_SHO_00_0100" && c.target == 4) => {
                    ensure(
                        self.state.party.contains(&(c.target as u32)) && !self.state.flag(2900),
                        "Only immediate recruitment without a prologue is audited",
                    )?;
                    // StartAddMemberEnd finds him already in the active party;
                    // EndPrologueReminiscence returns early without flag 0xb54.
                    self.boundary(format!("Character {} retained in active party; no party-change dialog",c.target));
                }
                3005 if name=="MS_KUS_00_0100" && c.target==3 => {
                    ensure(!self.state.flag(2900) && self.state.bench.contains(&3),"Castti must remain benched after Chapter 1 skip")?;
                    self.refresh_bar()?;
                    self.boundary("Castti retained on bench; party selection dismissed");
                }
                3005 if name=="MS_SIN_00_0300" && c.target==5 => {
                    ensure(!self.state.flag(2900)&&self.state.party==[7,6,8,4]&&self.state.bench.contains(&5),"Temenos full-party replacement boundary")?;
                    ensure(choices.get(&(name.into(),c.index)).map(String::as_str)==Some("8"),"Temenos must replace Agnea")?;
                    self.state.sync_progression()?;
                    let i=self.state.bench.iter().position(|&id|id==5).unwrap();self.state.bench[i]=8;self.state.party[2]=5;
                    self.state.party_experience=self.state.party.iter().map(|id|self.state.character_progression[id].experience).collect();
                    self.state.refresh_support_skills();self.refresh_bar()?;
                    self.boundary("Temenos replaces Agnea; Agnea progression retained on bench");
                }
                3005 if name=="MS_KEN_00_0300" && c.target==1 => {
                    ensure(!self.state.flag(2900) && self.state.party==[7,6,8,4]
                        && self.state.bench.contains(&1),"Hikari replacement requires the carried four-member party")?;
                    let selection=choices.get(&(name.into(),c.index)).map(String::as_str);
                    ensure(selection==Some(if self.state.galdera_route {"bench"}else{"6"}),
                        "Hikari recruitment requires the selected route's explicit party choice")?;
                    self.state.sync_progression()?;
                    ensure(self.state.character_progression.contains_key(&1),"Hikari starting progression was not initialized")?;
                    if self.state.galdera_route {
                        // Dismiss the same full-party selector used for Castti;
                        // no main-member swap or support rebuild occurs.
                        self.boundary("Hikari retained on bench; party selection dismissed; Osvald remains active");
                    } else {
                    let bench_index=self.state.bench.iter().position(|&id|id==1).unwrap();
                    self.state.party[1]=1;self.state.bench[bench_index]=6;
                    self.state.party_experience=self.state.party.iter().map(|id|self.state.character_progression[id].experience).collect();
                    // RefreshMainMember -> RefreshPartyAfterChangeMember rebuilds
                    // equipped support skills from active battle members only.
                    self.state.refresh_support_skills();
                    self.boundary("Party replacement: Hikari replaces Osvald; Osvald's ledger retained on bench; Evasive Maneuvers inactive");
                    }
                    // Add-friend mode refreshes tavern placement rather than
                    // respawning ordinary field NPCs. Ryu has BarEvent=None.
                    self.refresh_bar()?;
                }
                7410 if matches!(name,"MS_GAK_00_0100"|"MS_ODO_00_0200"|"MS_SHO_00_0100"|"MS_KEN_00_0300"|"MS_KUS_00_0100"|"MS_SIN_00_0300") => {
                    let amount = c.opt[0]
                        .parse::<i32>()
                        .map_err(|_| SimError("Invalid event variable increment".into()))?;
                    *self.state.event_variables.entry(c.target).or_default() += amount;
                }
                7420 if matches!(name,"MS_GAK_00_0100"|"MS_ODO_00_0200"|"MS_SHO_00_0100"|"MS_KEN_00_0300"|"MS_KUS_00_0100"|"MS_SIN_00_0300") => {
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
                        match self.run_event(&c.opt[0], false, depth + 1, choices, stop_before)? {
                            EventExit::Complete=>{}, stopped=>return Ok(stopped),
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
                40 if ((matches!(name,"SYS_SCH_GUILD_0010"|"SYS_SCH_GUILD_0030") && c.target==5105 && c.text.starts_with("TX_SYS_GUILD_SCH_"))
                    || (matches!(name,"SYS_WAR_GUILD_0010"|"SYS_WAR_GUILD_0030") && c.target==5107 && c.text.starts_with("TX_SYS_GUILD_WAR_"))
                    || (matches!(name,"SYS_DAN_GUILD_0010"|"SYS_DAN_GUILD_0030") && c.target==5102 && c.text.starts_with("TX_SYS_GUILD_DAN_"))
                    || (matches!(name,"SYS_MER_GUILD_0010"|"SYS_MER_GUILD_0030") && c.target==5104 && (c.text.starts_with("TX_SYS_GUILD_MER_")||c.text.starts_with("TX_SYS_GUILD_MAR_")))
                    || (name=="MapInStorySelect" && c.target==0 && c.text=="TX_SYS_MapInStorySelect_0010"))
                    && c.opt == ["1","0","0","0"] => {
                    if !bootstrap {self.boundary(format!("{name}: fixed dialogue {} completed",c.text));}
                }
                7110 if name=="MapInStorySelect" => {
                    if !bootstrap {
                        if let Some(selected)=choices.get(&(name.into(),c.index)).filter(|_|!self.state.galdera_route) {
                            let story=self.data.story_starts.get(selected).ok_or_else(||SimError(format!("Unaudited arrival story {selected}")))?;
                            ensure(story.release_flags.iter().all(|&flag|flag==0||self.state.flag(flag)),"Arrival chapter is not released")?;
                            ensure(!self.state.completed_scenarios.contains(&story.scenario),"Arrival chapter is already complete")?;
                            self.state.confirmed_scenarios.insert(story.scenario);
                            self.boundary(format!("Arrival story selection: confirmed {selected}"));
                            // BarMenuStorySelect.OnClosedConfirmDialog calls PlayEventForce,
                            // not ordinary PlayEvent. Preserve that distinction through loading.
                            match self.run_event_requested(&story.opening_event,false,depth+1,choices,stop_before,true)? {
                                EventExit::Complete=>{},stopped=>return Ok(stopped),
                            }
                        } else {
                            self.boundary("Arrival story selection closed without starting a chapter");
                        }
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
                // Agnea's cooked introduction uses fixed lines and event-only
                // actors. Camera/movement/animation commands have no CRT lottery.
                40 if name=="MS_ODO_00_0200" && c.text.starts_with("TX_PCJ_ODO_0000_")
                    && c.opt==["1","0","0","0"] => {}
                10 | 20 | 21 | 30 | 200 | 210 | 240 | 300 | 400 | 710 | 711 | 8010 | 9100
                    if name=="MS_ODO_00_0200" => {}
                2000 if name=="MS_ODO_00_0200" && c.opt==["ITM_FLV_0480","-1"] => {
                    self.boundary("Agnea introduction: lost purse returned (no CRT draw)");
                }
                7450 if name=="MS_SHO_00_0100" => {
                    // StartTopCharaTypeLabelJump uses the top member's first job
                    // for a fixed greeting. Throné's thief job selects opt index6.
                    ensure(self.state.party.first()==Some(&7) && c.opt.len()==8,
                        "Partitio greeting is audited for a Throné-start route")?;
                    pc=c.opt[6].parse::<usize>().map_err(|_|SimError("Invalid protagonist branch".into()))?+1;
                }
                40 if name=="MS_SHO_00_0100" && c.text.starts_with("TX_PCJ_SHO_0000_")
                    && c.opt==["1","0","0","0"] => {}
                4 | 10 | 20 | 30 | 60 | 106 | 200 | 210 | 240 | 300 | 710 | 711 | 8010 | 9100
                    if name=="MS_SHO_00_0100" => {}
                40 if matches!(name,"MS_SIN_00_0100"|"MS_SIN_00_0300") && c.text.starts_with("TX_PCJ_SIN_0000_") && c.opt==["1","0","0","0"] => {}
                40 if matches!(name,"MS_KEN_00_0100"|"MS_KEN_00_0300")
                    && c.text.starts_with("TX_PCJ_KEN_0000_") && c.opt==["1","0","0","0"] => {}
                // 55/65 are the source widget's fixed fade-in/fade-out. They
                // have no CRT lottery or field-state/placement side effects.
                4 | 10 | 20 | 30 | 55 | 60 | 65 | 106 | 200 | 210 | 240 | 300 | 400 | 710 | 711 | 8010 | 9100
                    if matches!(name,"MS_KEN_00_0100"|"MS_KEN_00_0300"|"MS_KUS_00_0100"|"MS_SIN_00_0100"|"MS_SIN_00_0300") => {}
                600 if name=="EV_Dng_Dst_2_2_0010" && c.index==1 && c.opt==["EVM_DNG_DST_2_2_L","EV_Dng_Dst_2_2_0020","0","1","0"] => {
                    ensure(matches!(stop_before,Some(EventStop::Battle)),"Lord Sands requires explicit battle boundary")?;
                    return Ok(EventExit::Battle(ScriptedBattle {event:name.into(),command_index:c.index,volume:c.opt[0].clone(),victory_event:c.opt[1].clone()}));
                }
                600 if name=="MS_SIN_00_0100" && c.index==82 && c.opt==["ENM_EVE_CLE_C00_010","MS_SIN_00_0300","0","1","0"] => {
                    ensure(matches!(stop_before,Some(EventStop::Battle)),"Temenos battle requires explicit boundary")?;
                    return Ok(EventExit::Battle(ScriptedBattle {event:name.into(),command_index:c.index,volume:c.opt[0].clone(),victory_event:c.opt[1].clone()}));
                }
                600 if name=="MS_KEN_00_0100" && c.index==66
                    && c.target==0 && c.opt==["ENM_EVE_WAR_C00_010","MS_KEN_00_0300","0","1","0"] => {
                    ensure(matches!(stop_before,Some(EventStop::Battle)),"Scripted battle requires an explicit battle boundary")?;
                    self.boundary("Hikari introduction: scripted battle requested before battle initialization");
                    return Ok(EventExit::Battle(ScriptedBattle {event:name.into(),command_index:c.index,
                        volume:c.opt[0].clone(),victory_event:c.opt[1].clone()}));
                }
                10000 => break,
                8610 if matches!(name,"SYS_INV_GUILD_0C10"|"SYS_INV_GUILD_0D10") => {
                    // Inventor dialog Yes, after verifying materials in retail:
                    // EventManagerBP.UpdateInventorFrow uses opt[2] as EventIndex.
                    pc=c.opt[2].parse::<usize>().map_err(|_|SimError("Invalid invention branch".into()))?+1;
                    self.boundary("Invention materials delivered; assumes required inventory available");
                }
                8620 if matches!(name,"SYS_INV_GUILD_0C10"|"SYS_INV_GUILD_0D10") => {
                    self.state.inventor_abilities.insert(if c.opt[0]=="INVENTION_ITEM_03" {"ABI_SET_INV_030"}else{"ABI_SET_INV_040"}.into());
                    self.boundary(format!("{} crafted and ability unlocked",c.opt[0]));
                }
                2000 if name=="SYS_INV_GUILD_0010" => {
                    if c.opt[0]=="ITM_EQP_JOB_0011" {
                        self.state.job_licenses.insert(11,1);
                        self.state.inventor_abilities.extend(["ABI_SET_INV_010".into(),"ABI_SET_INV_020".into()]);
                    }
                }
                40 if matches!(name,"SYS_INV_GUILD_0010"|"SYS_INV_GUILD_0C10"|"SYS_INV_GUILD_0D10") && c.text.starts_with("TX_SS_FC13_") => {}
                4 | 10 | 20 | 30 | 55 | 60 | 65 | 100 | 200 | 210 | 240 | 300 | 710 | 711 | 8010 | 9100 | 5006 | 7240 | 1300 | 2000
                    if matches!(name,"SYS_INV_GUILD_0010"|"SYS_INV_GUILD_0C10"|"SYS_INV_GUILD_0D10") => {}
                5500 if name=="MS_KUS_00_0100" && c.index==32 => {
                    ensure(choices.get(&(name.into(),c.index)).map(String::as_str)==Some("eDIALOG_YES"),"Castti requires delivery of one Healing Grape")?;
                    pc=c.opt[0].parse::<usize>().map_err(|_|SimError("Invalid delivery branch".into()))?+1;
                    self.boundary("Healing Grape delivered to Castti");
                }
                40 if name=="MS_KUS_00_0100" && c.text.starts_with("TX_PCJ_KUS_0000_") && c.opt==["1","0","0","0"] => {}
                4 | 10 | 20 | 30 | 55 | 60 | 65 | 106 | 200 | 210 | 240 | 250 | 300 | 710 | 711 | 8010 | 9100
                    if name=="MS_KUS_00_0100" => {}
                // 8031 temporarily enables existing EventFlagObjects (SetupEnable);
                // the persistent flag is written by command70 later in this script.
                8031 if name=="MS_ODO_20_0100" && c.target==34167 && c.opt==["1"] => {}
                // Partitio Ch2: temporary visibility of existing EventFlagObjects.
                8031 if matches!(name,"MS_SHO_20_0110"|"MS_SHO_20_0200"|"MS_SHO_20_0410"|"MS_SHO_20_0500"|"MS_SHO_20_0620")
                    && matches!(c.target,34024|34025|34051|34052) => {}
                // In-place staging/dialogue for the two short linked scenes.
                4 | 10 | 13 | 14 | 20 | 25 | 40 | 200 | 220 | 240 | 300
                    if matches!(name,"MS_SHO_20_0410"|"MS_SHO_20_0600") => {}
                2000 | 5005 if name=="MS_SHO_20_0610" && c.opt==["ITM_TRE_SHO_20_0010","5"] => {
                    self.boundary("Clockite x5 story reward / item popup; no CRT draw");
                }
                40 if name=="MS_SHO_20_0615" => {}
                5500 if name=="MS_SHO_20_0615" && c.index==2 => {
                    ensure(self.state.flag(6058),"Clockite delivery requires the weaponsmith reward")?;
                    ensure(choices.get(&(name.into(),c.index)).map(String::as_str)==Some("eDIALOG_YES"),"Clockite delivery requires Yes")?;
                    pc=c.opt[0].parse::<usize>().map_err(|_|SimError("Invalid Clockite delivery branch".into()))?+1;
                }
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
        Ok(EventExit::Complete)
    }
    pub fn skipped_event(&mut self, name: &str) -> Result<()> {
        self.skipped_event_with_choices(name, &EventChoices::new())
    }
    pub fn skipped_event_with_choices(&mut self, name: &str, choices: &EventChoices) -> Result<()> {
        let end=self.run_event(name,false,0,choices,None)?;
        ensure(matches!(end,EventExit::Complete),"Event did not complete")
    }
    /// Preserve the introduction's real command path, stopping before native
    /// battle initialization. The caller executes that battle in the core.
    pub fn skipped_event_until_battle(&mut self,name:&str,choices:&EventChoices)->Result<ScriptedBattle> {
        match self.run_event(name,false,0,choices,Some(EventStop::Battle))? {
            EventExit::Battle(battle)=>Ok(battle),
            _=>Err(SimError(format!("Event {name} did not reach its scripted battle"))),
        }
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
            Some(EventStop::Arrival(destination_event)),
        )?;
        ensure(
            matches!(arrived,EventExit::Arrival),
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
        armsmaster_abilities:BTreeSet::new(),swooned_route_npcs:BTreeSet::new(),
        hired_merchant:None,route_items:BTreeMap::new(),opened_route_chests:BTreeSet::new(),
        galdera_route: false,
        map: p.map.clone(),
        visited_maps: BTreeSet::new(),
        time: p.time,
        flags: BTreeSet::new(),
        event_variables: BTreeMap::new(),
        party: p.party.clone(),
        bench: p.bench.clone(),
        story_types: p.story_types.clone(),
        active_scenarios: p.active_scenarios.clone(),
        main_progress_points: 0,
        completed_scenarios: BTreeSet::new(),
        confirmed_scenarios: BTreeSet::new(),
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
        evasive_maneuvers: false,
        osvald_extra_skills: 0,
        party_experience: vec![],
        character_progression: BTreeMap::new(),
        job_licenses:BTreeMap::new(),
        inventor_abilities:BTreeSet::new(),
        enemy_knowledge: BTreeMap::new(),
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
    #[test]
    fn placement_story_filter_is_local_to_the_map() {
        let f:serde_json::Value=serde_json::from_str(include_str!("../data/source/live-hikari-canalbrine-20261010.json")).unwrap();
        let mut w:WorldState=serde_json::from_value(f["world_before"].clone()).unwrap();
        w.active_scenarios=BTreeSet::from([701]);w.story_types=BTreeSet::from([7]);
        // No excluded soldier labels: isolate the native story gate itself.
        let labels=BTreeSet::new();
        let label="NPC_Twn_Dst_1_1_A_TALK_0300_D000";
        let mut l=Loader::new(catalog(),w,CrtRng::new(0)).unwrap();
        assert!(l.placement_enabled(label,&labels,&mut Vec::new()).unwrap(),"Mother chapter in Oresrush is unrelated to Ryu");
        l.state.active_scenarios.insert(100);l.state.story_types.insert(1);
        assert!(!l.placement_enabled(label,&labels,&mut Vec::new()).unwrap(),"Active Hikari prologue is related to Ryu");
    }
    #[test]
    fn retail_conning_load_20261009() {
        let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/live-conning-load-20261009.json")).unwrap();
        for revisit in [false,true] {
            let mut state:WorldState=serde_json::from_value(fixture["world_before"].clone()).unwrap();
            if !revisit {state.visited_maps.remove("Twn_Sea_2_1_A");}
            let rng=CrtRng {state:fixture["initial_state"].as_u64().unwrap() as u32,draws:19287194};
            let mut loader=Loader::new(catalog(),state,rng).unwrap();
            loader.visit_map("Twn_Sea_2_1_A").unwrap();
            assert_eq!(loader.rng.draws,19287273);
            assert_eq!(loader.rng.state,fixture["final_state"].as_u64().unwrap() as u32);
            let draws:Vec<_>=loader.trace.iter().filter(|t|t.after.draws>t.before.draws).collect();
            let records=fixture["records"].as_array().unwrap();
            assert_eq!(draws.len(),records.len());
            for (actual,expected) in draws.iter().zip(records) {
                assert_eq!(actual.before.draws,expected["position"].as_u64().unwrap());
                assert_eq!(actual.before.state,expected["before"].as_u64().unwrap() as u32);
                if let Some(npc)=expected["npc"].as_str() {assert_eq!(actual.subject,npc);}
                let kind=match expected["node"].as_str().unwrap() {
                    "StreamedLevel"=>DrawKind::StreamedLevel,
                    "CalcEncountStepCount"=>DrawKind::Counter,
                    "RefreshBarCharacter"|"CalcItemData"=>DrawKind::TavernLottery,
                    _ if expected["npc"].is_string()=>DrawKind::NpcFrame,
                    _=>DrawKind::EnvironmentInitialization,
                };
                assert_eq!(actual.kind,kind,"at {}",actual.before.draws);
            }
            assert_eq!(draws.iter().filter(|t|t.kind==DrawKind::StreamedLevel).count(),6);
            assert_eq!(draws.iter().filter(|t|t.kind==DrawKind::EnvironmentInitialization).count(),33);
            assert_eq!(draws.iter().filter(|t|t.kind==DrawKind::TavernLottery).count(),3);
        }
    }
    #[test]
    fn forced_same_map_load_runs_lifecycle_without_recreating_streams() {
        let fixture:serde_json::Value=serde_json::from_str(include_str!("../data/source/live-oresrush-mother-20261009.json")).unwrap();
        let mut state:WorldState=serde_json::from_value(fixture["world_before"].clone()).unwrap();
        state.first_encounter=false;
        let levels=state.resident_levels.clone();
        let npcs=state.existing_npcs.clone();
        let rng=CrtRng::new(0);
        let mut l=Loader::new(catalog(),state,rng).unwrap();
        l.load_map("Twn_Wld_1_1_C").unwrap();
        assert_eq!(l.rng,rng);
        assert!(!l.state.first_encounter);
        l.load_map_requested("Twn_Wld_1_1_C",true).unwrap();
        assert_eq!(l.rng.draws,4);
        assert!(l.state.first_encounter);
        assert_eq!(l.state.resident_levels,levels);
        assert_eq!(l.state.existing_npcs,npcs);
        assert!(l.state.encounter_counter<900);
    }
    #[test]
    fn placement_refreshes_do_not_run_the_map_load_tavern_lottery() {
        for map in tavern_rules().maps.keys().filter(|m|catalog().maps.contains_key(*m)) {
            let mut state=throne_after_second_state().unwrap();
            state.map=map.clone();state.time=2;state.bench=vec![1,3,4];
            let mut loader=Loader::new(catalog(),state,CrtRng::new(17)).unwrap();
            loader.refresh_npcs("fixture initial placements").unwrap();loader.trace.clear();
            let counter=loader.state.encounter_counter;
            loader.manual_time_switch(4,"test night").unwrap();
            loader.manual_time_switch(2,"test day").unwrap();
            loader.set_time(3,"test dusk").unwrap();
            assert_eq!(loader.state.encounter_counter,counter);
            assert!(!loader.trace.iter().any(|t|matches!(t.kind,DrawKind::TavernLottery)),"{map}");
        }
    }
    #[test]
    fn tavern_draws_include_placeholders_and_selection_without_replacement() {
        for (map,capacity,bench) in [
            ("Twn_Snw_2_1_A",1,vec![]),
            ("Twn_Snw_2_1_A",1,vec![6]),
            ("Twn_Sea_2_1_A",2,vec![6]), // Osvald excluded here
            ("Twn_Sea_2_1_A",2,vec![1]),
            ("Twn_Sea_2_1_A",2,vec![1,3,4]),
        ] {
            let mut seen=[false;3];
            for sample in 0..1024u32 {
                let seed=sample.wrapping_mul(0x9e3779b9);
                let mut state=throne_after_second_state().unwrap();
                state.map=map.into();state.bench=bench.clone();
                let start=CrtRng::new(seed);
                let mut oracle=start;
                let roll=oracle.index(100);
                let requested=if roll<=50 {0}else if roll<75 {1}else{2};
                seen[requested]=true;
                let slots=requested.min(capacity);
                let mut candidates:Vec<_>=bench.iter().copied().filter(|id|tavern_rules().maps[map].allowed[(*id-1) as usize]).collect();
                let mut chosen=vec![];
                for _ in 0..slots {
                    if !candidates.is_empty() {let i=oracle.index(candidates.len() as u32);chosen.push(candidates.remove(i as usize));}
                    oracle.index(100);
                }
                let mut loader=Loader::new(catalog(),state.clone(),start).unwrap();
                loader.refresh_bar().unwrap();
                assert_eq!(loader.rng,oracle,"{map}, seed {seed}, bench {bench:?}");
                let mut quiet=Loader::new(catalog(),state,start).unwrap();
                without_diagnostics(||quiet.refresh_bar()).unwrap();
                assert_eq!(quiet.rng,oracle);
                assert_eq!(loader.state.bench,bench);
                let unique:BTreeSet<_>=chosen.iter().collect();assert_eq!(unique.len(),chosen.len());
            }
            assert!(seen.into_iter().all(|x|x));
        }
        let mut state=throne_after_second_state().unwrap();state.map="Twn_Wld_1_1_C".into();state.bench=vec![1,3];
        let mut loader=Loader::new(catalog(),state,CrtRng::new(42)).unwrap();
        loader.refresh_bar().unwrap();assert_eq!(loader.rng.draws,0);
        loader.state.map="Twn_Snw_2_1_A".into();loader.state.set_flag(2008,true);loader.state.set_flag(2009,false);
        loader.refresh_bar().unwrap();assert_eq!(loader.rng.draws,0);
    }
    #[test]
    fn quiet_loader_preserves_full_state_and_rng() {
        for variant in 0..8 {
            let mut state=throne_after_second_state().unwrap();
            match variant {
                1=>state.time=2,
                2=>{state.set_flag(20313,true);},
                3=>state.shared_levels.clear(),
                4=>state.suppress_level_lottery=true,
                5=>{state.party=vec![7,6];state.party_experience=vec![60,0];state.sync_progression().unwrap();},
                6=>{state.enemy_knowledge.insert("ENE_NML_FLD_CTY_010".into(),ot2_battle_engine::knowledge::EnemyKnowledge {weapons:1,elements:2,studied:true});},
                7=>{state.npc_start_delays.insert("unused".into(),3);},
                _=>{}
            }
            let rng=CrtRng {state:123456,draws:200};
            let mut oracle=Loader::new(catalog(),state.clone(),rng).unwrap();
            oracle.load_map("Twn_Cty_1_1_B").unwrap();
            let mut quiet=Loader::new(catalog(),state,rng).unwrap();
            without_diagnostics(|| quiet.load_map("Twn_Cty_1_1_B")).unwrap();
            assert_eq!(quiet.rng,oracle.rng);
            assert_eq!(serde_json::to_value(&quiet.state).unwrap(),serde_json::to_value(&oracle.state).unwrap());
            assert!(quiet.trace.is_empty() && quiet.decisions.is_empty());
        }
    }
    #[test]
    fn quiet_scope_restores_logging_on_error_and_unwind() {
        without_diagnostics(|| without_diagnostics(|| assert!(!diagnostics_enabled())));
        assert!(diagnostics_enabled());
        let _=std::panic::catch_unwind(|| without_diagnostics(|| panic!("test unwind")));
        assert!(diagnostics_enabled());
        let result:Result<()>=without_diagnostics(|| Err(SimError("test error".into())));
        assert!(result.is_err());assert!(diagnostics_enabled());
    }
    fn loader() -> Loader<'static> {
        Loader::new(
            catalog(),
            throne_after_second_state().unwrap(),
            CrtRng::new(0),
        )
        .unwrap()
    }
    #[test]
    fn galdera_discards_arrival_chapter_choices_without_changing_rng_or_story() {
        let mut l=loader();l.state.galdera_route=true;
        let before=l.rng;let active=l.state.active_scenarios.clone();let confirmed=l.state.confirmed_scenarios.clone();
        l.skipped_event_with_choices("MapInStorySelect",&EventChoices::from([(("MapInStorySelect".into(),1),"MS_ODO_01".into())])).unwrap();
        assert_eq!(l.rng,before);assert_eq!(l.state.active_scenarios,active);assert_eq!(l.state.confirmed_scenarios,confirmed);
    }

    #[test]
    fn skipped_prologues_add_source_points_once_without_rng() {
        let mut l = loader();
        assert_eq!(l.state.main_progress_points, 0);
        let throne = &catalog().events["MS_TOU_10_2000"].completion.as_ref().unwrap();
        l.state.complete_story(throne);
        assert_eq!(l.state.main_progress_points, 1);
        l.state.complete_story(throne);
        assert_eq!(l.state.main_progress_points, 1);
        l.state.time = 2;
        let choices = EventChoices::from([
            (("MS_GAK_00_0100".into(),15),"eDIALOG_YES".into()),
            (("MS_GAK_00_0100".into(),61),"eDIALOG_YES".into()),
            (("MS_GAK_00_0100".into(),70),"eDIALOG_NO".into()),
        ]);
        l.skipped_event_with_choices("MS_GAK_00_0100", &choices).unwrap();
        assert_eq!(l.state.main_progress_points, 2);
        let choices = EventChoices::from([
            (("MS_ODO_00_0200".into(),125),"eDIALOG_YES".into()),
            (("MS_ODO_00_0200".into(),134),"eDIALOG_NO".into()),
        ]);
        l.skipped_event_with_choices("MS_ODO_00_0200", &choices).unwrap();
        assert_eq!(l.state.main_progress_points, 3);
        assert_eq!(l.state.completed_scenarios, BTreeSet::from([600,601,700,800]));
        let rng = l.rng;
        l.state.complete_story(&catalog().prologue_completions[&8][0]);
        assert_eq!(l.state.main_progress_points, 3);
        assert_eq!(l.rng, rng);
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
