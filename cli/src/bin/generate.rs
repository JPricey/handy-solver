use cli::run_a_star::run_a_star_solver;
use cli::*;
use handy_core::game::*;
// use handy_core::utils::string_to_pile;
use rand::thread_rng;
use rand::Rng;

// const ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE: usize = 5_000_000;
// const ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE: usize = 2_000_000;
const ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE: usize = 5_000_000;

fn generate_example(hero: Class, monster: Class, extra_card_sets: &Vec<Vec<CardPtr>>) {
    let mut rng = thread_rng();
    let mut start_pile = get_random_pile_matching_stats(hero, monster, 30, 30, &mut rng);

    if extra_card_sets.len() > 0 {
        let extra_card_set_idx = rng.gen_range(0..extra_card_sets.len());
        let extra_cards = &extra_card_sets[extra_card_set_idx];

        for card in extra_cards {
            let insert_index = rng.gen_range(0..start_pile.len() + 1);
            start_pile.insert(insert_index, card.clone())
        }
    }

    println!(
        "{}, Starting new pile {hero:?} v {monster:?}: {:?}",
        get_datetime_stamp(),
        start_pile
    );

    let root_res = run_a_star_solver(
        start_pile.into(),
        None,
        Some(ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE),
        None,
        None,
        false,
    );
    if root_res.len() == 0 {
        return;
    }

    let example_iter =
        root_res
            .iter()
            .rev()
            .enumerate()
            .skip(1)
            .map(|(i, pile)| DepthModeTrainingExample {
                pile: pile.clone(),
                eval: StateEval::Win(i),
            });

    write_examples_to_file(&training_path_for_matchup((hero, monster)), example_iter);
}

fn main() {
    let extra_cards = vec![
        vec![CardPtr::new_from_id(103, FaceKey::A)],
        vec![CardPtr::new_from_id(103, FaceKey::B)],
        vec![CardPtr::new_from_id(103, FaceKey::D)],
    ];

    let all_matchups = get_training_matchups_from_args();
    if all_matchups.len() == 1 {
        let matchup = all_matchups.iter().next().unwrap();
        loop {
            generate_example(matchup.0, matchup.1, &extra_cards);
        }
    }

    loop {
        let matchup = find_least_used_matchup(all_matchups.iter());
        generate_example(matchup.0, matchup.1, &extra_cards);
    }
}
