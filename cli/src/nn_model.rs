use crate::model_t::ModelT;
use handy_core::game::*;
use std::collections::HashMap;
use tch::{nn::Module, Kind, Tensor};

pub const PILE_SIZE: i64 = 9;
pub const CARD_SIZE: i64 = 13;
pub const INPUT_SIZE: i64 = PILE_SIZE * CARD_SIZE;

type CardMap = HashMap<CardId, usize>;

pub struct NNModel {
    // hero: Class,
    // monster: Class,
    card_map: CardMap,
    net: Box<dyn Module>,
}

impl NNModel {
    pub fn new(hero: Class, monster: Class, net: Box<dyn Module>) -> Self {
        let card_map = make_card_map(hero, monster);
        Self {
            // hero,
            // monster,
            card_map,
            net,
        }
    }

    pub fn score_pile(&self, pile: &Pile) -> f32 {
        let x = pile_onehot(&self.card_map, &pile);
        let res = self.net.forward(&x.unsqueeze(0));
        return res.double_value(&[0]) as f32;
    }

    pub fn score_piles(&self, piles: &[Pile]) -> Tensor {
        let vec_x: Vec<Tensor> = piles
            .iter()
            .map(|p| pile_onehot(&self.card_map, p))
            .collect();
        let xs = Tensor::stack(&vec_x, 0);

        let res = self.net.forward(&xs);
        return res;
    }
}

impl ModelT for NNModel {
    fn score_pile(&self, pile: &Pile) -> f32 {
        self.score_pile(pile)
    }
}

pub fn empty_onehot() -> Tensor {
    let vec_source: Vec<f32> = vec![0.0; INPUT_SIZE as usize];
    let as_bytes = to_byte_slice(&vec_source);
    return Tensor::from_data_size(&as_bytes, &[CARD_SIZE, PILE_SIZE], Kind::Float);
}

pub fn make_card_map(hero: Class, monster: Class) -> CardMap {
    let mut cards = CARDS.get_cards_for_class(hero);
    cards.append(&mut CARDS.get_cards_for_class(monster));

    let mut card_ids: Vec<CardId> = cards.iter().map(|c| c.id).collect();
    card_ids.sort();

    let mut card_map = CardMap::new();
    for (idx, card_id) in card_ids.iter().enumerate() {
        card_map.insert(*card_id, idx);
    }
    card_map
}

pub fn to_byte_slice<'a>(floats: &'a [f32]) -> &'a [u8] {
    unsafe { std::slice::from_raw_parts(floats.as_ptr() as *const _, floats.len() * 4) }
}

pub fn face_to_idx(face_key: FaceKey) -> usize {
    match face_key {
        FaceKey::A => 0,
        FaceKey::B => 1,
        FaceKey::C => 2,
        FaceKey::D => 3,
    }
}

pub fn pile_onehot(card_map: &CardMap, pile: &Pile) -> Tensor {
    let mut vec_source: Vec<f32> = vec![0.0; INPUT_SIZE as usize];
    for i in 0..9 {
        let card_idx = card_map[&pile[i].card_id];
        let face_idx = PILE_SIZE as usize + face_to_idx(pile[i].get_card_face());
        let card_1d = card_idx * PILE_SIZE as usize + i;
        let face_1d = face_idx * PILE_SIZE as usize + i;
        // dbg!(card_idx, face_idx, i, card_1d, face_1d);
        vec_source[card_1d] = 1.0;
        vec_source[face_1d] = 1.0;
    }

    let as_bytes = to_byte_slice(&vec_source);
    return Tensor::from_data_size(&as_bytes, &[CARD_SIZE, PILE_SIZE], Kind::Float);
}
