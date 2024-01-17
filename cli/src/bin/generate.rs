use clap::Parser;
use cli::*;
use handy_core::game::*;
use handy_core::solver::*;
use priq::PriorityQueue;
use rand::thread_rng;
use rand::Rng;
use serde_json;
use std::cmp;
use std::cmp::Reverse;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::{thread, time};
use std::collections::BTreeMap;

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
) ->ScoreMap {
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

                let maybe_winner = is_game_winner(&child_pile);
                if let Some(winner) = maybe_winner {
                    if winner == Allegiance::Hero {
                        winners.push(pile.clone());
                    } else {
                        // Don't bother looking at anything else for this pile.
                        // TODO: it's still possible to enqueue stuff before this. Oh well.
                        continue;
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
    #[clap(long)]
    all: bool,

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

fn write_examples<R: Rng>(hero: Class, monster: Class, rng: &mut R) {
    let model = try_read_model_for_matchup((hero, monster)).expect("Could not read model");
    let training_examples_path = training_path_for_matchup((hero, monster));

    // let pile = get_start_from_classes(hero, monster, &mut rng);
    let pile = get_random_pile_matching_stats(hero, monster, 20, 20, rng);
    let per_level_keep_states = per_level_keep_states(hero, monster);
    let extra_levels = extra_levels(hero, monster);

    let examples = generate_examples(pile, per_level_keep_states, extra_levels, &model);

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

    if args.all {
        for hero in HEROS {
            for monster in BADDIES {
                all_matchups.insert((hero, monster));
            }
        }
    }

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
