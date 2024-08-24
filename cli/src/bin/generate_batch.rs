use cli::*;
use handy_core::game::*;
use handy_core::solver::*;
use priq::PriorityQueue;
use rand::thread_rng;
use rand::Rng;
use std::cmp;
use std::cmp::Reverse;
use std::collections::BTreeMap;
use std::collections::HashMap;
use handy_core::game::end_game::standard_check_is_game_winner;

pub type ScoreMap = BTreeMap<Pile, usize>;

fn per_level_keep_states(hero: Class, monster: Class) -> usize {
    if hero == Class::Beastmaster && monster == Class::Flora {
        return 80000;
    }

    let mut result = 400000;
    if hero == Class::Beastmaster {
        result /= 3;
    }

    if monster == Class::Vampire {
        result /= 2;
    }

    return result;
}

fn extra_levels(hero: Class, monster: Class) -> usize {
    if hero == Class::Beastmaster && monster == Class::Flora {
        return 0;
    }
    if hero == Class::Beastmaster {
        return 1;
    }
    return 2;
}

fn print_examples_per_depth_histo(ex: &ScoreMap) {
    let Some(max_depth) = ex.values().max() else {
        return;
    };
    let mut count_per_depth: Vec<usize> = vec![0; max_depth + 1];

    for depth in ex.values() {
        count_per_depth[*depth] += 1;
    }

    for i in 0..max_depth + 1 {
        println!("{}: {}", i, count_per_depth[i]);
    }
}

pub fn generate_examples<M: ModelT>(
    start_pile: Pile,
    keep_states: usize,
    extra_levels: usize,
    model: &M,
) -> ScoreMap {
    println!("{:?}", start_pile);

    let mut level: usize = 0;
    let mut current_level = vec![start_pile.clone()];
    let mut extra_levels_count: usize = 0;
    let mut parents_per_level: Vec<HashMap<Pile, Vec<Pile>>> = Vec::new();
    let mut winners_per_level: Vec<Vec<Pile>> = Vec::new();

    loop {
        println!("Starting level {}: {}", level, current_level.len());
        let mut winners: Vec<Pile> = Vec::new(); // parent pile only
        let mut next_level_queue: PriorityQueue<Reverse<f32>, Pile> = PriorityQueue::new();
        let mut is_queue_full = false;
        let mut in_queue: HashMap<Pile, Vec<Pile>> = HashMap::with_capacity(keep_states + 1); // (child pile, parents)

        for pile in current_level {
            let cur_state = GameStateWithEventLog::new(pile.clone());
            let child_states = resolve_top_card(&cur_state);

            for child_state in child_states {
                let child_pile = child_state.pile;

                let resolution = standard_check_is_game_winner(&child_pile);
                if resolution == WinType::Win {
                    winners.push(pile.clone());
                } else if resolution == WinType::Lose {
                    // Don't bother looking at anything else for this pile.
                    // TODO: it's still possible to enqueue stuff before this. Oh well.
                    continue;
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
                    is_queue_full = in_queue.len() >= keep_states;
                }
            }
        }

        let total_parents: usize = in_queue.values().map(|p| p.len()).sum();
        println!("queue_len: {} / parent: {}", in_queue.len(), total_parents);

        let num_winners = winners.len();

        let mut should_continue = true;

        if num_winners > 0 || extra_levels_count > 0 {
            println!("Winners: {} @ level {}", num_winners, level + 1);
            if extra_levels_count >= extra_levels {
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
            in_queue.shrink_to_fit();
            parents_per_level.push(in_queue);
            winners_per_level.push(winners);
        }

        current_level = next_level_queue.drain(0..).map(|(_, p)| p).collect();
        if current_level.len() == 0 {
            println!("No more paths");
            break;
        }

        if level > 50 {
            break;
        }

        level += 1;
    }

    let mut known_winners: ScoreMap = ScoreMap::new();
    for _ in 0..level + 1 {
        let cur_winners = winners_per_level.pop().unwrap();
        let cur_parents = parents_per_level.pop().unwrap();

        for parent_pile in cur_winners {
            known_winners.insert(parent_pile, 1);
            // known_winners.insert(child_pile, 0);
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

fn write_examples<R: Rng>(hero: Class, monster: Class, rng: &mut R) {
    let model = try_read_model_for_matchup((hero, monster)).expect("Could not read model");
    let training_examples_path = training_path_for_matchup((hero, monster));

    // let pile = get_start_from_classes(hero, monster, &mut rng);
    let pile = get_random_pile_matching_stats(hero, monster, 20, 20, rng);
    let per_level_keep_states = per_level_keep_states(hero, monster);
    let extra_levels = extra_levels(hero, monster);

    let examples = generate_examples(pile, per_level_keep_states, extra_levels, &model);

    let example_iter = examples.into_iter().filter_map(|(pile, depth)| {
        return if depth > 0 {
            Some(DepthModeTrainingExample {
                pile: pile.clone(),
                eval: StateEval::Win(depth),
            })
        } else {
            None
        };
    });

    write_examples_to_file(&training_examples_path, example_iter);
}

fn main() {
    let all_matchups = get_training_matchups_from_args();
    let mut rng = thread_rng();
    if all_matchups.len() == 1 {
        let matchup = all_matchups.iter().next().unwrap();
        loop {
            write_examples(matchup.0, matchup.1, &mut rng);
        }
    } else {
        loop {
            let matchup = find_least_used_matchup(all_matchups.iter());
            println!("{:?}", matchup);
            write_examples(matchup.0, matchup.1, &mut rng);
        }
    }
}
