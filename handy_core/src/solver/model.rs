use crate::game::card_ptr::*;
use crate::game::*;
use crate::solver::vectorize::*;
use enum_map::enum_map;
use enum_map::EnumMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::{BTreeMap, HashSet};
use vectorize_derive::*;
use crate::solver::model_t::ModelT;

pub const HEROS: [Class; 5] = [
    Class::Warrior,
    Class::Huntress,
    Class::Pyro,
    Class::Cursed,
    Class::Beastmaster,
];

pub const BADDIES: [Class; 5] = [
    Class::Ogre,
    Class::Vampire,
    Class::Spider,
    Class::Demon,
    Class::Flora,
];

pub type Matchup = (Class, Class);

fn serialize_card_side_values<S, T>(
    value: &Vec<Option<EnumMap<FaceKey, T>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Clone + Serialize,
{
    let ordered: BTreeMap<CardId, Vec<T>> = value
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| v.as_ref().map(|u| (i, u)))
        .map(|(i, v)| (i as CardId, face_map_to_values_vec(&v)))
        .collect();
    ordered.serialize(serializer)
}

fn face_map_to_values_vec<T>(faces: &EnumMap<FaceKey, T>) -> Vec<T>
where
    T: Clone,
{
    faces.values().cloned().collect()
}

fn values_vec_to_face_map<T>(values: &[T]) -> EnumMap<FaceKey, T>
where
    T: Clone,
{
    assert!(values.len() == 4);
    enum_map! {
        FaceKey::A => values[0].clone(),
        FaceKey::B => values[1].clone(),
        FaceKey::C => values[2].clone(),
        FaceKey::D => values[3].clone(),
    }
}

fn deserialize_card_side_values<'de, D, T>(
    deserializer: D,
) -> Result<Vec<Option<EnumMap<FaceKey, T>>>, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Clone + Deserialize<'de>,
{
    let values = BTreeMap::<CardId, Vec<T>>::deserialize(deserializer)?;
    let mut result: Vec<Option<EnumMap<FaceKey, T>>> = Vec::new();

    for (id, value) in values {
        while result.len() <= id as usize {
            result.push(None);
        }

        result[id as usize] = Some(values_vec_to_face_map(&value));
    }

    Ok(result)
}

fn default_0() -> f32 {
    0.0
}

fn default_0_arr() -> [f32; 9] {
    [0.0; 9]
}

type CardFaceFeaturesType = EnumMap<FaceKey, FaceFeatures>;

#[derive(Vectorize, Serialize, Deserialize, PartialEq, Debug, Clone, Default)]
pub struct FaceFeatures {
    #[serde(default = "default_0")]
    pub value: f32,
    #[serde(default = "default_0_arr")]
    pub value_in_position: [f32; 9],
    #[serde(default = "default_0")]
    pub single_bad_touching_infront: f32,
    #[serde(default = "default_0")]
    pub single_bad_touching_behind: f32,
    #[serde(default = "default_0")]
    pub bad_touching_infront_coeff: f32,
    #[serde(default = "default_0")]
    pub bad_touching_behind_coeff: f32,
    #[serde(default = "default_0")]
    pub single_good_touching_infront: f32,
    #[serde(default = "default_0")]
    pub single_good_touching_behind: f32,
    #[serde(default = "default_0")]
    pub good_touching_infront_coeff: f32,
    #[serde(default = "default_0")]
    pub good_touching_behind_coeff: f32,
    #[serde(default = "default_0")]
    pub is_touching_start_through_allies: f32,
    #[serde(default = "default_0")]
    pub is_start_num_consecutive_allies: f32,
    #[serde(default = "default_0")]
    pub is_start_num_consecutive_enemies: f32,
    #[serde(default = "default_0")]
    pub num_energy: f32,
    #[serde(default = "default_0")]
    pub is_start_num_energy: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Model {
    #[serde(default = "default_0")]
    flat_value: f32,
    #[serde(
        serialize_with = "serialize_card_side_values",
        deserialize_with = "deserialize_card_side_values"
    )]
    card_face_features: Vec<Option<CardFaceFeaturesType>>,
}

impl Model {
    pub fn new() -> Self {
        Model {
            card_face_features: Vec::new(),
            flat_value: 0.0,
        }
    }

    pub fn new_empty_for_cards(cards: &[CardId]) -> Self {
        let mut res = Self::new();
        for c_id in cards {
            res.set(
                *c_id as usize,
                values_vec_to_face_map(&vec![FaceFeatures::default(); 4]),
            );
        }

        res
    }

    pub fn set(&mut self, i: usize, value: EnumMap<FaceKey, FaceFeatures>) {
        while self.card_face_features.len() <= i {
            self.card_face_features.push(None);
        }
        self.card_face_features[i] = Some(value);
    }

    pub fn try_get_face_features(&self, i: usize) -> Option<CardFaceFeaturesType> {
        if i >= self.card_face_features.len() {
            None
        } else {
            self.card_face_features[i].clone()
        }
    }

    pub fn trim_to_cards(&mut self, _: &Vec<CardId>) {
        // self.card_face_features.retain(|k, _| cards.contains(k));
    }

    pub fn score_pile(&self, pile: &Pile) -> f32 {
        let mut total = self.flat_value;

        let mut total_energy = 0;
        {
            // Forward pass
            // Calculate:
            // Total energy
            // Update:
            // value
            // value_in_position
            // baddie infront
            // good infront
            // is_start_num_consecutive_allies
            // is_start_num_consecutive_enemies
            let mut num_baddie_infront = 0;
            let mut num_good_infront = 0;
            let mut touching_start: Option<Allegiance> = None;
            for (i, card) in pile.iter().enumerate() {
                let def_face = card.get_active_face();

                let feature_face = &self.card_face_features[card.get_card_id() as usize]
                    .as_ref()
                    .unwrap()[card.get_card_face()];

                total += feature_face.value;
                total += feature_face.value_in_position[i];

                if num_baddie_infront > 0 {
                    total += feature_face.single_bad_touching_infront;
                    total += num_baddie_infront as f32 * feature_face.bad_touching_infront_coeff;
                }
                if num_good_infront > 0 {
                    total += feature_face.single_good_touching_infront;
                    total += num_good_infront as f32 * feature_face.good_touching_infront_coeff;
                }

                if def_face.allegiance == Allegiance::Hero {
                    num_good_infront += 1;
                    num_baddie_infront = 0;
                } else {
                    num_baddie_infront += 1;
                    num_good_infront = 0;
                }

                if i == 0 {
                    total += feature_face.is_touching_start_through_allies;
                    touching_start = Some(def_face.allegiance);
                } else if Some(def_face.allegiance) == touching_start {
                    total += feature_face.is_touching_start_through_allies;
                } else {
                    touching_start = None;
                }

                if def_face.features.intersects(Features::Energy) {
                    total_energy += 1;
                }
            }
        }

        {
            // Backward pass
            // baddie behind
            // good behind
            // num_energy
            // is_start_num_energy
            // is_start_num_consecutive_allies
            // is_start_num_consecutive_enemies
            let mut num_baddie_behind = 0;
            let mut num_good_behind = 0;
            let start_type = pile[0].get_active_face().allegiance;
            let mut num_ally_of_start = 0;
            let mut num_enemy_of_start = 0;
            for (i, card) in pile.iter().enumerate().rev() {
                let def_face = card.get_active_face();

                let feature_face = &self.card_face_features[card.get_card_id() as usize]
                    .as_ref()
                    .unwrap()[card.get_card_face()];

                if num_baddie_behind > 0 {
                    total += feature_face.single_bad_touching_behind;
                    total += num_baddie_behind as f32 * feature_face.bad_touching_behind_coeff;
                }
                if num_good_behind > 0 {
                    total += feature_face.single_good_touching_behind;
                    total += num_good_behind as f32 * feature_face.good_touching_behind_coeff;
                }

                if def_face.allegiance == Allegiance::Hero {
                    num_good_behind += 1;
                    num_baddie_behind = 0;
                } else {
                    num_baddie_behind += 1;
                    num_good_behind = 0;
                }

                total += total_energy as f32 * feature_face.num_energy;
                if i == 0 {
                    total += total_energy as f32 * feature_face.is_start_num_energy;
                    total +=
                        num_ally_of_start as f32 * feature_face.is_start_num_consecutive_allies;
                    total +=
                        num_enemy_of_start as f32 * feature_face.is_start_num_consecutive_enemies;
                } else {
                    if def_face.allegiance == start_type {
                        num_ally_of_start += 1;
                        num_enemy_of_start = 0;
                    } else {
                        num_enemy_of_start += 1;
                        num_ally_of_start = 0;
                    }
                }
            }
        }

        total
    }
}

pub fn training_ex_to_model(pile: &Pile) -> Model {
    let mut cards: Vec<CardId> = Vec::new();
    for card in pile.iter() {
        cards.push(card.get_card_id())
    }

    let total_energy = pile
        .iter()
        .filter(|c| c.get_active_face().features.intersects(Features::Energy))
        .count() as f32;

    let mut model = Model::new_empty_for_cards(&cards);
    model.flat_value = 1.0;

    for (i, card) in pile.iter().enumerate() {
        let current_card = model.card_face_features[card.get_card_id() as usize]
            .as_mut()
            .unwrap();
        let current_face = &mut current_card[card.get_card_face()];

        for j in i + 1..pile.len() {
            if pile[j].get_active_face().allegiance == Allegiance::Baddie {
                current_face.single_bad_touching_behind = 1.0;
                current_face.bad_touching_behind_coeff += 1.0;
            } else {
                break;
            }
        }

        for j in i + 1..pile.len() {
            if pile[j].get_active_face().allegiance != Allegiance::Baddie {
                current_face.single_good_touching_behind = 1.0;
                current_face.good_touching_behind_coeff += 1.0;
            } else {
                break;
            }
        }

        for j in (0..i).rev() {
            if pile[j].get_active_face().allegiance == Allegiance::Baddie {
                current_face.single_bad_touching_infront = 1.0;
                current_face.bad_touching_infront_coeff += 1.0;
            } else {
                break;
            }
        }

        for j in (0..i).rev() {
            if pile[j].get_active_face().allegiance != Allegiance::Baddie {
                current_face.single_good_touching_infront = 1.0;
                current_face.good_touching_infront_coeff += 1.0;
            } else {
                break;
            }
        }

        current_face.is_touching_start_through_allies = 1.0;
        for j in (0..i).rev() {
            if pile[j].get_active_face().allegiance != card.get_active_face().allegiance {
                current_face.is_touching_start_through_allies = 0.0;
                break;
            }
        }

        if i == 0 {
            for j in 1..pile.len() {
                if pile[j].get_active_face().allegiance == card.get_active_face().allegiance {
                    current_face.is_start_num_consecutive_allies += 1.0;
                } else {
                    break;
                }
            }

            for j in 1..pile.len() {
                if pile[j].get_active_face().allegiance != card.get_active_face().allegiance {
                    current_face.is_start_num_consecutive_enemies += 1.0;
                } else {
                    break;
                }
            }
        }

        current_face.num_energy = total_energy;
        if i == 0 {
            current_face.is_start_num_energy = total_energy;
        }
        current_face.value = 1.0;
        current_face.value_in_position[i] = 1.0;
    }

    model
}

impl Vectorize for Model {
    fn vectorize_append(&self, vec: &mut Vec<f32>) {
        self.flat_value.vectorize_append(vec);
        for maybe_value in self.card_face_features.iter() {
            if let Some(value) = maybe_value {
                value.vectorize_append(vec);
            }
        }
    }
    fn vectorize(&self) -> Vec<f32> {
        let mut res = Vec::new();
        self.vectorize_append(&mut res);
        res
    }
}

impl ModelT for Model {
    fn score_pile(&self, pile: &Pile) -> f32 {
        self.score_pile(pile)
    }
}

pub fn vec_to_model(vec: &Vec<f32>, cards: &[CardId]) -> Model {
    let mut sorted_cards: Vec<_> = cards.into();
    sorted_cards.sort();

    let mut res = Model::new_empty_for_cards(&sorted_cards);
    let mut iter = vec.iter().cloned();
    res.flat_value = f32::unvectorize(&mut iter);

    for card_id in sorted_cards {
        let card = CardFaceFeaturesType::unvectorize(&mut iter);
        res.set(card_id as usize, card);
    }
    assert!(iter.next().is_none());
    res
}

impl Unvectorize for FaceFeatures {
    fn unvectorize(iter: &mut dyn Iterator<Item = f32>) -> Self {
        let mut result = Self::default();
        result.value = iter.next().unwrap();
        result.value_in_position = [
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ];
        result.single_bad_touching_infront = iter.next().unwrap();
        result.single_bad_touching_behind = iter.next().unwrap();
        result.bad_touching_infront_coeff = iter.next().unwrap();
        result.bad_touching_behind_coeff = iter.next().unwrap();
        result.single_good_touching_infront = iter.next().unwrap();
        result.single_good_touching_behind = iter.next().unwrap();
        result.good_touching_infront_coeff = iter.next().unwrap();
        result.good_touching_behind_coeff = iter.next().unwrap();
        result.is_touching_start_through_allies = iter.next().unwrap();
        result.is_start_num_consecutive_allies = iter.next().unwrap();
        result.is_start_num_consecutive_enemies = iter.next().unwrap();
        result.num_energy = iter.next().unwrap();
        result.is_start_num_energy = iter.next().unwrap();

        result
    }
}

pub fn is_hero_class(class: Class) -> bool {
    HEROS.contains(&class)
}

pub fn get_classes_from_pile(pile: &Pile) -> Vec<Class> {
    let hashset: HashSet<Class> =
        HashSet::from_iter(pile.iter().map(|card| card.get_card_def().class));
    hashset.into_iter().collect()
}

pub fn find_any_in_list(list: &[Class], targets: &[Class]) -> Option<Class> {
    for x in list {
        if targets.contains(x) {
            return Some(*x);
        }
    }
    return None;
}

pub fn try_get_matchup_from_classes(classes: &Vec<Class>) -> Option<Matchup> {
    let maybe_hero = find_any_in_list(&classes, &HEROS);
    let maybe_enemy = find_any_in_list(&classes, &BADDIES);

    if let (Some(hero), Some(enemy)) = (maybe_hero, maybe_enemy) {
        Some((hero, enemy))
    } else {
        None
    }
}

pub fn try_get_matchup_from_pile(pile: &Pile) -> Option<Matchup> {
    let classes = get_classes_from_pile(pile);
    try_get_matchup_from_classes(&classes)
}

pub fn get_all_matchups_from_pile(pile: &Pile) -> Vec<Matchup> {
    let classes = get_classes_from_pile(pile);
    let mut heros = Vec::new();
    let mut baddies = Vec::new();
    for class in classes {
        if is_hero_class(class) {
            heros.push(class);
        } else {
            baddies.push(class);
        }
    }

    let mut result = Vec::new();
    for hero in &heros {
        for baddie in &baddies {
            result.push((hero.clone(), baddie.clone()));
        }
    }

    return result;
}

pub fn merge_models_for_pile(pile: &Pile, models: &Vec<Model>) -> Model {
    let mut result = Model::new();

    let mut flat_value_sum: f32 = 0.0;
    let mut flat_value_count: usize = 0;

    for card in pile {
        let card_id = card.get_card_id() as usize;

        let mut vectorized: Vec<Vec<f32>> = Vec::new();
        for model in models {
            flat_value_sum += model.flat_value;
            flat_value_count += 1;

            if let Some(ff) = model.try_get_face_features(card_id) {
                vectorized.push(ff.vectorize());
            }
        }

        let mut avg_vec = Vec::new();
        for i in 0..vectorized[0].len() {
            let mut sum = 0.0;
            for v in &vectorized {
                sum += v[i];
            }
            sum = sum / vectorized.len() as f32;
            avg_vec.push(sum);
        }

        let card_features = CardFaceFeaturesType::unvectorize(&mut avg_vec.into_iter());
        result.set(card_id, card_features);
    }

    result.flat_value = flat_value_sum / flat_value_count as f32;

    result
}

pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let mut product = 0.0;
    for i in 0..a.len() {
        product += a[i] * b[i];
    }
    product
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::{get_random_face, get_start_from_classes, string_to_pile};
    use rand::seq::SliceRandom;
    use rand::{thread_rng, Rng};
    use serde_json;

    const MODEL_SIZE_FOR_9_CARDS: usize = 829;

    fn get_random_float_vec() -> Vec<f32> {
        let mut float_vec: Vec<f32> = Vec::new();
        let mut rng = thread_rng();

        for _ in 0..MODEL_SIZE_FOR_9_CARDS {
            float_vec.push(rng.gen::<f32>());
        }
        float_vec
    }

    #[test]
    fn test_enum_nested() {
        let matchup =
            try_get_matchup_from_pile(&string_to_pile("2A, 6A, 5A, 4A, 8A, 7A, 3A, 9A, 1A"));

        assert_eq!(matchup, Some((Class::Warrior, Class::Ogre)));
    }

    #[test]
    fn test_model_size() {
        let model = Model::new_empty_for_cards(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(model.vectorize().len(), MODEL_SIZE_FOR_9_CARDS);
    }

    #[test]
    fn test_vectorize_unvectorize() {
        let float_vec = get_random_float_vec();

        let model = vec_to_model(&float_vec, &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        let result = model.vectorize();

        assert_eq!(float_vec, result);
    }

    #[test]
    fn test_model_vs_vec_score() {
        let mut rng = thread_rng();
        let start_pile = get_start_from_classes(Class::Warrior, Class::Ogre, &mut rng);
        let cards: Vec<_> = start_pile.iter().map(|c| c.get_card_id()).collect();

        for _ in 0..100 {
            let mut end_pile = start_pile.clone();
            end_pile.shuffle(&mut rng);
            for c in end_pile.iter_mut() {
                c.key = get_random_face(&mut rng);
            }

            let vec_model = get_random_float_vec();
            let reg_model = vec_to_model(&vec_model, &cards);
            let vec_pile = training_ex_to_model(&end_pile).vectorize();

            let model_score = reg_model.score_pile(&end_pile);
            let vec_score = dot_product(&vec_model, &vec_pile);

            assert!((model_score - vec_score).abs() < 0.0001);
        }
    }

    #[test]
    fn test_model_serde() {
        let cards = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut rng = thread_rng();
        for _ in 0..100 {
            let mut card_clone = cards.clone();
            card_clone.shuffle(&mut rng);

            let vec_model = get_random_float_vec();
            let reg_model = vec_to_model(&vec_model, &cards);

            let model_str = serde_json::to_string(&reg_model).unwrap();
            let result_model: Model = serde_json::from_str(&model_str).unwrap();

            assert_eq!(reg_model, result_model);
        }
    }
}
