pub mod generate_helpers;
pub mod parsers;
pub mod paths;
pub mod pile_randomizers;
pub mod run_a_star;
pub mod training_utils;

pub use generate_helpers::*;
pub use parsers::*;
pub use paths::*;
pub use pile_randomizers::*;
pub use training_utils::*;

use chrono::offset::Utc;
use chrono::DateTime;
use clap::{ArgGroup, Parser};
use handy_core::game::*;
use handy_core::solver::*;
use handy_core::utils::*;
use rand::thread_rng;
use rand::RngCore;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::time::SystemTime;

pub fn get_datetime_stamp() -> String {
    Into::<DateTime<Utc>>::into(SystemTime::now())
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}

#[derive(Parser, Debug)]
#[clap(group(
            ArgGroup::new("boop")
                .required(true)
                .args(&["pile", "classes"]),
        ))]
pub struct StandardArgs {
    #[clap(short, long, value_parser=string_to_pile_result)]
    pub pile: Option<Pile>,
    #[clap(short, long, num_args = 2)]
    pub classes: Option<Vec<Class>>,
    #[clap(short, long)]
    pub seed: Option<String>,
    #[clap(short, long)]
    pub g_bias: Option<f32>,
}

pub fn get_starting_pile_from_args(args: &StandardArgs) -> Pile {
    if let Some(pile) = args.pile.clone() {
        pile
    } else {
        let classes = args.classes.clone().unwrap();
        let mut rng: Box<dyn RngCore> = if let Some(seed) = args.seed.clone() {
            Box::new(Seeder::from(seed).make_rng::<Pcg64>())
        } else {
            Box::new(thread_rng())
        };

        get_start_from_classes(classes[0], classes[1], &mut rng)
    }
}

pub fn get_starting_pile() -> Pile {
    let args = StandardArgs::parse();
    get_starting_pile_from_args(&args)
}

pub fn get_model_for_pile(pile: &Pile) -> Model {
    let matchups = get_all_matchups_from_pile(pile);
    let mut models: Vec<Model> = matchups
        .into_iter()
        .map(|matchup| try_read_model_for_matchup(matchup).unwrap())
        .collect();

    if does_have_quest(&pile) {
        let quest_model = try_read_quest_model().unwrap();
        models.push(quest_model);
    }

    if models.len() == 1 {
        models[0].clone()
    } else {
        merge_models_for_pile(pile, &models)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use handy_core::utils::string_to_pile;

    #[test]
    fn test_empty_pile() {
        let result = StandardArgs::try_parse_from(["cmd", "--pile", "beep"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_regular_pile_long() {
        let result = StandardArgs::try_parse_from(["cmd", "--pile", "1A2B3c"]);
        assert_eq!(result.unwrap().pile.unwrap(), string_to_pile("1 2B 3C"))
    }

    #[test]
    fn test_regular_pile_short() {
        let result = StandardArgs::try_parse_from(["cmd", "-p", "1A2A3A"]);
        assert_eq!(result.unwrap().pile.unwrap(), string_to_pile("1 2 3"))
    }

    #[test]
    fn test_classes_with_seed() {
        let args =
            StandardArgs::parse_from(["cmd", "--classes", "warrior", "ogre", "--seed", "abc"]);
        let pile = get_starting_pile_from_args(&args);
        assert_eq!(pile, string_to_pile("3A 9A 4A 1A 5A 7A 2A 8A 6A"));
    }
}
