use crate::*;
use clap::Parser;
use handy_core::game::*;
use handy_core::solver::*;
use serde_json;
use std::collections::HashSet;
use std::fmt::Debug;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::{thread, time};

pub fn write_examples_to_file(
    filename: &str,
    examples: impl Iterator<Item = DepthModeTrainingExample>,
) {
    loop {
        let maybe_file_handle = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(&filename);

        if let Ok(mut file) = maybe_file_handle {
            for example in examples {
                let ex_str = serde_json::to_string(&example).unwrap();
                if let Err(e) = writeln!(file, "{}", ex_str) {
                    eprintln!("Couldn't write to file: {}", e);
                }
            }
            return;
        } else {
            eprintln!("Couldn't open file. Sleeping and trying again");
            thread::sleep(time::Duration::from_millis(100));
        }
    }
}

fn file_size_for_matchup(matchup: Matchup) -> u64 {
    let path = training_path_for_matchup(matchup);

    let Ok(file) = OpenOptions::new().read(true).open(path) else {
        return 0;
    };

    return file.metadata().map_or(0, |m| m.len());
}

pub fn find_least_used_matchup<'a>(matchups: impl Iterator<Item = &'a Matchup>) -> Matchup {
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

#[derive(Parser, Debug)]
pub struct TrainArgs {
    #[clap(long)]
    all: bool,

    #[clap(long, num_args=0..)]
    pub classes: Vec<Class>,

    #[clap(long, value_parser=parse_dot_separated_matchup, num_args=0..)]
    pub matchups: Vec<Matchup>,
}

pub fn get_training_matchups_from_args() -> Vec<Matchup> {
    let args = TrainArgs::parse();
    let mut all_matchups = HashSet::new();
    for matchup in args.matchups {
        all_matchups.insert(matchup);
    }

    for class in args.classes {
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

    all_matchups.into_iter().collect()
}
