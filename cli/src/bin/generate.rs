use clap::Parser;
use cli::run_a_star::run_a_star_solver;
use cli::*;
use handy_core::game::*;
use handy_core::solver::*;
use handy_core::utils::*;
use rand::thread_rng;
use rand::Rng;
use serde_json;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::prelude::*;

const ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE: usize = 8_000_000;
const RANDOMIZE_SIZED_PCT: usize = 30;
const RANDOMIZE_HERO_SIDES_PCT: usize = 30;

pub fn maybe_randomize_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
    let score: usize = rng.gen_range(0..100);
    if score < RANDOMIZE_SIZED_PCT {
        println!("Random start!!");
        randomize_sides(pile, rng);
    } else if score < RANDOMIZE_SIZED_PCT + RANDOMIZE_HERO_SIDES_PCT {
        println!("RANDOM BAD START");
        randomize_hero_sides(pile, rng);
    }
}

pub fn randomize_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
    for card_ptr in pile.iter_mut() {
        card_ptr.key = get_random_face(rng);
    }

    if is_game_winner(pile).is_some() {
        randomize_sides(pile, rng);
    }
}

pub fn randomize_hero_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
    for card_ptr in pile.iter_mut() {
        if card_ptr.get_active_face().allegiance != Allegiance::Baddie {
            card_ptr.key = get_random_face(rng);
        }
    }

    if is_game_winner(pile).is_some() {
        randomize_hero_sides(pile, rng);
    }
}

fn generate_example(hero: Class, baddie: Class) {
    let mut rng = thread_rng();
    let mut start_pile = get_start_from_classes(hero, baddie, &mut rng);
    maybe_randomize_sides(&mut start_pile, &mut rng);

    println!(
        "{}, Starting new pile {hero:?} v {baddie:?}: {:?}",
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
        let path = training_path_for_matchup((hero, baddie));

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

#[derive(Parser, Debug)]
pub struct TrainArgs {
    #[clap(short, long, num_args = 2)]
    pub classes: Vec<Class>,
}

fn main() {
    let args = TrainArgs::parse();
    let matchup = try_get_matchup_from_classes(&args.classes).expect("Could not parse matchup");

    loop {
        let (hero, baddie) = matchup;
        generate_example(hero, baddie);
    }
}
