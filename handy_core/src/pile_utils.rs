use regex::Regex;
use super::types::*;

pub fn string_to_card_ptr_result(input: &str) -> Result<CardPtr, ()> {
    let re = Regex::new(r"(\d+)([a-dA-D]?)").unwrap();
    let Some(captures) = re.captures(input) else { return Err(()) };
    let (_, [id_str, key_str]) = captures.extract();
    let key: FaceKey = key_str
        .to_uppercase()
        .to_owned()
        .parse()
        .unwrap_or(FaceKey::A);

    let Ok(id) = id_str.parse() else { return Err(()) };
    Ok(CardPtr::new_from_id(id, key))
}

pub fn string_to_card_ptr(input: &str) -> CardPtr {
    string_to_card_ptr_result(input).unwrap()
}

pub fn string_to_pile_result(input: &str) -> Result<Pile, String> {
    let mut result = vec![];

    let re = Regex::new(r"(\d+[a-dA-D]?)").unwrap();

    for (_, [card_ptr_str]) in re.captures_iter(input).map(|c| c.extract()) {
        let Ok(card_ptr) = string_to_card_ptr_result(card_ptr_str) else { return Err(card_ptr_str.to_owned()) };
        result.push(card_ptr);
    }

    if result.len() == 0 {
        Err("Could not parse any cards".to_owned())
    } else {
        Ok(result.into())
    }
}

pub fn string_to_pile(input: &str) -> Pile {
    string_to_pile_result(input).unwrap()
}

