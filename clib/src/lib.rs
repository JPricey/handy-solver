use handy_core::game::*;
use handy_core::utils::*;
use serde;
use serde::ser::{SerializeMap, SerializeSeq};
use serde::Serialize;
use serde_json;
use std::collections::{HashMap, HashSet};

use rand::thread_rng;
use rand::Rng;
use std::ffi::CStr;
use std::ffi::CString;
use std::mem::swap;
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

fn serialize_pile_vec<S>(value: &Vec<Pile>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(value.len()))?;
    for b in value {
        seq.serialize_element(&b as &[_])?;
    }
    seq.end()
}

fn serialize_pile_set<S>(value: &HashSet<Pile>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(value.len()))?;
    for b in value {
        seq.serialize_element(&b as &[_])?;
    }
    seq.end()
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

const PILE_WINNER_HERO: char = 'h';
const PILE_WINNER_BADDIE: char = 'b';
const PILE_WINNER_NOONE: char = 'n';

fn _pile_to_game_winner_char(pile: &Pile) -> char {
    match is_game_winner(pile) {
        Some(x) => match x {
            Allegiance::Hero => PILE_WINNER_HERO,
            _ => PILE_WINNER_BADDIE,
        },
        None => PILE_WINNER_NOONE,
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
            let winner = _pile_to_game_winner_char(&pile);
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

const DEEP_EXAMPLE_RESULT_CUTOFF: usize = 1000;

#[derive(Clone, Debug, PartialEq, Serialize)]
struct DeepExample {
    #[serde(serialize_with = "serialize_pile_set")]
    children: HashSet<Pile>,
    depth: usize,
    winner: char,
}

fn serialize_deep_example_map<S>(
    value: &HashMap<Pile, DeepExample>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut seq = serializer.serialize_seq(Some(value.len()))?;
    for (k, v) in value {
        seq.serialize_element(&(k as &[_], v))?;
    }
    seq.end()
}

#[derive(Clone, Debug, PartialEq, Serialize)]
struct FinalDeepExample {
    #[serde(serialize_with = "serialize_pile")]
    root_pile: Pile,
    #[serde(serialize_with = "serialize_deep_example_map")]
    examples: HashMap<Pile, DeepExample>,
    max_depth: usize,
}

#[no_mangle]
pub extern "C" fn get_deep_example() -> *mut c_char {
    let mut start_pile = get_start_from_classes(HERO_CLASS, MONSTER_CLASS, &mut rand::thread_rng());

    let mut rng = thread_rng();
    randomize_sides(&mut start_pile, &mut rng);

    if is_game_winner(&start_pile).is_some() {
        return get_deep_example();
    }

    let mut res_map: HashMap<Pile, DeepExample> = HashMap::new();

    let mut num_examples = 1;
    let mut depth: usize = 0;
    let mut next_piles = vec![start_pile.clone()];

    res_map.insert(
        start_pile.clone(),
        DeepExample {
            children: HashSet::new(),
            depth,
            winner: _pile_to_game_winner_char(&start_pile),
        },
    );

    loop {
        if num_examples > DEEP_EXAMPLE_RESULT_CUTOFF || next_piles.len() == 0 {
            break;
        }
        depth += 1;

        let mut cur_layer_piles: Vec<Pile> = Vec::new();
        swap(&mut cur_layer_piles, &mut next_piles);
        for current_pile in cur_layer_piles {
            let child_states = resolve_top_card(&GameStateNoEventLog::new(current_pile.clone()));

            {
                let this_example = res_map.get_mut(&current_pile).unwrap();
                for child_state in &child_states {
                    let child_pile = &child_state.pile;
                    this_example.children.insert(child_pile.clone());
                }
            }

            for child_state in child_states {
                let child_pile = child_state.pile;
                if !res_map.contains_key(&child_pile) {
                    let winner = _pile_to_game_winner_char(&child_pile);
                    res_map.insert(
                        child_pile.clone(),
                        DeepExample {
                            children: HashSet::new(),
                            depth,
                            winner,
                        },
                    );

                    if winner == PILE_WINNER_NOONE {
                        next_piles.push(child_pile.clone());
                        num_examples += 1;
                    }
                }
            }
        }
    }

    let final_res = FinalDeepExample {
        root_pile: start_pile.clone(),
        max_depth: depth,
        examples: res_map,
    };
    let serialized_result = serde_json::to_string(&final_res).unwrap();
    let c_string = CString::new(serialized_result).unwrap();
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
