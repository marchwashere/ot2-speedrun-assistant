fn main(){std::thread::Builder::new().stack_size(32*1024*1024).spawn(||{
 let voices="NSNNNNSNSNSSSSSSNSSNNSSNNNSNSNSSN";
 let id=ot2_speedrun_assistant::voice_identification::identify_observed(voices,Some((false,197))).unwrap();
 eprintln!("IDENTIFIED {id}");assert_eq!(id["candidate"]["current_state"],0x1e10aa9cu32);
 let result=ot2_speedrun_assistant::identified_route::simulate_route_progress(voices,false,197,true,&mut |message,value|{
  eprintln!("{message}");std::fs::write("target/audit-chubby-1e10aa9c.json",serde_json::to_vec(value).unwrap()).unwrap();
  if message.starts_with("EXP") {return Err(ot2_speedrun_assistant::SimError("Audit captured EXP endpoint".into()));}Ok(())
 });eprintln!("{}",result.err().unwrap());
 }).unwrap().join().unwrap();}
