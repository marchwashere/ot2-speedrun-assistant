//! Temenos recruitment after the encounterless Flamechurch approach.
use crate::{Result,ensure,CrtRng,world::{self,Loader,CharacterProgression},snows_escape::Entry};
use serde::Serialize;
#[derive(Debug,Serialize)]pub struct Trip {pub introduction:Entry,pub battle:ot2_battle_engine::temenos_recruitment::Prediction,pub field_return:Entry,pub recruitment:Entry,pub quicksand:crate::galdera_quicksand::Trip}
fn point(l:&mut Loader<'_>,arrival:CrtRng)->Entry {Entry {arrival,after:l.rng,world_after:l.state.clone(),trace:std::mem::take(&mut l.trace),decisions:std::mem::take(&mut l.decisions)}}
pub(crate) fn finish(entry:&Entry)->Result<Trip> {
 ensure(entry.world_after.map=="Twn_Mnt_1_1_A"&&entry.world_after.time==2&&entry.world_after.party==[7,6,8,4],"Temenos recruitment requires daytime Flamechurch with original Galdera party")?;
 let mut l=Loader::new(world::catalog(),entry.world_after.clone(),entry.after)?;
 l.state.sync_progression()?;
 let intro="MS_SIN_00_0100";let victory="MS_SIN_00_0300";
 let request=l.skipped_event_until_battle(intro,&world::EventChoices::from([((intro.into(),59),"eDIALOG_YES".into())]))?;
 ensure(request.volume=="ENM_EVE_CLE_C00_010"&&request.victory_event==victory,"Unexpected Temenos encounter")?;
 let introduction=point(&mut l,entry.after);
 let players:Vec<_>=l.state.party.iter().map(|id|l.state.character_progression[id].character.clone()).collect();
 let battle=ot2_battle_engine::temenos_recruitment::predict(l.rng,&players,&l.state.enemy_knowledge)?;
 for (&id,r) in l.state.party.iter().zip(&battle.party_resources_after){l.state.character_progression.get_mut(&id).unwrap().character.resources=Some(r.clone());}
 l.state.enemy_knowledge=battle.enemy_knowledge.clone();l.state.award_victory(battle.rewards)?;
 for (item,&n) in &battle.drops {*l.state.route_items.entry(item.clone()).or_default()+=n;}
 l.rng=battle.after;l.restore_field_after_battle("Temenos recruitment victory: field restored before recruitment scenes")?;
 let field_return=point(&mut l,battle.after);
 let character=ot2_battle_engine::field_escape::route_player(5,0)?;let growth=ot2_battle_engine::progression::growth(5,0)?.clone();
 l.state.character_progression.insert(5,CharacterProgression {experience:0,jp_earned:0,jp_spent:0,level:growth.level,growth,character,jobs:Default::default()});
 l.skipped_event_with_choices(victory,&world::EventChoices::from([((victory.into(),46),"eDIALOG_YES".into()),((victory.into(),58),"eDIALOG_NO".into()),((victory.into(),68),"8".into())]))?;
 ensure(l.state.party==[7,6,5,4]&&l.state.bench.contains(&8)&&l.state.evasive_maneuvers,"Temenos did not replace Agnea correctly")?;
 let recruitment=point(&mut l,field_return.after);
 let quicksand=crate::galdera_quicksand::finish(&recruitment)?;
 Ok(Trip {introduction,battle,field_return,recruitment,quicksand})
}
