
/*
fn write_examples<R: Rng>(hero: Class, monster: Class, rng: &mut R) {
    let start_pile = get_random_pile_matching_stats(hero, monster, RANDOMIZE_SIDES_PCT, RANDOMIZE_HERO_SIDES_PCT, &mut rng);

    println!(
        "{}, Starting new pile {hero:?} v {monster:?}: {:?}",
        get_datetime_stamp(),
        start_pile
    );

    let root_res = run_a_star_solver(
        start_pile.into(),
        None,
        Some(ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE),
    );
    if root_res.len() == 0 {
        return;
    }

    for (i, pile) in root_res.iter().rev().enumerate().skip(1) {
        let example = DepthModeTrainingExample {
            pile: pile.clone(),
            eval: StateEval::Win(i),
        };
        let ex_str = serde_json::to_string(&example).unwrap();
        let path = training_path_for_matchup((hero, monster));

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path)
            .unwrap();

        if let Err(e) = writeln!(file, "{}", ex_str) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}
*/
