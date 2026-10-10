//! Audit current carried Cropdale time-switch callbacks, with no fixed offsets.
use ot2_speedrun_assistant::{CrtRng,world::WorldState,winter_voyage,snows_escape::Entry};
use serde::Deserialize;
#[derive(Deserialize)]
struct Snapshot {after:CrtRng,world_after:WorldState}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let response:serde_json::Value=serde_json::from_str(&std::fs::read_to_string("outputs/canalbrine-water-live-4997.json")?)?;
    let snapshot:Snapshot=serde_json::from_value(response.pointer("/prediction/post_battle_field/flats_escape/snows_escape/before_osvald/western/winterbloom/voyage_with_second/destination").unwrap().clone())?;
    let entry=Entry {arrival:snapshot.after,after:snapshot.after,world_after:snapshot.world_after,trace:vec![],decisions:vec![]};
    let reach=winter_voyage::cropdale_reach(&entry.world_after)?.unwrap();
    for count in [0,2,4,6] {
        let route=winter_voyage::enter_cropdale_with_switches(&entry,&reach,count)?;
        println!("{count} switches: recruitment phase {}, departure phase {}, total draws {}, cost {} inputs",
            route.recruitment.world_after.time,route.departure().world_after.time,
            route.departure().after.draws-route.entry.after.draws,route.time_switch_input_cost());
        for (i,switch) in route.time_switches.iter().enumerate() {
            println!("  switch {}: party {:?}, phase {}, {} CRT draws",i+1,switch.world_after.party,
                switch.world_after.time,switch.after.draws-switch.arrival.draws);
        }
    }
    Ok(())
}
