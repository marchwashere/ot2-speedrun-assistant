fn main() {
    let city = ot2_speedrun_assistant::city::after_second_victory(ot2_speedrun_assistant::CrtRng {
        state: 0x41c7db10,
        draws: 5264,
    })
    .unwrap();
    let parlor = ot2_speedrun_assistant::parlor::after_city(&city).unwrap();
    println!("{}", serde_json::to_string_pretty(&parlor).unwrap());
}
