use handy_core::game::*;
use handy_core::utils::*;
use serde;
use serde::Serialize;
use serde_json;

use rand::thread_rng;
use rand::Rng;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;
use std::str;

const RANDOMIZE_SIZED_PCT: usize = 30;
const RANDOMIZE_HERO_SIDES_PCT: usize = 30;

const HERO_CLASS: Class = Class::Cursed;
const MONSTER_CLASS: Class = Class::Demon;

#[no_mangle]
pub extern "C" fn next_pile_states(c_string_ptr: *const c_char) -> *mut c_char {
    let bytes = unsafe { CStr::from_ptr(c_string_ptr).to_bytes() };
    let string = str::from_utf8(bytes).unwrap();

    let start_pile = string_to_pile(string);
    let child_states = resolve_top_card(&GameStateNoEventLog::new(start_pile));
    let child_piles: Vec<Vec<CardPtr>> = child_states
        .into_iter()
        .map(|s| s.pile.into_iter().collect())
        .collect();

    let new_string = serde_json::to_string(&child_piles).unwrap();

    let c_string = CString::new(new_string.clone()).unwrap();
    let result = c_string.into_raw();
    return result;
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ChildPile {
    #[serde(serialize_with = "serialize_pile")]
    pile: Pile,
    winner: char,
}

fn serialize_pile<S>(value: &Pile, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    (&value as &[_]).serialize(serializer)
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Example {
    #[serde(serialize_with = "serialize_pile")]
    parent_pile: Pile,
    child_piles: Vec<ChildPile>,
}

pub fn randomize_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
    for card_ptr in pile.iter_mut() {
        card_ptr.key = get_random_face(rng);
    }

    if is_game_winner(pile).is_some() {
        randomize_sides(pile, rng);
    }
}

pub fn randomize_hero_sides<R: Rng>(pile: &mut Pile, rng: &mut R) {
    for card_ptr in pile.iter_mut() {
        if card_ptr.get_active_face().allegiance != Allegiance::Baddie {
            card_ptr.key = get_random_face(rng);
        }
    }

    if is_game_winner(pile).is_some() {
        randomize_hero_sides(pile, rng);
    }
}

#[no_mangle]
pub extern "C" fn get_example() -> *mut c_char {
    let mut start_pile = get_start_from_classes(HERO_CLASS, MONSTER_CLASS, &mut rand::thread_rng());

    let mut rng = thread_rng();
    randomize_sides(&mut start_pile, &mut rng);

    if is_game_winner(&start_pile).is_some() {
        return get_example();
    }

    let child_states = resolve_top_card(&GameStateNoEventLog::new(start_pile.clone()));
    let child_piles: Vec<ChildPile> = child_states
        .into_iter()
        .map(|s| {
            let pile = s.pile;
            let winner = match is_game_winner(&pile) {
                Some(x) => match x {
                    Allegiance::Hero => 'h',
                    _ => 'b',
                },
                None => 'n',
            };
            return ChildPile { pile, winner };
        })
        .collect();

    let result = Example {
        parent_pile: start_pile,
        child_piles,
    };

    let new_string = serde_json::to_string(&result).unwrap();
    let c_string = CString::new(new_string.clone()).unwrap();
    let result = c_string.into_raw();
    return result;
}

#[no_mangle]
pub extern "C" fn get_won_pile() -> *mut c_char {
    let mut start_pile = get_start_from_classes(HERO_CLASS, MONSTER_CLASS, &mut rand::thread_rng());

    let mut rng = thread_rng();
    randomize_sides(&mut start_pile, &mut rng);
    for card in &mut start_pile {
        if card.get_active_face().allegiance == Allegiance::Baddie {
            card.key = get_random_exhausted_face(&mut rng, card.get_card_def());
        }
    }

    let new_string = serde_json::to_string(&start_pile as &[CardPtr]).unwrap();
    let c_string = CString::new(new_string.clone()).unwrap();
    let result = c_string.into_raw();
    return result;
}
