// use clap::Parser;
// use cli::*;
// use cli::run_a_star::run_a_star_solver;
// use handy_core::game::*;
// use handy_core::solver::*;
// use handy_core::utils::*;
// use priq::PriorityQueue;
// use rand::thread_rng;
// use rand::Rng;
// use serde_json;
// use std::cmp;
// use std::collections::BTreeMap;
// use std::collections::HashMap;
// use std::fmt::Debug;
// use std::fs::OpenOptions;
// use std::io::prelude::*;
//
// const ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE: usize = 8_000_000;
// // const ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE: usize = 15_000_000;
// // const ROOT_PILE_SOLVE_NUM_ITERS_FOR_DEPTH_MODE: usize = 1_000_000;
//
// type SeenMap = BTreeMap<Pile, Node>;
//
// #[derive(Debug)]
// struct Node {
//     parents: Vec<Pile>,
//     depth: usize,
//     win_depth: Option<usize>,
// }
//
// const INF_WIN_DEPTH: usize = 100000;
// const F_SCORE_CUTOFF: f32 = 10.0;
//
// fn propagate_win(pile: &Pile, win_depth: usize, seen_states: &mut SeenMap) {
//     let state = seen_states.get_mut(pile).unwrap();
//     let mut parent_updates = vec![];
//     if state.win_depth.map_or(true, |d| d > win_depth) {
//         state.win_depth = Some(win_depth);
//         parent_updates = state.parents.clone();
//     }
//
//     for p in parent_updates {
//         propagate_win(&p, win_depth + 1, seen_states);
//     }
// }
//
// pub fn run_generator(start_pile: Pile, model: &Model) {
//     let mut seen_states: BTreeMap<Pile, Node> = SeenMap::new();
//
//     if is_game_winner(&start_pile).is_some() {
//         return;
//     }
//
//     seen_states.insert(
//         start_pile.clone(),
//         Node {
//             parents: vec![],
//             depth: 0,
//             win_depth: None,
//         },
//     );
//
//     let mut seeds: Vec<_> = resolve_top_card(&GameStateNoEventLog::new(start_pile.clone()))
//         .into_iter()
//         .map(|s| s.pile)
//         .collect();
//     // Highest to lowest score
//     seeds.sort_by(|a, b| {
//         model
//             .score_pile(b)
//             .partial_cmp(&model.score_pile(a))
//             .unwrap_or(cmp::Ordering::Less)
//     });
//
//     for seed in &seeds {
//         if is_game_winner(seed).is_some() {
//             return;
//         }
//
//         seen_states.insert(
//             seed.clone(),
//             Node {
//                 parents: vec![],
//                 depth: 1,
//                 win_depth: None,
//             },
//         );
//     }
//
//     for seed in &seeds {
//         print!("Starting seed {seed:?}");
//         let mut win_depth: usize = INF_WIN_DEPTH;
//         let mut queue: PriorityQueue<f32, Pile> = PriorityQueue::new();
//         queue.put(0.0, seed.clone());
//
//         loop {
//             let Some((score, pile)) = queue.pop() else {
//                 break;
//             };
//
//             if F_SCORE_CUTOFF + (win_depth as f32) > score {
//                 print!("F-score cutoff reached")
//             }
//
//             let (current_win_depth, current_depth) = {
//                 let current_state = seen_states.get(&pile).unwrap();
//                 (current_state.win_depth, current_state.depth)
//             };
//             let child_depth = current_depth + 1;
//
//             let child_piles: Vec<_> = resolve_top_card(&GameStateNoEventLog::new(pile.clone()))
//                 .into_iter()
//                 .map(|s| s.pile)
//                 .collect();
//             for child_pile in child_piles {
//                 if let Some(winner) = is_game_winner(&child_pile) {
//                     if winner == Allegiance::Hero {
//                         win_depth = cmp::min(child_depth, win_depth);
//
//                         if current_win_depth.map_or(true, |d| d > child_depth) {
//                             propagate_win(&pile, 1, &mut seen_states);
//                         }
//                     }
//                 } else {
//                     if let Some(child_seen_state) = seen_states.get_mut(&child_pile) {
//                         child_seen_state.parents.push(pile.clone());
//                         if child_depth < child_seen_state.depth {
//                             queue.put(
//                                 child_depth as f32 + model.score_pile(&child_pile),
//                                 child_pile.clone(),
//                             );
//                         }
//                     } else {
//                         queue.put(
//                             child_depth as f32 + model.score_pile(&child_pile),
//                             child_pile.clone(),
//                         );
//                         seen_states.insert(
//                             child_pile,
//                             Node {
//                                 parents: vec![pile.clone()],
//                                 depth: child_depth,
//                                 win_depth: None,
//                             },
//                         );
//                     }
//                 }
//             }
//         }
//     }
//
//     let mut results: HashMap<Pile, usize> = HashMap::new();
//     {
//         results.insert(
//             start_pile.clone(),
//             seen_states.get(&start_pile).unwrap().win_depth.unwrap(),
//         );
//     }
//
//     for seed in seeds {
//         // let seed_state = seen_states.get(seed).unwrap();
//     }
// }
//
// pub fn randomize_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
//     for card_ptr in pile.iter_mut() {
//         card_ptr.key = get_random_face(rng);
//     }
//
//     if is_game_winner(pile).is_some() {
//         randomize_sides(pile, rng);
//     }
// }
//
// pub fn randomize_hero_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
//     for card_ptr in pile.iter_mut() {
//         if card_ptr.get_active_face().allegiance != Allegiance::Baddie {
//             card_ptr.key = get_random_face(rng);
//         }
//     }
//
//     if is_game_winner(pile).is_some() {
//         randomize_hero_sides(pile, rng);
//     }
// }
//
// const RANDOMIZE_SIZED_PCT: usize = 30;
// const RANDOMIZE_HERO_SIDES_PCT: usize = 30;
// pub fn maybe_randomize_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
//     let score: usize = rng.gen_range(0..100);
//     if score < RANDOMIZE_SIZED_PCT {
//         println!("Random start!!");
//         randomize_sides(pile, rng);
//     } else if score < RANDOMIZE_SIZED_PCT + RANDOMIZE_HERO_SIDES_PCT {
//         println!("RANDOM BAD START");
//         randomize_hero_sides(pile, rng);
//     }
// }
//

fn main() {
    println!("WIP");
}
