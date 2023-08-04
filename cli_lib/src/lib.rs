use clap::{ArgGroup, Parser};
use handy_core::card_defs::*;
use handy_core::pile_utils::*;
use handy_core::types::*;
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

pub fn format_wrapped_action(wrapped_action: &WrappedAction) -> String {
    format!("{:?} {:?}", wrapped_action.action, wrapped_action.target)
}

pub fn format_event_for_cli(event: &Event) -> String {
    match event {
        Event::PickRow(row_num, card_ptr) => format!("{:?}: Row {}", card_ptr, row_num),
        Event::SkipTurn => "Skip".to_owned(),
        Event::BottomCard(card_ptr) => format!("{:?}: Bottomed", card_ptr),
        Event::SkipAction(card_ptr, wrapped_action) => format!(
            "{:?}: Skip {}",
            card_ptr,
            format_wrapped_action(wrapped_action)
        ),
        Event::StartAction(card_ptr, wrapped_action) => format!(
            "{:?}: Trigger {}",
            card_ptr,
            format_wrapped_action(wrapped_action)
        ),
        Event::AttackCard(card_ptr, wrapped_action) => {
            format!("{} {:?}", format_wrapped_action(wrapped_action), card_ptr)
        }
        Event::UseActionAssist(assist_idx, assist_card_ptr, assist_row_idx) => format!(
            "{:?}@{}: Assist Row {}",
            assist_card_ptr, assist_idx, assist_row_idx
        ),
        Event::Damage(card_idx, card_ptr, hit_type, result_face) => format!(
            "{:?}@{}: Damage({:?})=>{:?}",
            card_ptr, card_idx, hit_type, result_face
        ),
        Event::Death => format!("Death"),
        Event::Void(card_idx, card_ptr, face_key) => {
            format!("{:?}@{}: Void=>{:?}", card_ptr, card_idx, face_key)
        }
        Event::Inspire(card_idx, card_ptr) => format!("Inspire {:?}@{}", card_ptr, card_idx),
        Event::Quicken(card_idx, card_ptr, amount) => {
            format!("Quicken {:?}@{}+{}", card_ptr, card_idx, amount)
        }
        Event::Delay(card_idx, card_ptr, amount) => {
            format!("Delay {:?}@{}-{}", card_ptr, card_idx, amount)
        }
        Event::Pull(card_idx, card_ptr) => format!("Pull {:?}@{}", card_ptr, card_idx),
        Event::Push(card_idx, card_ptr) => format!("Push {:?}@{}", card_ptr, card_idx),
        Event::Teleport(card_idx1, card_ptr1, card_idx2, card_ptr2) => format!(
            "Teleport {:?}@{} <> {:?}@{}",
            card_ptr1, card_idx1, card_ptr2, card_idx2,
        ),
        Event::Mandatory(card_ptr, self_action) => {
            format!("{:?}: Forced({:?})", card_ptr, self_action)
        }
        Event::FireballTarget(card_idx, card_ptr) => {
            format!("{:?}@{}: Fireball", card_ptr, card_idx)
        }
        Event::Ablaze(card_idx1, card_ptr1, card_idx2, card_ptr2) => format!(
            "Ablaze {:?}@{}<>{:?}@{}",
            card_ptr1, card_idx1, card_ptr2, card_idx2,
        ),
        Event::ReactAssistUsed(card_idx, card_ptr, cost) => {
            format!("{:?}@{}: React Assited({:?})", card_ptr, card_idx, cost)
        }
        Event::Heal(card_idx, card_ptr) => {
            format!("{:?}@{}: Heal", card_ptr, card_idx)
        }
        Event::Revive(card_idx, card_ptr) => {
            format!("{:?}@{}: Revive", card_ptr, card_idx)
        }
        Event::Block(card_idx, card_ptr, cost) => {
            format!("{:?}@{}: Block({:?})", card_ptr, card_idx, cost)
        }
        Event::Dodge(card_idx, card_ptr, cost) => {
            format!("{:?}@{}: Dodge({:?})", card_ptr, card_idx, cost)
        }
        Event::OnHurt(card_idx, card_ptr) => {
            format!("{:?}@{}: Hurt", card_ptr, card_idx)
        }
        Event::PayEnergy(card_idx, card_ptr) => {
            format!("{:?}@{}: Energy", card_ptr, card_idx)
        }
        Event::Manouver(card_idx, card_ptr) => {
            format!("{:?}@{}: Manouver", card_ptr, card_idx)
        }
        Event::Swarm(card_idx, card_ptr) => {
            format!("{:?}@{}: Swarm", card_ptr, card_idx)
        } // _ => format!("{:?}", event),
    }
}

pub fn format_multiple_events(events: &[Event]) -> String {
    events
        .iter()
        .map(format_event_for_cli)
        .collect::<Vec<String>>()
        .join("|")
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

