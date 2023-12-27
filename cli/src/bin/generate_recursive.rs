use clap::Parser;
use cli::run_a_star::run_a_star_solver;
use cli::*;
use handy_core::game::*;
use handy_core::solver::*;
use handy_core::utils::*;
use rand::thread_rng;
use rand::Rng;
use serde_json;
use std::cmp;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::prelude::*;

const F_SCORE_BUFFER: f32 = 10.0;
const MAX_DEPTH: usize = 14;
const CULL_BUFFER: f32 = 10.0;

fn solve_state_recursive(
    pile: &Pile,
    max_f_score: f32,
    depth: usize,
    model: &Model,
) -> Option<usize> {
    let child_max_f_score = max_f_score - 1.0;
    let new_states = resolve_top_card(&GameStateNoEventLog::new(pile.clone()));

    let mut min_win: Option<usize> = None;

    for state in new_states {
        if let Some(winner) = is_game_winner(&state.pile) {
            if winner == Allegiance::Hero {
                min_win = Some(1);
            }
            return min_win;
        }

        if depth + 1 >= MAX_DEPTH {
            continue;
        }

        if child_max_f_score < 1.0 {
            continue;
        }

        let state_score = model.score_pile(&state.pile);

        if state_score > (MAX_DEPTH - depth) as f32 + CULL_BUFFER {
            continue;
        }

        if state_score < child_max_f_score {
            let maybe_child_best_score =
                solve_state_recursive(&state.pile, child_max_f_score, depth + 1, model);

            if let Some(child_best_score) = maybe_child_best_score {
                let child_best_score = child_best_score + 1;
                min_win = min_win.map_or(Some(child_best_score), |x| {
                    Some(cmp::min(x, child_best_score))
                })
            }
        }
    }

    if min_win.is_some() {
        println!("{}/{:?}: {:?}", depth, min_win, pile);
    }

    return min_win;
}

fn generate_example(hero: Class, baddie: Class) {
    let mut rng = thread_rng();
    let start_pile = get_start_from_classes(hero, baddie, &mut rng);
    let model = get_model_for_pile(&start_pile);
    let expected_score = model.score_pile(&start_pile);
    solve_state_recursive(&start_pile, expected_score + F_SCORE_BUFFER, 0, &model);
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
            return;
        }
    } else {
        loop {
            let matchup = find_least_used_matchup(all_matchups.iter());
            generate_example(matchup.0, matchup.1);
        }
    }
}
