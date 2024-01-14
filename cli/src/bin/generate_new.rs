use cli::*;
use handy_core::game::*;
use handy_core::solver::*;
use handy_core::utils::*;
use priq::PriorityQueue;
use rand::thread_rng;
use std::cmp;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::{thread, time};

const HERO: Class = Class::Cursed;
const MONSTER: Class = Class::Spider;

const PER_LEVEL_KEEP_STATES: usize = 500000;
const NUM_LEVELS_TO_EXPAND_AFTER_WINNER: usize = 2;

fn print_examples_per_depth_histo(ex: &HashMap<Pile, usize>) {
    let max_depth = ex.values().max().unwrap();
    let mut count_per_depth: Vec<usize> = vec![0; max_depth + 1];

    for depth in ex.values() {
        count_per_depth[*depth] += 1;
    }

    for i in 0..max_depth + 1 {
        println!("{}: {}", i, count_per_depth[i]);
    }
}

pub fn generate_examples<M: ModelT>(start_pile: Pile, model: &M) -> HashMap<Pile, usize> {
    println!("{:?}", start_pile);

    let mut level: usize = 0;
    let mut current_level = vec![start_pile.clone()];
    let mut extra_levels_count: usize = 0;
    let mut parents_per_level: Vec<HashMap<Pile, Vec<Pile>>> = Vec::new();
    let mut winners_per_level: Vec<Vec<(Pile, Pile)>> = Vec::new();

    loop {
        println!("Starting level {}: {}", level, current_level.len());

        let mut winners: Vec<(Pile, Pile)> = Vec::new(); // (parent pile, child pile)
        let mut next_level_queue: PriorityQueue<Reverse<f32>, Pile> = PriorityQueue::new();
        let mut is_queue_full = false;
        let mut in_queue: HashMap<Pile, Vec<Pile>> = HashMap::new(); // (child pile, parents)

        for pile in current_level {
            let cur_state = GameStateNoEventLog::new(pile.clone());
            let child_states = resolve_top_card(&cur_state);

            for child_state in child_states {
                let child_pile = child_state.pile;

                let maybe_winner = is_game_winner(&child_pile);
                if let Some(winner) = maybe_winner {
                    if winner == Allegiance::Hero {
                        winners.push((pile.clone(), child_pile.clone()));
                    } else {
                        // Don't bother looking at anything else for this pile.
                        // TODO: it's still possible to enqueue stuff before this. Oh well.
                        break;
                    }
                }

                if let Some(parents_list) = in_queue.get_mut(&child_pile) {
                    parents_list.push(pile.clone());
                    continue;
                }

                if is_queue_full {
                    let child_score = model.score_pile(&child_pile);
                    let (top_score, top_pile) = next_level_queue.peek().unwrap();
                    if child_score < top_score.0 {
                        in_queue.insert(child_pile.clone(), vec![pile.clone()]);
                        in_queue.remove(top_pile);
                        next_level_queue.pop();
                        next_level_queue.put(Reverse(child_score), child_pile.clone());
                    }
                } else {
                    let child_score = model.score_pile(&child_pile);
                    in_queue.insert(child_pile.clone(), vec![pile.clone()]);
                    next_level_queue.put(Reverse(child_score), child_pile.clone());
                    is_queue_full = in_queue.len() >= PER_LEVEL_KEEP_STATES;
                }
            }
        }

        let num_winners = winners.len();

        let mut should_continue = true;

        if num_winners > 0 || extra_levels_count > 0 {
            println!("Winners: {} @ level {}", num_winners, level);
            if extra_levels_count >= NUM_LEVELS_TO_EXPAND_AFTER_WINNER {
                should_continue = false;
            } else {
                extra_levels_count += 1;
            }
        }

        if !should_continue {
            parents_per_level.push(HashMap::new());
            winners_per_level.push(winners);
            break;
        } else {
            parents_per_level.push(in_queue);
            winners_per_level.push(winners);
        }

        level += 1;
        current_level = next_level_queue.drain(0..).map(|(_, p)| p).collect();
    }

    let mut known_winners: HashMap<Pile, usize> = HashMap::new();
    for _ in 0..level + 1 {
        let cur_winners = winners_per_level.pop().unwrap();
        let cur_parents = parents_per_level.pop().unwrap();

        for (parent_pile, child_pile) in cur_winners {
            known_winners.insert(parent_pile, 1);
            known_winners.insert(child_pile, 0);
        }

        for (child_pile, parent_list) in cur_parents {
            if let Some(child_score) = known_winners.get(&child_pile) {
                let score = child_score + 1;
                for parent_pile in parent_list {
                    known_winners
                        .entry(parent_pile)
                        .and_modify(|e| *e = cmp::min(*e, score))
                        .or_insert(score);
                }
            }
        }
    }

    assert!(winners_per_level.len() == 0);
    assert!(parents_per_level.len() == 0);

    println!("{} examples", known_winners.len());
    // for (pile, score) in known_winners.iter() {
    //     println!("{:?} : {}", pile, score);
    // }
    print_examples_per_depth_histo(&known_winners);

    known_winners
}

fn main() {
    let mut rng = thread_rng();
    let hero = HERO;
    let monster = MONSTER;
    let model = try_read_model_for_matchup((hero, monster)).expect("Could not read model");
    let training_examples_path = training_path_for_matchup((hero, monster));

    loop {
        let pile = get_start_from_classes(hero, monster, &mut rng);
        let examples = generate_examples(pile, &model);

        loop {
            let maybe_file_handle = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&training_examples_path);

            if let Ok(mut file) = maybe_file_handle {
                for (pile, depth) in &examples {
                    if *depth > 0 {
                        let example = DepthModeTrainingExample {
                            pile: pile.clone(),
                            eval: StateEval::Win(*depth),
                        };
                        let ex_str = serde_json::to_string(&example).unwrap();
                        if let Err(e) = writeln!(file, "{}", ex_str) {
                            eprintln!("Couldn't write to file: {}", e);
                        }
                    }
                }
                break;
            } else {
                eprintln!("Couldn't open file. Sleeping and trying again");
                thread::sleep(time::Duration::from_millis(100));
            }
        }
    }
}
