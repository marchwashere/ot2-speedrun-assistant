use ot2_speedrun_assistant::{
    CrtRng,
    encounter::{Boundary, PredictionOptions, catalog},
};

#[test]
fn underground_waterway_progression_selects_different_first_encounters() {
    let mut options = ot2_speedrun_assistant::route::throne_field_profile()
        .options
        .clone();
    options.current_counter = Some(549);
    for (progress, table, enemy, count) in [
        (0, "ENC_DNG_CTY_3_1_LND_DAY_010", "ENE_EVE_THI_CTY_010", 2),
        (1, "ENC_DNG_CTY_3_1_LND_DAY_020", "ENE_NML_FLD_CTY_010", 1),
    ] {
        options.progress = progress;
        let p = catalog()
            .predict(&mut CrtRng::new(0x194611CD), &options)
            .unwrap();
        assert_eq!(p.encounter_table.as_deref(), Some(table));
        assert_eq!(p.enemies.len(), count);
        assert!(p.enemies.iter().all(|e| e.enemy.id == enemy));
        assert!(p.first_override);
        assert_eq!(p.draws, 6);
    }
    options.dash = true;
    let p = catalog()
        .predict(&mut CrtRng::new(0x194611CD), &options)
        .unwrap();
    assert_eq!((p.decrement, p.counter_updates), (Some(16), Some(35)));
}

#[test]
fn live_cait_capture_matches_every_selection_checkpoint() {
    let recorded: serde_json::Value = serde_json::from_str(include_str!(
        "../data/source/live-deserted-highroad-4940.json"
    ))
    .unwrap();
    let mut rng = CrtRng::new(0);
    rng.advance(4940);
    let o = PredictionOptions {
        boundary: Boundary::Trigger,
        party_levels: vec![84, 85, 77, 84],
        cait_bonus: 2,
        rare_support_bonus: 2,
        ..Default::default()
    };
    let p = catalog().predict(&mut rng, &o).unwrap();
    assert_eq!(p.group_roll, Some(16));
    assert_eq!(p.situation_roll, Some(3));
    assert_eq!(p.situation.as_deref(), Some("Preemptive"));
    assert_eq!(p.rare_family.as_deref(), Some("Cait"));
    assert_eq!(
        p.enemies
            .iter()
            .map(|e| e.enemy.id.as_str())
            .collect::<Vec<_>>(),
        vec!["ENE_NML_OTR_150", "ENE_NML_FLD_CTY_090"]
    );
    assert_eq!(
        (p.counter.roll_b, p.counter.roll_a, p.counter.value),
        (304, 167, 1471)
    );
    let states: Vec<_> = p
        .checkpoints
        .iter()
        .map(|c| serde_json::json!([c.draws, c.state]))
        .collect();
    assert_eq!(serde_json::json!(states), recorded["checkpoints"]);
    assert_eq!(rng.draws, 4951);
    assert_eq!(rng.state, 2246832313);
}

#[test]
fn walk_and_existing_counter_match_explicit_reset_then_trigger() {
    let mut o = PredictionOptions {
        intervening_advances: "37".into(),
        ..Default::default()
    };
    let mut all = CrtRng::new(123);
    let p = catalog().predict(&mut all, &o).unwrap();
    let mut sequential = CrtRng::new(123);
    let initial = catalog()
        .predict(
            &mut sequential,
            &PredictionOptions {
                boundary: Boundary::Reset,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(p.counter.value, initial.counter.value);
    let mut existing = sequential;
    let ep = catalog()
        .predict(
            &mut existing,
            &PredictionOptions {
                boundary: Boundary::CurrentCounter,
                current_counter: Some(initial.counter.value),
                ..o.clone()
            },
        )
        .unwrap();
    assert_eq!(existing, all);
    assert_eq!(ep.draws + 2, p.draws);
    sequential.advance(37);
    o.boundary = Boundary::Trigger;
    o.intervening_advances = "0".into();
    let q = catalog().predict(&mut sequential, &o).unwrap();
    assert_eq!(sequential, all);
    assert_eq!(q.group, p.group);
    assert_eq!(q.counter.value, p.trigger_reset.unwrap().value);
}

#[test]
fn first_counter_ratio_and_day_night_dash_modifiers() {
    let c = catalog();
    let mut r = CrtRng::new(123);
    let full = c
        .predict(
            &mut r,
            &PredictionOptions {
                boundary: Boundary::Reset,
                ..Default::default()
            },
        )
        .unwrap();
    let first = c
        .predict(
            &mut CrtRng::new(123),
            &PredictionOptions {
                boundary: Boundary::Reset,
                first_encounter: true,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(first.counter.value, full.counter.value / 2);
    assert_eq!(first.draws, 2);
    let day = c
        .predict(&mut CrtRng::new(123), &PredictionOptions::default())
        .unwrap();
    let night = c
        .predict(
            &mut CrtRng::new(123),
            &PredictionOptions {
                period: "Night".into(),
                dash: true,
                ability_percent: 50,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!(day.decrement, Some(11));
    assert_eq!(night.decrement, Some(12));
    assert_eq!(day.counter_updates, Some(day.counter.value.div_ceil(11)));
    assert_ne!(day.encounter_table, night.encounter_table);
}

#[test]
fn invalid_settings_do_not_advance_rng() {
    for o in [
        PredictionOptions {
            period: "DarkArea".into(),
            ..Default::default()
        },
        PredictionOptions {
            zone: "ENT_DNG_CTY_3_1_LND".into(),
            ..Default::default()
        },
        PredictionOptions {
            ability_percent: 0,
            ..Default::default()
        },
        PredictionOptions {
            party_levels: vec![],
            ..Default::default()
        },
        PredictionOptions {
            boundary: Boundary::CurrentCounter,
            ..Default::default()
        },
    ] {
        let mut r = CrtRng::new(123);
        r.advance(50);
        let before = r;
        assert!(catalog().predict(&mut r, &o).is_err());
        assert_eq!(r, before);
    }
}

#[test]
fn maps_without_encounters_still_support_counter_reset() {
    let c = catalog();
    assert_eq!(
        (c.maps.len(), c.zones.len(), c.volumes.len()),
        (229, 304, 721)
    );
    let p = c
        .predict(
            &mut CrtRng::new(123),
            &PredictionOptions {
                map: "MainTitle".into(),
                boundary: Boundary::Reset,
                ..Default::default()
            },
        )
        .unwrap();
    assert_eq!((p.counter.value, p.draws), (0, 0));
    assert!(p.enemies.is_empty());
}
