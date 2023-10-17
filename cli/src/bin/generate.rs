use clap::Parser;
use cli::run_a_star::run_a_star_solver;
use cli::*;
use handy_core::game::*;
use handy_core::solver::*;
use handy_core::utils::*;
use rand::thread_rng;
use rand::Rng;
use serde_json;
use std::collections::HashSet;
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

fn parse_dot_separated_matchup(s: &str) -> Result<Matchup, String> {
    let pos = s
        .find('.')
        .ok_or_else(|| format!("invalid KEY=value: no `.` found in `{}`", s))?;
    let hero: Class = s[..pos].parse().map_err(|err| format!("{}", err))?;
    let enemy: Class = s[pos + 1..].parse().map_err(|err| format!("{}", err))?;

    if !is_hero_class(hero) {
        return Err("First class must be a hero".into());
    }

    if is_hero_class(enemy) {
        return Err("Second class must be an enemy".into());
    }

    Ok((hero, enemy))
}

#[derive(Parser, Debug)]
pub struct TrainArgs {
    #[clap(long, num_args=0..)]
    pub full: Vec<Class>,

    #[clap(long, value_parser=parse_dot_separated_matchup, num_args=0..)]
    pub matchups: Vec<Matchup>,
}

fn file_size_for_matchup(matchup: Matchup) -> u64 {
    let path = training_path_for_matchup(matchup);

    let Ok(file) = OpenOptions::new().read(true).open(path) else {
        return 0;
    };

    return file.metadata().map_or(0, |m| m.len());
}

fn find_least_used_matchup<'a>(matchups: impl Iterator<Item = &'a Matchup>) -> Matchup {
    let mut result: Option<(Matchup, u64)> = None;

    for item in matchups {
        let matchup = item.clone();
        let file_len = file_size_for_matchup(matchup);

        if let Some(known_result) = result {
            if file_len < known_result.1 {
                result = Some((matchup, file_len));
            }
        } else {
            result = Some((matchup, file_len));
        }
    }

    result.unwrap().0
}

fn main() {
    let args = TrainArgs::parse();

    let mut all_matchups = HashSet::new();
    for matchup in args.matchups {
        all_matchups.insert(matchup);
    }

    for class in args.full {
        if is_hero_class(class) {
            for other in BADDIES {
                all_matchups.insert((class, other));
            }
        } else {
            for other in HEROS {
                all_matchups.insert((other, class));
            }
        }
    }

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
