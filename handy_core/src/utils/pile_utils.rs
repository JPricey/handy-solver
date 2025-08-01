use crate::game::card_defs::*;
use crate::game::card_ptr::*;
use crate::game::primitives::*;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;
use regex::Regex;
use strum::IntoEnumIterator;

pub fn get_start_from_classes(
    hero_class: Class,
    monster_class: Class,
    rng: &mut dyn RngCore,
) -> Pile {
    let mut cards = CARDS.get_cards_for_class(hero_class);
    cards.append(&mut CARDS.get_cards_for_class(monster_class));
    cards.shuffle(rng);

    let mut back_idx = cards.len() - 1;
    for i in (0..cards.len()).rev() {
        if cards[i].is_back_start {
            if i != back_idx {
                cards.swap(i, back_idx);
            }
            back_idx -= 1;
        }
    }

    let cards_vec = cards
        .iter()
        .map(|&card_def| CardPtr::new_from_id(card_def.id, FaceKey::A))
        .collect::<Vec<_>>();

    (&cards_vec as &[_]).try_into().unwrap()
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScenarioSelection {
    ScenarioOne(Class),
    ScenarioTwo(Class),
    ScenarioThree(Class),
}

fn _scenario_one(hero_class: Class, rng: &mut dyn RngCore) -> Pile {
    let mut cards = CARDS.get_cards_for_class(hero_class);
    cards.append(&mut CARDS.get_cards_for_class(Class::Ogre));

    let mut cards_vec = cards
        .iter()
        .map(|&card_def| CardPtr::new_from_id(card_def.id, FaceKey::A))
        .collect::<Vec<_>>();

    let c101 = CardPtr::new_from_id(101, FaceKey::C);
    cards_vec.push(c101);
    cards_vec.shuffle(rng);

    let c102 = CardPtr::new_from_id(102, FaceKey::A);
    cards_vec.push(c102);

    (&cards_vec as &[_]).try_into().unwrap()
}

fn _scenario_two(hero_class: Class, rng: &mut dyn RngCore) -> Pile {
    let mut cards = CARDS.get_cards_for_class(hero_class);
    cards.append(&mut CARDS.get_cards_for_class(Class::Vampire));

    let mut cards_vec = cards
        .iter()
        .map(|&card_def| CardPtr::new_from_id(card_def.id, FaceKey::A))
        .collect::<Vec<_>>();

    let c101 = CardPtr::new_from_id(111, FaceKey::A);
    cards_vec.push(c101);
    cards_vec.shuffle(rng);

    (&cards_vec as &[_]).try_into().unwrap()
}

fn _scenario_three(hero_class: Class, rng: &mut dyn RngCore) -> Pile {
    let mut cards = CARDS.get_cards_for_class(hero_class);
    cards.append(&mut CARDS.get_cards_for_class(Class::Troupe));

    let mut cards_vec = cards
        .iter()
        .map(|&card_def| CardPtr::new_from_id(card_def.id, FaceKey::A))
        .collect::<Vec<_>>();

    let c101 = CardPtr::new_from_id(101, FaceKey::B);
    cards_vec.push(c101);
    cards_vec.shuffle(rng);

    (&cards_vec as &[_]).try_into().unwrap()
}

pub fn get_scenario_starts(scenario_selection: ScenarioSelection, rng: &mut dyn RngCore) -> Pile {
    match scenario_selection {
        ScenarioSelection::ScenarioOne(hero) => _scenario_one(hero, rng),
        ScenarioSelection::ScenarioTwo(hero) => _scenario_two(hero, rng),
        ScenarioSelection::ScenarioThree(hero) => _scenario_three(hero, rng),
    }
}

pub fn get_random_face<R: Rng>(rng: &mut R) -> FaceKey {
    match rng.gen_range(0..4) {
        0 => FaceKey::A,
        1 => FaceKey::B,
        2 => FaceKey::C,
        3 => FaceKey::D,
        _ => panic!(),
    }
}

pub fn get_random_exhausted_face<R: Rng>(mut rng: &mut R, card_def: &CardDef) -> FaceKey {
    let faces: Vec<FaceKey> = FaceKey::iter()
        .filter(|f| card_def.faces[*f].health == Health::Empty)
        .collect();

    *faces.choose(&mut rng).unwrap()
}

pub fn string_to_card_id_result(input: &str) -> Result<CardId, String> {
    let card_id: CardId = input.parse().map_err(|err| format!("{err:?}"))?;

    if CARDS.get_card_if_exists(card_id as usize).is_some() {
        Ok(card_id)
    } else {
        Err("Card Id does not exist".to_owned())
    }
}

pub fn string_to_card_ptr_result(input: &str) -> Result<CardPtr, String> {
    let re = Regex::new(r"(\d+)([a-dA-D]?)").unwrap();
    let Some(captures) = re.captures(input) else {
        return Err("Could not parse card ptr".to_owned());
    };
    let (_, [id_str, key_str]) = captures.extract();
    let key: FaceKey = key_str
        .to_uppercase()
        .to_owned()
        .parse()
        .unwrap_or(FaceKey::A);

    let Ok(id) = id_str.parse::<CardId>() else {
        return Err("Card Id does not exist".to_owned());
    };

    if CARDS.get_card_if_exists(id as usize).is_some() {
        Ok(CardPtr::new_from_id(id, key))
    } else {
        Err("Card Id does not exist".to_owned())
    }
}

pub fn string_to_card_ptr(input: &str) -> CardPtr {
    string_to_card_ptr_result(input).unwrap()
}

pub fn string_to_pile_result(input: &str) -> Result<Pile, String> {
    let mut result = Pile::default();

    let re = Regex::new(r"(\d+[a-dA-D]?)").unwrap();

    for (_, [card_ptr_str]) in re.captures_iter(input).map(|c| c.extract()) {
        let card_ptr = string_to_card_ptr_result(card_ptr_str)?;
        result.push(card_ptr);

        if result.len() == MAX_PILE_LEN {
            return Ok(result);
        }
    }

    if result.len() == 0 {
        Err("Could not parse any cards".to_owned())
    } else {
        Ok(result)
    }
}

pub fn string_to_pile(input: &str) -> Pile {
    string_to_pile_result(input).unwrap()
}
