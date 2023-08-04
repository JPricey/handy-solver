use handy_core::game_state::*;
use handy_core::game_utils::*;
use handy_core::types::*;
use cli_lib::*;

use chrono::offset::Utc;
use chrono::DateTime;
use priq::PriorityQueue;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::time::SystemTime;

// BTree is slower, but memory is more compact, and resize events are gradual
// type SeenMap = HashMap<Pile, SolverState>;
type SeenMap = BTreeMap<Pile, SolverState>;

fn score_pile(pile: &Pile) -> f32 {
    let mut total = 0.0;

    for card in pile.iter() {
        let active_face = card.get_active_face();
        let active_face_value = active_face.value;
        if active_face_value > 0.0 {
            total += active_face.value;
        } else {
            total += 2.0 * active_face.value;
        }
    }

    total
}

#[derive(Debug)]
struct SolverState {
    parent: Option<Pile>,
    depth: usize,
}

fn print_solution(pile: &Pile, seen_state: &SeenMap) {
    let solve_state = seen_state.get(pile).unwrap();

    if let Some(parent) = &solve_state.parent {
        print_solution(&parent, seen_state);

        let possible_paths = resolve_top_card(&GameStateWithEventLog::new(parent.clone()));
        for path in &possible_paths {
            if &path.pile == pile {
                for e in &path.events {
                    println!("\t{}", format_event_for_cli(e));
                }
                println!("{:?}", pile);
                return;
            }
        }
        println!("{:?}: ?Unknown Path. This is a bug?", pile);
    } else {
        println!("{:?}", pile);
    }
}

fn clear_depth(depth: usize, seen_state: &mut SeenMap, best_wins: &Vec<Pile>) {
    println!("Clearing to depth {}", depth);
    let mut keeps = best_wins.clone();
    for b in best_wins {
        if let Some(state) = seen_state.get(b) {
            if let Some(parent) = &state.parent {
                keeps.push(parent.clone())
            }
        }
    }

    seen_state.retain(|k, v| v.depth < depth - 1 || keeps.contains(k));
}

pub fn run_solver(start_pile: Pile) {
    let mut seen_state = SeenMap::new();
    let mut q: PriorityQueue<f32, Pile> = PriorityQueue::new();
    let mut shortest_win = 100000;

    // By depth and then score:
    let mut best_score: (usize, f32) = (100000, 100.0);
    let mut best_wins: Vec<Pile> = vec![];

    seen_state.insert(
        start_pile.clone(),
        SolverState {
            parent: None,
            depth: 0,
        },
    );
    q.put(0.0, start_pile);

    let mut iters = 0;

    loop {
        iters += 1;
        if iters > 100000 {
            iters = 0;
            let now = SystemTime::now();
            let datetime: DateTime<Utc> = now.into();
            println!(
                "{}: in queue: {} | seen: {} | best score: {:?}",
                datetime.format("%Y-%m-%d %H:%M:%S"),
                q.len(),
                seen_state.len(),
                best_score,
            );
        }

        let pile_entry = q.pop();
        if pile_entry == None {
            println!("Done searching");
            break;
        }
        let pile = pile_entry.unwrap().1;

        let Some(parent) = seen_state.get(&pile)
        else {
            continue;
        };

        let this_depth = parent.depth + 1;

        if this_depth >= shortest_win - 1 {
            continue;
        }

        let init_state = GameStateNoEventLog::new(pile.clone());
        let new_states = resolve_top_card(&init_state);

        for state in new_states {
            let new_pile = state.get_pile().clone();

            let maybe_winner = is_game_winner(&new_pile);
            if Some(Allegiance::Baddie) == maybe_winner {
                continue;
            }

            if !seen_state.contains_key(&state.pile) {
                q.put(-score_pile(&state.pile), state.pile.clone());
                seen_state.insert(
                    new_pile.clone(),
                    SolverState {
                        parent: Some(pile.clone()),
                        depth: this_depth,
                    },
                );
            }

            if let Some(winner) = maybe_winner {
                if winner == Allegiance::Hero {
                    let this_health_score = score_pile(&new_pile);
                    let this_score = (this_depth, -this_health_score);

                    if this_score < best_score {
                        best_score = this_score;
                        println!("New best solution: {:?}", best_score);
                        print_solution(&new_pile, &seen_state);

                        best_wins = vec![new_pile];
                    } else if this_score == best_score {
                        best_wins.push(new_pile);
                    }

                    if this_depth < shortest_win {
                        shortest_win = this_depth;
                        clear_depth(this_depth, &mut seen_state, &best_wins);
                    }
                }
            }
        }
    }
}

fn main() {
    let start_pile = get_starting_pile();
    run_solver(start_pile.into());
}
