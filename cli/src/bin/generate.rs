use cli::run_a_star::run_a_star_solver;
use cli::*;
use handy_core::game::*;
use handy_core::utils::string_to_pile;
use rand::thread_rng;

const ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE: usize = 8_000_000;

fn generate_example(hero: Class, monster: Class) {
    let mut rng = thread_rng();
    // let start_pile = get_random_pile_matching_stats(hero, monster, 30, 30, &mut rng);

    let start_pile = string_to_pile("42C, 46A, 49D, 43B, 48C, 47B, 45B, 50D, 44B");

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
    let all_matchups = get_training_matchups_from_args();
    if all_matchups.len() == 1 {
        let matchup = all_matchups.iter().next().unwrap();
        loop {
            generate_example(matchup.0, matchup.1);
        }
    } else {
        loop {
            let matchup = find_least_used_matchup(all_matchups.iter());
            generate_example(matchup.0, matchup.1);
        }
    }
}
