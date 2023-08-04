use crate::game::card_defs::*;
use crate::game::card_ptr::*;
use crate::game::primitives::*;
use rand::seq::SliceRandom;
use rand::Rng;
use rand::RngCore;
use regex::Regex;

pub fn get_start_from_classes(
    hero_class: Class,
    monster_class: Class,
    rng: &mut dyn RngCore,
) -> Pile {
    let mut cards = CARDS.get_cards_for_class(hero_class);
    cards.append(&mut CARDS.get_cards_for_class(monster_class));
    cards.shuffle(rng);

    let cards_vec = cards
        .iter()
        .map(|&card_def| CardPtr::new_from_id(card_def.id, FaceKey::A))
        .collect::<Vec<_>>();

    return (&cards_vec as &[_]).try_into().unwrap();
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

pub fn string_to_card_ptr_result(input: &str) -> Result<CardPtr, ()> {
    let re = Regex::new(r"(\d+)([a-dA-D]?)").unwrap();
    let Some(captures) = re.captures(input) else {
        return Err(());
    };
    let (_, [id_str, key_str]) = captures.extract();
    let key: FaceKey = key_str
        .to_uppercase()
        .to_owned()
        .parse()
        .unwrap_or(FaceKey::A);

    let Ok(id) = id_str.parse::<u8>() else {
        return Err(());
    };

    if CARDS.get_card_if_exists(id as usize).is_some() {
        Ok(CardPtr::new_from_id(id, key))
    } else {
        Err(())
    }
}

pub fn string_to_card_ptr(input: &str) -> CardPtr {
    string_to_card_ptr_result(input).unwrap()
}

pub fn string_to_pile_result(input: &str) -> Result<Pile, String> {
    let mut result = Pile::default();

    let re = Regex::new(r"(\d+[a-dA-D]?)").unwrap();

    for (_, [card_ptr_str]) in re.captures_iter(input).map(|c| c.extract()) {
        let Ok(card_ptr) = string_to_card_ptr_result(card_ptr_str) else {
            return Err(card_ptr_str.to_owned());
        };
        result.push(card_ptr);

        if result.len() == 9 {
            return Ok(result)
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
