use ot2_battle_engine::{Battle,BattleConfig,ItemCatalog,ActorId,pirro::PirroRoutine};
fn run()->Result<(),Box<dyn std::error::Error>> {
 let mut b=Battle::new_pirro(BattleConfig::throne_pirro(5678)?,ItemCatalog::builtin()?,false)?;
 let mut routine=PirroRoutine::default();
 while b.state().round!=3 || b.current_actor()!=Some(ActorId(0)){routine.step(&mut b)?;}
 let pirro_first=b.state().turn_order().first()==Some(&ActorId(1));
 b.advance_boost_input(0,1)?;b.advance_boost_input(1,2)?;
 let mut voices=String::new();
 for i in 0..48 {
  if i>0{b.advance_boost_input(3,2)?;}
  let mut check=b.state().rng;check.advance(3);
  voices.push(if check.state&0x4000_0000!=0{'N'}else{'S'});
  b.advance_boost_input(2,3)?;
 }
 eprintln!("Pirro first {pirro_first}; transcript {voices}; seed 0x{:08X}",b.state().rng.state&0x7fff_ffff);
 let id=ot2_speedrun_assistant::voice_identification::identify(&voices)?;
 assert_eq!(id["count"],1);assert_eq!(id["candidate"]["current_state"],b.state().rng.state&0x7fff_ffff);
 let value=ot2_speedrun_assistant::identified_route::simulate(&voices,pirro_first,b.state().actors[0].resources.hp)?;
 std::fs::write("outputs/identified-route-check.json",serde_json::to_vec_pretty(&value)?)?;
 println!("Complete route saved: identified seed {}, Pirro after {}",value["initial_seed"],value["state"]["rng"]["draws"]);
 Ok(())
}

fn main(){std::thread::Builder::new().stack_size(16*1024*1024).spawn(||run().unwrap()).unwrap().join().unwrap();}
