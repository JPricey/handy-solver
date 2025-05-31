use std::collections::{HashMap};

use cli::get_fully_random_pile;
use handy_core::{
    game::{
        end_game::{is_game_winner, GameEndCheckType}, resolve_top_card, Allegiance, CardPtrT, Class, EngineGameState, GameStateNoEventLog, Pile, WinType
    },
    utils::get_start_from_classes,
};
use rand::thread_rng;

#[derive(Debug, PartialEq, Eq, Clone)]
enum RecursiveResult {
    NoSolutions,
    SingleSolution(usize),
    ManySolutions(usize),
}

const LOG_SOLN_FOR_DEPTH: usize = 5;

type VisitedCache = HashMap<Pile, (usize, RecursiveResult)>;

fn _inner(
    visited_cache: &mut VisitedCache,
    source_pile: &Pile,
    rem_depth: usize,
) -> RecursiveResult {
    match is_game_winner(source_pile, GameEndCheckType::Standard) {
        WinType::Win => return RecursiveResult::SingleSolution(0),
        WinType::Lose => return RecursiveResult::NoSolutions,
        WinType::Unresolved => (),
    }

    if rem_depth == 0 {
        return RecursiveResult::NoSolutions;
    }

    if let Some((cached_depth, cached_result)) = visited_cache.get(source_pile) {
        if *cached_depth >= rem_depth || *cached_result != RecursiveResult::NoSolutions {
            return cached_result.clone();
        }
    }

    visited_cache.insert(
        source_pile.clone(),
        (rem_depth, RecursiveResult::NoSolutions),
    );

    let child_states = resolve_top_card(&GameStateNoEventLog::new(source_pile.clone()));

    let mut current_outcome = RecursiveResult::NoSolutions;

    for child_state in &child_states {
        let child_result = _inner(visited_cache, &child_state.pile, rem_depth - 1);

        match child_result {
            RecursiveResult::NoSolutions => (),
            RecursiveResult::ManySolutions(amt) => {
                current_outcome = match current_outcome {
                    RecursiveResult::NoSolutions => child_result,
                    RecursiveResult::ManySolutions(other) => {
                        RecursiveResult::ManySolutions(amt + other)
                    }
                    RecursiveResult::SingleSolution(_) => RecursiveResult::ManySolutions(amt + 1),
                }
            }
            RecursiveResult::SingleSolution(child_win_depth) => {
                current_outcome = match current_outcome {
                    RecursiveResult::NoSolutions => {
                        RecursiveResult::SingleSolution(child_win_depth + 1)
                    }
                    RecursiveResult::ManySolutions(other) => {
                        RecursiveResult::ManySolutions(other + 1)
                    }
                    RecursiveResult::SingleSolution(_) => RecursiveResult::ManySolutions(2),
                }
            }
        }
    }

    if let RecursiveResult::SingleSolution(win_depth) = current_outcome {
        if win_depth >= LOG_SOLN_FOR_DEPTH {
            if source_pile[0].get_active_face().allegiance == Allegiance::Hero {
                println!("Depth {} only soln: {:?}", win_depth, source_pile);
            }
        }
    }

    if current_outcome != RecursiveResult::NoSolutions {
        visited_cache.insert(source_pile.clone(), (rem_depth, current_outcome.clone()));
    }

    current_outcome
}

fn main() {
    let mut rng = thread_rng();
    let hero = Class::Assassin;
    let monster = Class::Vampire;

    for i in 1.. {
        let root = if false {
            get_fully_random_pile(hero, monster, &mut rng)
        } else {
            get_start_from_classes(hero, monster, &mut rng)
        };
        println!("{i}: investigating source {:?}", root);

        let mut visited_set = VisitedCache::new();
        _inner(&mut visited_set, &root, 9);
    }
}
