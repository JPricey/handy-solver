use super::interface_utils::*;
use super::card_defs::*;
use super::types::*;
use clap::{ArgGroup, Parser};
use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::RngCore;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

#[derive(Parser, Debug)]
#[clap(group(
            ArgGroup::new("boop")
                .required(true)
                .args(&["pile", "classes"]),
        ))]
struct Args {
    #[clap(short, long, value_parser=string_to_pile_result)]
    pile: Option<Pile>,
    #[clap(short, long, num_args = 2)]
    classes: Option<Vec<Class>>,
    #[clap(short, long)]
    seed: Option<String>,
}

pub fn get_start_from_classes(
    hero_class: Class,
    monster_class: Class,
    rng: &mut dyn RngCore,
) -> Pile {
    let mut cards = CARDS.get_cards_for_class(hero_class);
    cards.append(&mut CARDS.get_cards_for_class(monster_class));
    cards.shuffle(rng);

    return cards
        .iter()
        .map(|&card_def| CardPtr::new_from_id(card_def.id as usize, FaceKey::A))
        .collect();
}

fn get_starting_pile_from_args(args: Args) -> Pile {
    if let Some(pile) = args.pile {
        pile
    } else {
        let classes = args.classes.unwrap();
        let mut rng: Box<dyn RngCore> = if let Some(seed) = args.seed {
            Box::new(Seeder::from(seed).make_rng::<Pcg64>())
        } else {
            Box::new(thread_rng())
        };

        get_start_from_classes(classes[0], classes[1], &mut rng)
    }
}

pub fn get_starting_pile() -> Pile {
    let args = Args::parse();
    get_starting_pile_from_args(args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pile() {
        let result = Args::try_parse_from(["cmd", "--pile", "beep"]);
        assert!(result.is_err());
    }

    #[test]
    fn test_regular_pile_long() {
        let result = Args::try_parse_from(["cmd", "--pile", "1A2B3c"]);
        assert_eq!(result.unwrap().pile.unwrap(), string_to_pile("1 2B 3C"))
    }

    #[test]
    fn test_regular_pile_short() {
        let result = Args::try_parse_from(["cmd", "-p", "1A2A3A"]);
        assert_eq!(result.unwrap().pile.unwrap(), string_to_pile("1 2 3"))
    }

    #[test]
    fn test_classes_with_seed() {
        let args = Args::parse_from(["cmd", "--classes", "paladin", "ogre", "--seed", "abc"]);
        let pile = get_starting_pile_from_args(args);
        assert_eq!(pile, string_to_pile("3A 9A 4A 1A 5A 7A 2A 8A 6A"));
    }
}
